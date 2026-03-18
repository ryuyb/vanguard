use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;

use crate::application::crypto::encryption;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

pub fn decrypt_optional_field(
    value: Option<String>,
    user_key: &VaultUserKeyMaterial,
    field_name: &str,
) -> Result<Option<String>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => {
            if !looks_like_cipher_string(&raw) {
                return Ok(Some(raw));
            }

            decrypt_cipher_string(&raw, user_key)
                .map(Some)
                .map_err(|error| AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: format!(
                        "failed to decrypt field `{field_name}`: {}",
                        error.message()
                    ),
                })
        }
    }
}

pub fn parse_user_key(raw: &str) -> Result<VaultUserKeyMaterial, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "user_key cannot be empty".to_string(),
        });
    }

    if let Some((enc, mac)) = trimmed.split_once('|') {
        let enc_key = decode_base64_flexible(enc.trim(), "user_key.enc_key")?;
        let mac_key = decode_base64_flexible(mac.trim(), "user_key.mac_key")?;
        validate_key_lengths(&enc_key, Some(&mac_key))?;
        return Ok(VaultUserKeyMaterial {
            enc_key,
            mac_key: Some(mac_key),
        });
    }

    let raw_bytes = decode_base64_flexible(trimmed, "user_key")?;
    match raw_bytes.len() {
        32 => Ok(VaultUserKeyMaterial {
            enc_key: raw_bytes,
            mac_key: None,
        }),
        64 => Ok(VaultUserKeyMaterial {
            enc_key: raw_bytes[..32].to_vec(),
            mac_key: Some(raw_bytes[32..].to_vec()),
        }),
        len => Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!(
                "user_key length must be 32 or 64 bytes after base64 decode, got {len}"
            ),
        }),
    }
}

pub fn validate_key_lengths(enc_key: &[u8], mac_key: Option<&[u8]>) -> Result<(), AppError> {
    if enc_key.len() != 32 {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("enc key must be 32 bytes, got {}", enc_key.len()),
        });
    }
    if let Some(mac_key) = mac_key {
        if mac_key.len() != 32 {
            return Err(AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("mac key must be 32 bytes, got {}", mac_key.len()),
            });
        }
    }
    Ok(())
}

pub fn decrypt_cipher_string(value: &str, key: &VaultUserKeyMaterial) -> Result<String, AppError> {
    let plaintext = decrypt_cipher_bytes(value, key)?;
    String::from_utf8(plaintext).map_err(|error| AppError::ValidationFieldError {
        field: "unknown".to_string(),
        message: format!("plaintext is not utf-8: {error}"),
    })
}

pub fn decrypt_cipher_bytes(value: &str, key: &VaultUserKeyMaterial) -> Result<Vec<u8>, AppError> {
    let trimmed = value.trim();
    let (enc_type, payload) =
        trimmed
            .split_once('.')
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "cipher string missing encryption type".to_string(),
            })?;
    if enc_type.is_empty() || payload.is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "cipher string has empty segments".to_string(),
        });
    }
    let enc_type = enc_type
        .parse::<u8>()
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "cipher string has invalid encryption type".to_string(),
        })?;

    match enc_type {
        0 => {
            let parts = split_cipher_payload(payload, 2)?;
            let iv = decode_base64_flexible(parts[0], "cipher.iv")?;
            let ct = decode_base64_flexible(parts[1], "cipher.data")?;
            // Type 0: AES-CBC without MAC — use raw decrypt
            encryption::decrypt_aes_cbc_raw(&iv, &ct, &key.enc_key).map_err(wrap_validation_error)
        }
        2 => {
            let parts = split_cipher_payload(payload, 3)?;
            let iv = decode_base64_flexible(parts[0], "cipher.iv")?;
            let ciphertext = decode_base64_flexible(parts[1], "cipher.data")?;
            let mac = decode_base64_flexible(parts[2], "cipher.mac")?;
            let mac_key = key
                .mac_key
                .as_deref()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "mac key required for encryption type 2".to_string(),
                })?;
            encryption::verify_hmac(&iv, &ciphertext, &mac, mac_key)
                .map_err(wrap_validation_error)?;
            encryption::decrypt_aes_cbc_raw(&iv, &ciphertext, &key.enc_key)
                .map_err(wrap_validation_error)
        }
        _ => Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("unsupported cipher string encryption type: {enc_type}"),
        }),
    }
}

pub fn decode_base64_flexible(value: &str, label: &str) -> Result<Vec<u8>, AppError> {
    STANDARD
        .decode(value)
        .or_else(|_| STANDARD_NO_PAD.decode(value))
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("{label} is not valid base64"),
        })
}

pub fn looks_like_cipher_string(value: &str) -> bool {
    let trimmed = value.trim();
    let Some((enc_type, payload)) = trimmed.split_once('.') else {
        return false;
    };
    !enc_type.is_empty()
        && enc_type.chars().all(|char| char.is_ascii_digit())
        && !payload.is_empty()
        && payload.contains('|')
}

pub fn parse_user_key_material(raw: &[u8]) -> Result<VaultUserKeyMaterial, AppError> {
    match raw.len() {
        32 => {
            return Ok(VaultUserKeyMaterial {
                enc_key: raw.to_vec(),
                mac_key: None,
            });
        }
        64 => {
            return Ok(VaultUserKeyMaterial {
                enc_key: raw[..32].to_vec(),
                mac_key: Some(raw[32..].to_vec()),
            });
        }
        _ => {}
    }

    let text = std::str::from_utf8(raw).map_err(|error| AppError::ValidationFieldError {
        field: "unknown".to_string(),
        message: format!("user_key plaintext is not utf-8: {error}"),
    })?;
    parse_user_key(text)
}

/// Encrypts a string using AES-256-CBC with HMAC-SHA256 (encryption type 2)
/// Returns a CipherString in the format: "2.iv|ciphertext|mac"
pub fn encrypt_cipher_string(
    plaintext: &str,
    key: &VaultUserKeyMaterial,
) -> Result<String, AppError> {
    encrypt_cipher_bytes(plaintext.as_bytes(), key)
}

/// Encrypts bytes using AES-256-CBC with HMAC-SHA256 (encryption type 2)
/// Returns a CipherString in the format: "2.iv|ciphertext|mac"
pub fn encrypt_cipher_bytes(
    plaintext: &[u8],
    key: &VaultUserKeyMaterial,
) -> Result<String, AppError> {
    if key.mac_key.is_some() {
        // Type 2: AES-CBC + HMAC
        let (iv, ciphertext, mac) =
            encryption::encrypt_aes256_hmac(plaintext, key).map_err(wrap_validation_error)?;
        Ok(format!(
            "2.{}|{}|{}",
            STANDARD.encode(&iv),
            STANDARD.encode(&ciphertext),
            STANDARD.encode(&mac)
        ))
    } else {
        // Type 0: AES-CBC without MAC
        let (iv, ciphertext) = encryption::encrypt_aes_cbc_raw(plaintext, &key.enc_key)
            .map_err(wrap_validation_error)?;
        Ok(format!(
            "0.{}|{}",
            STANDARD.encode(&iv),
            STANDARD.encode(&ciphertext)
        ))
    }
}

fn split_cipher_payload(payload: &str, expected: usize) -> Result<Vec<&str>, AppError> {
    let parts: Vec<&str> = payload.split('|').collect();
    if parts.len() != expected || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "cipher string payload shape is invalid".to_string(),
        });
    }
    Ok(parts)
}

/// Maps crypto module errors to ValidationFieldError for backward compatibility.
fn wrap_validation_error(error: AppError) -> AppError {
    match error {
        AppError::CryptoInvalidKey => AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid encryption key".to_string(),
        },
        AppError::CryptoEncryptionFailed => AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "plaintext encryption failed".to_string(),
        },
        AppError::CryptoDecryptionFailed => AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "cipher string mac verification failed".to_string(),
        },
        other => other,
    }
}
