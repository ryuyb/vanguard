use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use hkdf::Hkdf;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngExt;
use sha2::Sha256;

use crate::application::dto::sync::{SyncSend, SyncSendFile, SyncSendText};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Generates a 16-byte random send key.
pub fn generate_send_key() -> Vec<u8> {
    let mut key = vec![0u8; 16];
    rand::rng().fill(key.as_mut_slice());
    key
}

/// Derives a 64-byte shareable key from the send key via HKDF-SHA256.
/// Returns (enc_key[32], mac_key[32]).
pub fn derive_send_shareable_key(send_key: &[u8]) -> AppResult<VaultUserKeyMaterial> {
    let hk = Hkdf::<Sha256>::new(Some(b"send"), send_key);
    let mut okm = [0u8; 64];
    hk.expand(b"send", &mut okm).map_err(|_| AppError::InternalUnexpected {
        message: "HKDF expand failed for send key".to_string(),
    })?;
    Ok(VaultUserKeyMaterial {
        enc_key: okm[..32].to_vec(),
        mac_key: Some(okm[32..].to_vec()),
        refresh_token: None,
    })
}

/// Encrypts the send key with the user key, returning a CipherString.
pub fn encrypt_send_key(send_key: &[u8], user_key: &VaultUserKeyMaterial) -> AppResult<String> {
    vault_crypto::encrypt_cipher_bytes(send_key, user_key).map_err(|e| {
        AppError::InternalUnexpected {
            message: format!("failed to encrypt send key: {}", e.message()),
        }
    })
}

/// Decrypts the send key CipherString with the user key.
pub fn decrypt_send_key(encrypted_key: &str, user_key: &VaultUserKeyMaterial) -> AppResult<Vec<u8>> {
    vault_crypto::decrypt_cipher_bytes(encrypted_key, user_key).map_err(|e| {
        AppError::InternalUnexpected {
            message: format!("failed to decrypt send key: {}", e.message()),
        }
    })
}

/// Encrypts name, notes, text.text, file.file_name using the send's shareable key.
/// The `key` field must already be set (encrypted send key).
pub fn encrypt_send(send: &SyncSend, user_key: &VaultUserKeyMaterial) -> AppResult<SyncSend> {
    // Derive shareable key from the raw send key bytes.
    // For a new send, the caller must have set send.key to the base64-encoded raw send key
    // before calling this; we decrypt it first to get the raw bytes.
    let shareable_key = resolve_shareable_key(send, user_key)?;

    Ok(SyncSend {
        name: encrypt_field(&send.name, &shareable_key, "send.name")?,
        notes: encrypt_field(&send.notes, &shareable_key, "send.notes")?,
        text: send
            .text
            .as_ref()
            .map(|t| -> AppResult<SyncSendText> {
                Ok(SyncSendText {
                    text: encrypt_field(&t.text, &shareable_key, "send.text.text")?,
                    hidden: t.hidden,
                })
            })
            .transpose()?,
        file: send
            .file
            .as_ref()
            .map(|f| -> AppResult<SyncSendFile> {
                Ok(SyncSendFile {
                    file_name: encrypt_field(&f.file_name, &shareable_key, "send.file.file_name")?,
                    ..f.clone()
                })
            })
            .transpose()?,
        ..send.clone()
    })
}

/// Decrypts name, notes, text.text, file.file_name using the send's shareable key.
pub fn decrypt_send(send: &SyncSend, user_key: &VaultUserKeyMaterial) -> AppResult<SyncSend> {
    let Some(encrypted_key) = &send.key else {
        return Ok(send.clone());
    };

    let send_key_bytes = decrypt_send_key(encrypted_key, user_key)?;
    let shareable_key = derive_send_shareable_key(&send_key_bytes)?;

    Ok(SyncSend {
        name: decrypt_field(&send.name, &shareable_key, "send.name")?,
        notes: decrypt_field(&send.notes, &shareable_key, "send.notes")?,
        text: send
            .text
            .as_ref()
            .map(|t| -> AppResult<SyncSendText> {
                Ok(SyncSendText {
                    text: decrypt_field(&t.text, &shareable_key, "send.text.text")?,
                    hidden: t.hidden,
                })
            })
            .transpose()?,
        file: send
            .file
            .as_ref()
            .map(|f| -> AppResult<SyncSendFile> {
                Ok(SyncSendFile {
                    file_name: decrypt_field(&f.file_name, &shareable_key, "send.file.file_name")?,
                    ..f.clone()
                })
            })
            .transpose()?,
        ..send.clone()
    })
}

/// PBKDF2-SHA256(password, send_key, 100_000) → base64.
pub fn hash_send_password(password: &str, send_key: &[u8]) -> String {
    let mut output = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), send_key, 100_000, &mut output)
        .expect("PBKDF2 should not fail with valid parameters");
    STANDARD.encode(output)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn resolve_shareable_key(
    send: &SyncSend,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<VaultUserKeyMaterial> {
    match &send.key {
        Some(k) if vault_crypto::looks_like_cipher_string(k) => {
            // Already encrypted with user key — decrypt to get raw bytes
            let raw = decrypt_send_key(k, user_key)?;
            derive_send_shareable_key(&raw)
        }
        Some(k) => {
            // Raw base64 send key (new send, not yet encrypted)
            let raw = vault_crypto::decode_base64_flexible(k, "send.key")?;
            derive_send_shareable_key(&raw)
        }
        None => Err(AppError::ValidationFieldError {
            field: "key".to_string(),
            message: "send key is required for encryption".to_string(),
        }),
    }
}

fn encrypt_field(
    value: &Option<String>,
    key: &VaultUserKeyMaterial,
    field: &str,
) -> AppResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(v) if v.trim().is_empty() => Ok(Some(v.clone())),
        Some(v) if vault_crypto::looks_like_cipher_string(v) => Ok(Some(v.clone())),
        Some(v) => vault_crypto::encrypt_cipher_string(v, key)
            .map(Some)
            .map_err(|e| AppError::ValidationFieldError {
                field: field.to_string(),
                message: format!("failed to encrypt {field}: {}", e.message()),
            }),
    }
}

fn decrypt_field(
    value: &Option<String>,
    key: &VaultUserKeyMaterial,
    field: &str,
) -> AppResult<Option<String>> {
    vault_crypto::decrypt_optional_field(value.clone(), key, field)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user_key() -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
            refresh_token: None,
        }
    }

    fn make_send_with_raw_key(send_key: &[u8]) -> SyncSend {
        SyncSend {
            id: "send-1".to_string(),
            r#type: Some(0),
            name: Some("My Send".to_string()),
            notes: Some("Some notes".to_string()),
            key: Some(STANDARD.encode(send_key)),
            text: Some(SyncSendText {
                text: Some("secret text".to_string()),
                hidden: Some(false),
            }),
            file: None,
            revision_date: None,
            deletion_date: None,
            object: None,
            access_id: None,
            password: None,
            max_access_count: None,
            access_count: None,
            disabled: None,
            hide_email: None,
            expiration_date: None,
            emails: None,
            auth_type: None,
        }
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let user_key = test_user_key();
        let send_key = generate_send_key();
        let send = make_send_with_raw_key(&send_key);

        let encrypted = encrypt_send(&send, &user_key).expect("encrypt");

        // Fields should be encrypted
        assert!(vault_crypto::looks_like_cipher_string(
            encrypted.name.as_ref().unwrap()
        ));
        assert!(vault_crypto::looks_like_cipher_string(
            encrypted.notes.as_ref().unwrap()
        ));
        assert!(vault_crypto::looks_like_cipher_string(
            encrypted.text.as_ref().unwrap().text.as_ref().unwrap()
        ));

        // Now encrypt the send key and set it so decrypt_send can work
        let encrypted_key = encrypt_send_key(&send_key, &user_key).expect("encrypt key");
        let send_with_enc_key = SyncSend {
            key: Some(encrypted_key),
            ..encrypted
        };

        let decrypted = decrypt_send(&send_with_enc_key, &user_key).expect("decrypt");
        assert_eq!(decrypted.name.as_deref(), Some("My Send"));
        assert_eq!(decrypted.notes.as_deref(), Some("Some notes"));
        assert_eq!(
            decrypted.text.as_ref().unwrap().text.as_deref(),
            Some("secret text")
        );
    }

    #[test]
    fn hash_send_password_is_base64() {
        let send_key = generate_send_key();
        let hash = hash_send_password("my-password", &send_key);
        // Should be valid base64 and 32 bytes → 44 chars with padding
        let decoded = STANDARD.decode(&hash).expect("valid base64");
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn hash_send_password_is_deterministic() {
        let send_key = vec![42u8; 16];
        let h1 = hash_send_password("pass", &send_key);
        let h2 = hash_send_password("pass", &send_key);
        assert_eq!(h1, h2);
    }

    #[test]
    fn decrypt_send_without_key_returns_clone() {
        let user_key = test_user_key();
        let send = SyncSend {
            id: "s".to_string(),
            key: None,
            name: Some("plain".to_string()),
            r#type: None,
            notes: None,
            text: None,
            file: None,
            revision_date: None,
            deletion_date: None,
            object: None,
            access_id: None,
            password: None,
            max_access_count: None,
            access_count: None,
            disabled: None,
            hide_email: None,
            expiration_date: None,
            emails: None,
            auth_type: None,
        };
        let result = decrypt_send(&send, &user_key).expect("no-op decrypt");
        assert_eq!(result.name.as_deref(), Some("plain"));
    }
}
