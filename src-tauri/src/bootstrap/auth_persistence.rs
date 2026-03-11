use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::support::error::AppError;
use crate::support::result::AppResult;

const AUTH_STATE_VERSION: u8 = 1;
const WRAP_ALGORITHM: &str = "xchacha20poly1305";
const WRAP_KDF: &str = "argon2id";
const WRAP_SALT_LEN: usize = 16;
const WRAP_NONCE_LEN: usize = 24;
const WRAP_KEY_LEN: usize = 32;
const WRAP_KDF_MEMORY_KIB: u32 = 65_536;
const WRAP_KDF_ITERATIONS: u32 = 3;
const WRAP_KDF_PARALLELISM: u32 = 1;

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionWrapRuntime {
    key: [u8; WRAP_KEY_LEN],
    kdf_memory_kib: u32,
    kdf_iterations: u32,
    kdf_parallelism: u32,
    salt_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedAuthState {
    pub version: u8,
    pub account_id: String,
    pub base_url: String,
    pub email: String,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
    pub encrypted_session: PersistedEncryptedSession,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct PersistedAuthStateContext {
    pub account_id: String,
    pub base_url: String,
    pub email: String,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

impl PersistedAuthState {
    pub fn new(
        context: PersistedAuthStateContext,
        encrypted_session: PersistedEncryptedSession,
    ) -> AppResult<Self> {
        Ok(Self {
            version: AUTH_STATE_VERSION,
            account_id: context.account_id,
            base_url: context.base_url,
            email: context.email,
            kdf: context.kdf,
            kdf_iterations: context.kdf_iterations,
            kdf_memory: context.kdf_memory,
            kdf_parallelism: context.kdf_parallelism,
            encrypted_session,
            updated_at_ms: now_unix_ms()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedEncryptedSession {
    pub algorithm: String,
    pub kdf: String,
    pub kdf_memory_kib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

pub fn encrypt_refresh_token(
    master_password: &str,
    account_id: &str,
    base_url: &str,
    email: &str,
    refresh_token: &str,
) -> AppResult<(PersistedEncryptedSession, SessionWrapRuntime)> {
    if master_password.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "master_password cannot be empty".to_string(),
        });
    }
    if refresh_token.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "refresh_token cannot be empty".to_string(),
        });
    }

    let mut salt = [0u8; WRAP_SALT_LEN];
    rand::rng().fill(&mut salt);
    let key = derive_wrap_key(
        master_password,
        &salt,
        WRAP_KDF_MEMORY_KIB,
        WRAP_KDF_ITERATIONS,
        WRAP_KDF_PARALLELISM,
    )?;
    let runtime = SessionWrapRuntime {
        key,
        kdf_memory_kib: WRAP_KDF_MEMORY_KIB,
        kdf_iterations: WRAP_KDF_ITERATIONS,
        kdf_parallelism: WRAP_KDF_PARALLELISM,
        salt_b64: STANDARD_NO_PAD.encode(salt),
    };
    let encrypted =
        encrypt_refresh_token_with_runtime(&runtime, account_id, base_url, email, refresh_token)?;
    Ok((encrypted, runtime))
}

pub fn decrypt_refresh_token(
    master_password: &str,
    account_id: &str,
    base_url: &str,
    email: &str,
    encrypted: &PersistedEncryptedSession,
) -> AppResult<(String, SessionWrapRuntime)> {
    if master_password.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "master_password cannot be empty".to_string(),
        });
    }
    if encrypted.algorithm != WRAP_ALGORITHM {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!(
                "unsupported encrypted session algorithm: {}",
                encrypted.algorithm
            ),
        });
    }
    if encrypted.kdf != WRAP_KDF {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("unsupported encrypted session kdf: {}", encrypted.kdf),
        });
    }

    let salt = decode_fixed_len(
        &encrypted.salt_b64,
        WRAP_SALT_LEN,
        "encrypted_session.salt_b64",
    )?;
    let nonce = decode_fixed_len(
        &encrypted.nonce_b64,
        WRAP_NONCE_LEN,
        "encrypted_session.nonce_b64",
    )?;
    let ciphertext = STANDARD_NO_PAD
        .decode(encrypted.ciphertext_b64.as_bytes())
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "encrypted_session.ciphertext_b64 is not valid base64".to_string(),
        })?;
    if ciphertext.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "encrypted_session.ciphertext_b64 is empty".to_string(),
        });
    }

    let key = derive_wrap_key(
        master_password,
        &salt,
        encrypted.kdf_memory_kib,
        encrypted.kdf_iterations,
        encrypted.kdf_parallelism,
    )?;
    let aad = build_aad(account_id, base_url, email);
    let cipher = XChaCha20Poly1305::new((&key).into());
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: ciphertext.as_slice(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "failed to decrypt persisted session with provided master password"
                .to_string(),
        })?;
    let refresh_token =
        String::from_utf8(plaintext).map_err(|error| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("decrypted session is not utf-8: {error}"),
        })?;
    let runtime = SessionWrapRuntime {
        key,
        kdf_memory_kib: encrypted.kdf_memory_kib,
        kdf_iterations: encrypted.kdf_iterations,
        kdf_parallelism: encrypted.kdf_parallelism,
        salt_b64: encrypted.salt_b64.clone(),
    };
    Ok((refresh_token, runtime))
}

pub fn encrypt_refresh_token_with_runtime(
    runtime: &SessionWrapRuntime,
    account_id: &str,
    base_url: &str,
    email: &str,
    refresh_token: &str,
) -> AppResult<PersistedEncryptedSession> {
    if refresh_token.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "refresh_token cannot be empty".to_string(),
        });
    }

    let mut nonce = [0u8; WRAP_NONCE_LEN];
    rand::rng().fill(&mut nonce);
    let aad = build_aad(account_id, base_url, email);
    let cipher = XChaCha20Poly1305::new((&runtime.key).into());
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: refresh_token.as_bytes(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "failed to encrypt persisted refresh token".to_string(),
        })?;

    Ok(PersistedEncryptedSession {
        algorithm: String::from(WRAP_ALGORITHM),
        kdf: String::from(WRAP_KDF),
        kdf_memory_kib: runtime.kdf_memory_kib,
        kdf_iterations: runtime.kdf_iterations,
        kdf_parallelism: runtime.kdf_parallelism,
        salt_b64: runtime.salt_b64.clone(),
        nonce_b64: STANDARD_NO_PAD.encode(nonce),
        ciphertext_b64: STANDARD_NO_PAD.encode(ciphertext),
    })
}

fn decode_fixed_len(value: &str, expected_len: usize, field_name: &str) -> AppResult<Vec<u8>> {
    let decoded =
        STANDARD_NO_PAD
            .decode(value.as_bytes())
            .map_err(|_| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("{field_name} is not valid base64"),
            })?;
    if decoded.len() != expected_len {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!(
                "{field_name} must decode to {expected_len} bytes, got {}",
                decoded.len()
            ),
        });
    }
    Ok(decoded)
}

fn derive_wrap_key(
    master_password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> AppResult<[u8; WRAP_KEY_LEN]> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(WRAP_KEY_LEN))
        .map_err(|error| {
            AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!(
                    "invalid session wrap argon2 params memory_kib={memory_kib} iterations={iterations} parallelism={parallelism}: {error}"
                ),
            }
        })?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; WRAP_KEY_LEN];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|error| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("failed to derive session wrap key with argon2id: {error}"),
        })?;
    Ok(key)
}

fn build_aad(account_id: &str, base_url: &str, email: &str) -> String {
    format!(
        "vanguard:auth-state:v{AUTH_STATE_VERSION}:{account_id}:{}:{}",
        base_url.trim().to_lowercase(),
        email.trim().to_lowercase()
    )
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::InternalUnexpected {
            message: format!("system clock before unix epoch: {error}"),
        })?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}

#[cfg(test)]
mod tests {
    use super::{decrypt_refresh_token, encrypt_refresh_token, encrypt_refresh_token_with_runtime};

    #[test]
    fn encrypt_then_decrypt_round_trip() {
        let account_id = "https://vault.example::user";
        let base_url = "https://vault.example";
        let email = "user@example.com";
        let refresh_token = "refresh-token-value";
        let (encrypted, _) = encrypt_refresh_token(
            "master-password",
            account_id,
            base_url,
            email,
            refresh_token,
        )
        .expect("encrypt");

        let (decrypted, _) =
            decrypt_refresh_token("master-password", account_id, base_url, email, &encrypted)
                .expect("decrypt");

        assert_eq!(decrypted, refresh_token);
    }

    #[test]
    fn re_encrypt_with_runtime_keeps_decryptable() {
        let account_id = "https://vault.example::user";
        let base_url = "https://vault.example";
        let email = "user@example.com";
        let (encrypted, runtime) =
            encrypt_refresh_token("master-password", account_id, base_url, email, "token-old")
                .expect("encrypt");

        let _ = decrypt_refresh_token("master-password", account_id, base_url, email, &encrypted)
            .expect("decrypt original");

        let rotated = encrypt_refresh_token_with_runtime(
            &runtime,
            account_id,
            base_url,
            email,
            "token-rotated",
        )
        .expect("re-encrypt");
        let (decrypted_rotated, _) =
            decrypt_refresh_token("master-password", account_id, base_url, email, &rotated)
                .expect("decrypt rotated");

        assert_eq!(decrypted_rotated, "token-rotated");
    }

    #[test]
    fn decrypt_fails_when_aad_context_differs() {
        let account_id = "https://vault.example::user";
        let base_url = "https://vault.example";
        let email = "user@example.com";
        let (encrypted, _) = encrypt_refresh_token(
            "master-password",
            account_id,
            base_url,
            email,
            "refresh-token",
        )
        .expect("encrypt");

        let error = decrypt_refresh_token(
            "master-password",
            account_id,
            base_url,
            "other@example.com",
            &encrypted,
        )
        .expect_err("decrypt should fail");
        assert_eq!(error.code(), "VALIDATION_FIELD_ERROR");
    }
}
