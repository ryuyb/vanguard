use aes::Aes256;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

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
                .map_err(|error| {
                    AppError::validation(format!(
                        "failed to decrypt field `{field_name}`: {}",
                        error.message()
                    ))
                })
        }
    }
}

pub fn parse_user_key(raw: &str) -> Result<VaultUserKeyMaterial, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("user_key cannot be empty"));
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
        len => Err(AppError::validation(format!(
            "user_key length must be 32 or 64 bytes after base64 decode, got {len}"
        ))),
    }
}

pub fn validate_key_lengths(enc_key: &[u8], mac_key: Option<&[u8]>) -> Result<(), AppError> {
    if enc_key.len() != 32 {
        return Err(AppError::validation(format!(
            "enc key must be 32 bytes, got {}",
            enc_key.len()
        )));
    }
    if let Some(mac_key) = mac_key {
        if mac_key.len() != 32 {
            return Err(AppError::validation(format!(
                "mac key must be 32 bytes, got {}",
                mac_key.len()
            )));
        }
    }
    Ok(())
}

pub fn decrypt_cipher_string(value: &str, key: &VaultUserKeyMaterial) -> Result<String, AppError> {
    let plaintext = decrypt_cipher_bytes(value, key)?;
    String::from_utf8(plaintext)
        .map_err(|error| AppError::validation(format!("plaintext is not utf-8: {error}")))
}

pub fn decrypt_cipher_bytes(value: &str, key: &VaultUserKeyMaterial) -> Result<Vec<u8>, AppError> {
    let trimmed = value.trim();
    let (enc_type, payload) = trimmed
        .split_once('.')
        .ok_or_else(|| AppError::validation("cipher string missing encryption type"))?;
    if enc_type.is_empty() || payload.is_empty() {
        return Err(AppError::validation("cipher string has empty segments"));
    }
    let enc_type = enc_type
        .parse::<u8>()
        .map_err(|_| AppError::validation("cipher string has invalid encryption type"))?;

    match enc_type {
        0 => {
            let parts = split_cipher_payload(payload, 2)?;
            decrypt_aes_cbc(
                &decode_base64_flexible(parts[0], "cipher.iv")?,
                &decode_base64_flexible(parts[1], "cipher.data")?,
                &key.enc_key,
            )
        }
        2 => {
            let parts = split_cipher_payload(payload, 3)?;
            let iv = decode_base64_flexible(parts[0], "cipher.iv")?;
            let ciphertext = decode_base64_flexible(parts[1], "cipher.data")?;
            let mac = decode_base64_flexible(parts[2], "cipher.mac")?;
            verify_mac(&iv, &ciphertext, &mac, key.mac_key.as_deref())?;
            decrypt_aes_cbc(&iv, &ciphertext, &key.enc_key)
        }
        _ => Err(AppError::validation(format!(
            "unsupported cipher string encryption type: {enc_type}"
        ))),
    }
}

pub fn decode_base64_flexible(value: &str, label: &str) -> Result<Vec<u8>, AppError> {
    STANDARD
        .decode(value)
        .or_else(|_| STANDARD_NO_PAD.decode(value))
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .map_err(|_| AppError::validation(format!("{label} is not valid base64")))
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

    let text = std::str::from_utf8(raw).map_err(|error| {
        AppError::validation(format!("user_key plaintext is not utf-8: {error}"))
    })?;
    parse_user_key(text)
}

fn split_cipher_payload(payload: &str, expected: usize) -> Result<Vec<&str>, AppError> {
    let parts: Vec<&str> = payload.split('|').collect();
    if parts.len() != expected || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(AppError::validation(
            "cipher string payload shape is invalid",
        ));
    }
    Ok(parts)
}

fn verify_mac(
    iv: &[u8],
    ciphertext: &[u8],
    mac: &[u8],
    mac_key: Option<&[u8]>,
) -> Result<(), AppError> {
    let mac_key =
        mac_key.ok_or_else(|| AppError::validation("mac key required for encryption type 2"))?;
    let mut signer = HmacSha256::new_from_slice(mac_key)
        .map_err(|error| AppError::validation(format!("invalid mac key: {error}")))?;
    signer.update(iv);
    signer.update(ciphertext);
    signer
        .verify_slice(mac)
        .map_err(|_| AppError::validation("cipher string mac verification failed"))?;
    Ok(())
}

fn decrypt_aes_cbc(iv: &[u8], ciphertext: &[u8], enc_key: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut buffer = ciphertext.to_vec();
    let decryptor = Aes256CbcDecryptor::new_from_slices(enc_key, iv)
        .map_err(|error| AppError::validation(format!("invalid aes key/iv: {error}")))?;
    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| AppError::validation("ciphertext decryption failed"))?;
    Ok(plaintext.to_vec())
}
