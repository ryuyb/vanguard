use std::sync::Arc;
use std::time::Duration;

use crate::application::dto::vault::{
    CopyCipherFieldCommand, CopyCipherFieldResult, GetCipherDetailQuery, VaultCipherDetail,
    VaultCopyField,
};
use crate::application::ports::clipboard_port::ClipboardPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::totp::{current_unix_seconds, generate_current_totp};
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::support::error::AppError;
use crate::support::result::AppResult;

const MAX_CLEAR_AFTER_MS: u64 = 300_000;

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
            let snapshot = generate_current_totp(&raw_totp, current_unix_seconds()?)?;
            Ok(snapshot.code)
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
