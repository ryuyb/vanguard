use std::sync::Arc;

use crate::application::dto::sync::SyncCipher;
use crate::application::dto::vault::{GetCipherDetailQuery, VaultUserKeyMaterial};
use crate::application::services::sync_service::SyncService;
use crate::application::vault_crypto;
use crate::domain::cipher::{Cipher, Decrypted, Encrypted};
use crate::domain::crypto::Decryptable;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct GetCipherDetailUseCase {
    sync_service: Arc<SyncService>,
}

impl GetCipherDetailUseCase {
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self { sync_service }
    }

    pub async fn execute(&self, query: GetCipherDetailQuery) -> AppResult<Cipher<Decrypted>> {
        require_non_empty(&query.account_id, "account_id")?;
        require_non_empty(&query.cipher_id, "cipher_id")?;
        vault_crypto::validate_key_lengths(
            &query.user_key.enc_key,
            query.user_key.mac_key.as_deref(),
        )?;

        let cipher = self
            .sync_service
            .get_live_cipher(query.account_id.clone(), query.cipher_id.clone())
            .await?
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("cipher not found: {}", query.cipher_id),
            })?;

        decrypt_cipher_detail(cipher, &query.user_key)
    }
}

fn decrypt_cipher_detail(
    cipher: SyncCipher,
    user_key: &VaultUserKeyMaterial,
) -> Result<Cipher<Decrypted>, AppError> {
    // Convert SyncCipher to Cipher<Encrypted>
    let encrypted: Cipher<Encrypted> = cipher.into();

    // Resolve the decryption key (cipher-specific key or user key)
    // cipher.key is a plain Option<String> field
    let cipher_key_str = encrypted.key.as_deref();

    let detail_key = resolve_cipher_decryption_key(cipher_key_str, user_key)?;

    // Decrypt using the new type-state pattern
    encrypted.decrypt(&detail_key, "cipher")
}

fn resolve_cipher_decryption_key(
    cipher_key: Option<&str>,
    user_key: &VaultUserKeyMaterial,
) -> Result<VaultUserKeyMaterial, AppError> {
    let Some(raw) = cipher_key else {
        return Ok(user_key.clone());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(user_key.clone());
    }

    if vault_crypto::looks_like_cipher_string(trimmed) {
        let decrypted = vault_crypto::decrypt_cipher_bytes(trimmed, user_key).map_err(|error| {
            AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("failed to decrypt field `cipher.key`: {}", error.message()),
            }
        })?;
        return vault_crypto::parse_user_key_material(&decrypted).map_err(|error| {
            AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!(
                    "failed to parse decrypted field `cipher.key`: {}",
                    error.message()
                ),
            }
        });
    }

    vault_crypto::parse_user_key(trimmed).map_err(|error| AppError::ValidationFieldError {
        field: "unknown".to_string(),
        message: format!("failed to parse field `cipher.key`: {}", error.message()),
    })
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("{field} cannot be empty"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::sync::{
        SyncCipherLogin, SyncCipherLoginUri, SyncCipherPasswordHistory,
    };
    use aes::Aes256;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;
    type HmacSha256 = Hmac<Sha256>;

    #[test]
    fn decrypt_type2_cipher_string_roundtrip() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
            refresh_token: None,
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let plaintext = vault_crypto::decrypt_cipher_string(&cipher, &key).expect("decrypt type2");
        assert_eq!(plaintext, "hello-vault");
    }

    #[test]
    fn decrypt_type2_without_mac_key_is_rejected() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: None,
            refresh_token: None,
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let error = vault_crypto::decrypt_cipher_string(&cipher, &key).expect_err("must fail");
        assert_eq!(error.code(), "VALIDATION_FIELD_ERROR");
    }

    #[test]
    fn decrypt_cipher_detail_decrypts_whitelisted_fields_only() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
            refresh_token: None,
        };
        let encrypted_name = encrypt_type2("demo", &enc_key, &mac_key);
        let encrypted_password = encrypt_type2("history-pass", &enc_key, &mac_key);
        let encrypted_login_uri = encrypt_type2("https://example.com/login", &enc_key, &mac_key);
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(encrypted_name),
            notes: Some(String::from("note")),
            key: None,
            favorite: Some(false),
            edit: Some(true),
            view_password: Some(true),
            organization_use_totp: Some(false),
            creation_date: Some(String::from("2026-03-01T00:00:00Z")),
            revision_date: Some(String::from("2026-03-01T00:00:00Z")),
            deleted_date: None,
            archived_date: None,
            reprompt: Some(0),
            permissions: None,
            object: Some(String::from("cipher")),
            fields: Vec::new(),
            password_history: vec![SyncCipherPasswordHistory {
                password: Some(encrypted_password),
                last_used_date: Some(String::from("2026-03-01T00:00:00Z")),
            }],
            collection_ids: Vec::new(),
            data: None,
            login: Some(SyncCipherLogin {
                uri: None,
                uris: vec![SyncCipherLoginUri {
                    uri: Some(encrypted_login_uri),
                    r#match: Some(0),
                    uri_checksum: Some(String::from("2.not-a-cipher|string|shape")),
                }],
                username: None,
                password: None,
                password_revision_date: None,
                totp: None,
                autofill_on_page_load: Some(false),
                fido2_credentials: Vec::new(),
            }),
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let detail = decrypt_cipher_detail(cipher, &user_key).expect("detail deserialize");
        assert_eq!(detail.id, "cipher-1");
        assert_eq!(detail.name.as_ref(), Some(&"demo".to_string()));
        assert_eq!(detail.key, None);
        assert_eq!(detail.password_history.len(), 1);
        assert_eq!(
            detail.password_history[0].password.as_ref(),
            Some(&"history-pass".to_string())
        );
        assert_eq!(
            detail.password_history[0].last_used_date,
            Some("2026-03-01T00:00:00Z".to_string())
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0].uri.as_ref(),
            Some(&"https://example.com/login".to_string())
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0].uri_checksum,
            Some("2.not-a-cipher|string|shape".to_string())
        );
    }

    #[test]
    fn decrypt_cipher_detail_uses_cipher_key_for_field_decryption() {
        let user_enc_key = [1u8; 32];
        let user_mac_key = [2u8; 32];
        let user_key = VaultUserKeyMaterial {
            enc_key: user_enc_key.to_vec(),
            mac_key: Some(user_mac_key.to_vec()),
            refresh_token: None,
        };

        let cipher_enc_key = [3u8; 32];
        let cipher_mac_key = [4u8; 32];
        let mut cipher_key_material = Vec::with_capacity(64);
        cipher_key_material.extend_from_slice(&cipher_enc_key);
        cipher_key_material.extend_from_slice(&cipher_mac_key);
        let plain_cipher_key = STANDARD.encode(&cipher_key_material);

        let encrypted_cipher_key = encrypt_type2(&plain_cipher_key, &user_enc_key, &user_mac_key);
        let encrypted_name = encrypt_type2("cipher-name", &cipher_enc_key, &cipher_mac_key);
        let encrypted_password = encrypt_type2("cipher-pass", &cipher_enc_key, &cipher_mac_key);

        let cipher = SyncCipher {
            id: String::from("cipher-key-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(encrypted_name),
            notes: None,
            key: Some(encrypted_cipher_key.clone()),
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: Vec::new(),
            password_history: vec![SyncCipherPasswordHistory {
                password: Some(encrypted_password),
                last_used_date: None,
            }],
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let detail = decrypt_cipher_detail(cipher, &user_key).expect("detail decrypt");
        assert_eq!(detail.name.as_ref(), Some(&"cipher-name".to_string()));
        assert_eq!(
            detail.password_history[0].password.as_ref(),
            Some(&"cipher-pass".to_string())
        );
        assert_eq!(detail.key.as_deref(), Some(encrypted_cipher_key.as_str()));
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_on_whitelisted_field() {
        let user_key = VaultUserKeyMaterial {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
            refresh_token: None,
        };
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("2.not-base64|still-not-base64|bad-mac")),
            notes: None,
            key: None,
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let error = decrypt_cipher_detail(cipher, &user_key)
            .expect_err("invalid encrypted field must fail");
        assert_eq!(error.code(), "VALIDATION_FIELD_ERROR");
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_key() {
        let user_key = VaultUserKeyMaterial {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
            refresh_token: None,
        };
        let cipher = SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("plain-name")),
            notes: None,
            key: Some(String::from("2.not-base64|still-not-base64|bad-mac")),
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        let error =
            decrypt_cipher_detail(cipher, &user_key).expect_err("invalid cipher key must fail");
        assert_eq!(error.code(), "VALIDATION_FIELD_ERROR");
    }

    fn encrypt_type2(plaintext: &str, enc_key: &[u8; 32], mac_key: &[u8; 32]) -> String {
        encrypt_type2_bytes(plaintext.as_bytes(), enc_key, mac_key)
    }

    fn encrypt_type2_bytes(plaintext: &[u8], enc_key: &[u8], mac_key: &[u8]) -> String {
        let iv = [9u8; 16];
        let mut buffer = plaintext.to_vec();
        let message_len = buffer.len();
        buffer.resize(message_len + 16, 0);

        let ciphertext = Aes256CbcEncryptor::new_from_slices(enc_key, &iv)
            .expect("build encryptor")
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .expect("encrypt")
            .to_vec();

        let mut mac = HmacSha256::new_from_slice(mac_key).expect("build hmac");
        mac.update(&iv);
        mac.update(&ciphertext);
        let mac = mac.finalize().into_bytes();

        format!(
            "2.{}|{}|{}",
            STANDARD.encode(iv),
            STANDARD.encode(ciphertext),
            STANDARD.encode(mac)
        )
    }

    #[test]
    fn decrypt_cipher_and_serialize_to_dto_no_stack_overflow() {
        // This test verifies that serializing Cipher<Decrypted> through VaultCipherDetailDto
        // does not cause stack overflow due to infinite recursion

        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
            refresh_token: None,
        };
        let encrypted_name = encrypt_type2("Test Cipher", &enc_key, &mac_key);

        let cipher = SyncCipher {
            id: String::from("test-serialization"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(encrypted_name),
            notes: None,
            key: None,
            favorite: Some(true),
            edit: Some(true),
            view_password: None,
            organization_use_totp: None,
            creation_date: Some(String::from("2026-03-01T00:00:00Z")),
            revision_date: Some(String::from("2026-03-01T00:00:00Z")),
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: Some(String::from("cipher")),
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        };

        // Decrypt the cipher
        let decrypted = decrypt_cipher_detail(cipher, &user_key).expect("decrypt cipher");

        // Convert to DTO (this uses the From trait)
        let dto: crate::interfaces::tauri::dto::cipher::VaultCipherDetailDto = decrypted.into();

        // Serialize to JSON - this should NOT cause stack overflow
        let json_result = serde_json::to_string(&dto);
        assert!(
            json_result.is_ok(),
            "Serialization should succeed without stack overflow"
        );

        let json = json_result.unwrap();

        // Verify the JSON is valid and contains expected fields
        assert!(
            json.contains("\"id\":\"test-serialization\""),
            "JSON should contain id"
        );
        assert!(
            json.contains("\"name\":\"Test Cipher\""),
            "JSON should contain decrypted name"
        );
        assert!(
            json.contains("\"hasTotp\":false"),
            "JSON should contain hasTotp field"
        );

        // Verify it can be parsed back
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");
        assert_eq!(parsed["id"], "test-serialization");
        assert_eq!(parsed["name"], "Test Cipher");
        assert_eq!(parsed["hasTotp"], false);
    }
}
