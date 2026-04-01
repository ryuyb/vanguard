use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::vault_crypto;
use crate::support::error::AppError;

/// 解析 Cipher 的解密密钥
///
/// Cipher 可能有专用的解密密钥（cipher.key 字段），
/// 该字段可能为空、明文 base64 或加密的 CipherString。
///
/// # Arguments
/// * `cipher_key` - cipher.key 字段的原始值
/// * `user_key` - 用户的主密钥，用于解密 cipher.key（如果它是加密的）
///
/// # Returns
/// * 如果 cipher_key 为空，返回 user_key
/// * 如果 cipher_key 是明文 base64，解析后返回专用密钥
/// * 如果 cipher_key 是加密的 CipherString，先用 user_key 解密，再返回专用密钥
pub fn resolve_decryption_key(
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

    // 判断是否为加密的 CipherString 格式
    if vault_crypto::looks_like_cipher_string(trimmed) {
        let decrypted = vault_crypto::decrypt_cipher_bytes(trimmed, user_key).map_err(|error| {
            AppError::ValidationFieldError {
                field: "cipher.key".to_string(),
                message: format!("failed to decrypt cipher.key: {}", error.message()),
            }
        })?;
        return vault_crypto::parse_user_key_material(&decrypted).map_err(|error| {
            AppError::ValidationFieldError {
                field: "cipher.key".to_string(),
                message: format!("failed to parse decrypted cipher.key: {}", error.message()),
            }
        });
    }

    // 明文 base64 格式
    vault_crypto::parse_user_key(trimmed).map_err(|error| AppError::ValidationFieldError {
        field: "cipher.key".to_string(),
        message: format!("failed to parse cipher.key: {}", error.message()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_key(enc_key: [u8; 32], mac_key: [u8; 32]) -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
            refresh_token: None,
        }
    }

    #[test]
    fn none_key_returns_user_key() {
        let user_key = create_test_key([1u8; 32], [2u8; 32]);
        let result = resolve_decryption_key(None, &user_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().enc_key, user_key.enc_key);
    }

    #[test]
    fn empty_key_returns_user_key() {
        let user_key = create_test_key([1u8; 32], [2u8; 32]);
        let result = resolve_decryption_key(Some(""), &user_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().enc_key, user_key.enc_key);
    }

    #[test]
    fn whitespace_only_key_returns_user_key() {
        let user_key = create_test_key([1u8; 32], [2u8; 32]);
        let result = resolve_decryption_key(Some("   "), &user_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().enc_key, user_key.enc_key);
    }
}
