use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use url::Url;

use crate::application::dto::vault::{
    CopyCipherFieldCommand, CopyCipherFieldResult, GetCipherDetailQuery, VaultCipherDetail,
    VaultCopyField,
};
use crate::application::ports::clipboard_port::ClipboardPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::support::error::AppError;
use crate::support::result::AppResult;

const MAX_CLEAR_AFTER_MS: u64 = 300_000;
const BASE32_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

#[derive(Debug, Clone, Copy)]
enum TotpHashAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone)]
struct TotpConfig {
    secret: Vec<u8>,
    hash_algorithm: TotpHashAlgorithm,
    digits: u32,
    period: u64,
}

#[derive(Clone)]
pub struct CopyCipherFieldUseCase {
    get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>,
    clipboard_port: Arc<dyn ClipboardPort>,
}

impl CopyCipherFieldUseCase {
    pub fn new(
        get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>,
        clipboard_port: Arc<dyn ClipboardPort>,
    ) -> Self {
        Self {
            get_cipher_detail_use_case,
            clipboard_port,
        }
    }

    pub async fn execute(
        &self,
        runtime: &dyn VaultRuntimePort,
        command: CopyCipherFieldCommand,
    ) -> AppResult<CopyCipherFieldResult> {
        let cipher_id = command.cipher_id.trim();
        if cipher_id.is_empty() {
            return Err(AppError::validation("cipher_id cannot be empty"));
        }

        let clear_after_ms = validate_clear_after_ms(command.clear_after_ms)?;
        let account_id = runtime.active_account_id()?;
        let user_key = runtime
            .get_vault_user_key_material(&account_id)?
            .ok_or_else(|| {
                AppError::validation("vault is locked, please unlock with master password first")
            })?;

        let cipher = self
            .get_cipher_detail_use_case
            .execute(GetCipherDetailQuery {
                account_id,
                cipher_id: String::from(cipher_id),
                user_key,
            })
            .await?;

        let value = resolve_copy_value(&cipher, command.field)?;
        self.clipboard_port.write_text(&value)?;

        if let Some(delay_ms) = clear_after_ms {
            let clipboard_port = Arc::clone(&self.clipboard_port);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                if let Err(error) = clipboard_port.clear() {
                    log::warn!(
                        target: "vanguard::application::vault_copy",
                        "failed to clear clipboard after timeout: [{}] {}",
                        error.code(),
                        error.log_message(),
                    );
                }
            });
        }

        Ok(CopyCipherFieldResult {
            copied: true,
            clear_after_ms,
        })
    }
}

fn validate_clear_after_ms(value: Option<u64>) -> AppResult<Option<u64>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value == 0 {
        return Err(AppError::validation(
            "clear_after_ms must be greater than 0",
        ));
    }
    if value > MAX_CLEAR_AFTER_MS {
        return Err(AppError::validation(format!(
            "clear_after_ms cannot exceed {}",
            MAX_CLEAR_AFTER_MS
        )));
    }
    Ok(Some(value))
}

fn resolve_copy_value(cipher: &VaultCipherDetail, field: VaultCopyField) -> AppResult<String> {
    match field {
        VaultCopyField::Username => pick_first_non_empty(&[
            cipher
                .login
                .as_ref()
                .and_then(|entry| entry.username.clone()),
            cipher
                .data
                .as_ref()
                .and_then(|entry| entry.username.clone()),
            cipher
                .identity
                .as_ref()
                .and_then(|entry| entry.username.clone()),
        ])
        .ok_or_else(|| AppError::validation("requested field is empty: username")),
        VaultCopyField::Password => pick_first_non_empty(&[
            cipher
                .login
                .as_ref()
                .and_then(|entry| entry.password.clone()),
            cipher
                .data
                .as_ref()
                .and_then(|entry| entry.password.clone()),
        ])
        .ok_or_else(|| AppError::validation("requested field is empty: password")),
        VaultCopyField::Totp => {
            let raw_totp = pick_first_non_empty(&[
                cipher.login.as_ref().and_then(|entry| entry.totp.clone()),
                cipher.data.as_ref().and_then(|entry| entry.totp.clone()),
            ])
            .ok_or_else(|| AppError::validation("requested field is empty: totp"))?;
            let config = parse_totp_config(&raw_totp)?;
            let unix_seconds = current_unix_seconds()?;
            generate_totp_code(&config, unix_seconds)
        }
    }
}

fn pick_first_non_empty(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .filter_map(|value| value.as_ref())
        .find(|value| !value.trim().is_empty())
        .cloned()
}

fn parse_totp_config(raw_totp: &str) -> AppResult<TotpConfig> {
    let raw = raw_totp.trim();
    if raw.is_empty() {
        return Err(AppError::validation("invalid totp configuration"));
    }

    let mut secret_text: Option<String> = None;
    let mut hash_algorithm = TotpHashAlgorithm::Sha1;
    let mut digits: u32 = 6;
    let mut period: u64 = 30;

    if raw.to_ascii_lowercase().starts_with("otpauth://") {
        let url =
            Url::parse(raw).map_err(|_| AppError::validation("invalid totp configuration"))?;
        if url.scheme() != "otpauth"
            || url.host_str().map(|value| value.to_ascii_lowercase()) != Some(String::from("totp"))
        {
            return Err(AppError::validation("invalid totp configuration"));
        }

        for (key, value) in url.query_pairs() {
            if key.eq_ignore_ascii_case("secret") {
                secret_text = Some(value.into_owned());
            } else if key.eq_ignore_ascii_case("algorithm") {
                hash_algorithm = parse_totp_algorithm(value.as_ref())?;
            } else if key.eq_ignore_ascii_case("digits") {
                digits = parse_totp_digits(value.as_ref())?;
            } else if key.eq_ignore_ascii_case("period") {
                period = parse_totp_period(value.as_ref())?;
            }
        }

        if secret_text.is_none() {
            return Err(AppError::validation("invalid totp configuration"));
        }
    } else {
        secret_text = Some(String::from(raw));
    }

    let secret = decode_base32_secret(
        secret_text
            .as_deref()
            .ok_or_else(|| AppError::validation("invalid totp configuration"))?,
    )?;

    Ok(TotpConfig {
        secret,
        hash_algorithm,
        digits,
        period,
    })
}

fn parse_totp_algorithm(raw_value: &str) -> AppResult<TotpHashAlgorithm> {
    let normalized = raw_value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase();
    match normalized.as_str() {
        "SHA1" => Ok(TotpHashAlgorithm::Sha1),
        "SHA256" => Ok(TotpHashAlgorithm::Sha256),
        "SHA512" => Ok(TotpHashAlgorithm::Sha512),
        _ => Err(AppError::validation("invalid totp configuration")),
    }
}

fn parse_totp_digits(raw_value: &str) -> AppResult<u32> {
    let parsed = raw_value
        .parse::<u32>()
        .map_err(|_| AppError::validation("invalid totp configuration"))?;
    if !(6..=10).contains(&parsed) {
        return Err(AppError::validation("invalid totp configuration"));
    }
    Ok(parsed)
}

fn parse_totp_period(raw_value: &str) -> AppResult<u64> {
    let parsed = raw_value
        .parse::<u64>()
        .map_err(|_| AppError::validation("invalid totp configuration"))?;
    if parsed == 0 || parsed > 300 {
        return Err(AppError::validation("invalid totp configuration"));
    }
    Ok(parsed)
}

fn decode_base32_secret(raw_secret: &str) -> AppResult<Vec<u8>> {
    let sanitized = raw_secret
        .trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|character| {
            !character.is_ascii_whitespace() && *character != '-' && *character != '='
        })
        .collect::<String>();
    if sanitized.is_empty() {
        return Err(AppError::validation("invalid totp configuration"));
    }

    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits_in_buffer: u8 = 0;

    for character in sanitized.chars() {
        let Some(index) = BASE32_ALPHABET.find(character) else {
            return Err(AppError::validation("invalid totp configuration"));
        };
        buffer = (buffer << 5) | u32::try_from(index).expect("base32 index fits in u32");
        bits_in_buffer += 5;

        while bits_in_buffer >= 8 {
            output.push(
                ((buffer >> u32::from(bits_in_buffer - 8)) & 0xff)
                    .try_into()
                    .expect("masked byte fits in u8"),
            );
            bits_in_buffer -= 8;
        }
    }

    if output.is_empty() {
        return Err(AppError::validation("invalid totp configuration"));
    }

    Ok(output)
}

fn current_unix_seconds() -> AppResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AppError::internal("failed to read system time for totp generation"))
}

fn generate_totp_code(config: &TotpConfig, unix_seconds: u64) -> AppResult<String> {
    let counter = unix_seconds / config.period;
    let counter_bytes = counter.to_be_bytes();
    let signature = hmac_sign(config.hash_algorithm, &config.secret, &counter_bytes)?;
    if signature.len() < 20 {
        return Err(AppError::internal("failed to generate totp code"));
    }

    let offset = (signature[signature.len() - 1] & 0x0f) as usize;
    if offset + 3 >= signature.len() {
        return Err(AppError::internal("failed to generate totp code"));
    }

    let binary = ((u32::from(signature[offset]) & 0x7f) << 24)
        | (u32::from(signature[offset + 1]) << 16)
        | (u32::from(signature[offset + 2]) << 8)
        | u32::from(signature[offset + 3]);
    let divisor = 10_u64.pow(config.digits);
    let otp = u64::from(binary) % divisor;
    Ok(format!("{otp:0width$}", width = config.digits as usize))
}

fn hmac_sign(algorithm: TotpHashAlgorithm, secret: &[u8], data: &[u8]) -> AppResult<Vec<u8>> {
    match algorithm {
        TotpHashAlgorithm::Sha1 => {
            type HmacSha1 = Hmac<Sha1>;
            let mut mac = HmacSha1::new_from_slice(secret)
                .map_err(|_| AppError::validation("invalid totp configuration"))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        TotpHashAlgorithm::Sha256 => {
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(secret)
                .map_err(|_| AppError::validation("invalid totp configuration"))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        TotpHashAlgorithm::Sha512 => {
            type HmacSha512 = Hmac<Sha512>;
            let mut mac = HmacSha512::new_from_slice(secret)
                .map_err(|_| AppError::validation("invalid totp configuration"))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_totp_code, parse_totp_config};

    #[test]
    fn generate_totp_from_base32_secret() {
        let config = parse_totp_config("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ").expect("parse config");
        let code = generate_totp_code(&config, 59).expect("generate code");
        assert_eq!(code, "287082");
    }

    #[test]
    fn generate_totp_from_otpauth_uri() {
        let config = parse_totp_config(
            "otpauth://totp/Vanguard?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&algorithm=SHA1&digits=8&period=30",
        )
        .expect("parse config");
        let code = generate_totp_code(&config, 59).expect("generate code");
        assert_eq!(code, "94287082");
    }

    #[test]
    fn invalid_totp_algorithm_returns_error() {
        let result =
            parse_totp_config("otpauth://totp/Vanguard?secret=GEZDGNBVGY3TQOJQ&algorithm=MD5");
        assert!(result.is_err());
    }
}
