use aes::Aes256;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tauri::State;

use crate::application::dto::auth::PreloginQuery;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::infrastructure::vaultwarden::password_hash::derive_master_key;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::vault::{
    VaultCipherItemDto, VaultDecryptionStatusDto, VaultFolderItemDto, VaultLockRequestDto,
    VaultUnlockWithPasswordRequestDto, VaultUnlockWithUserKeyRequestDto, VaultViewDataRequestDto,
    VaultViewDataResponseDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;

type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 200;

fn log_command_error(command: &str, error: AppError) -> String {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::vault",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload.message
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock_with_user_key(
    state: State<'_, AppState>,
    request: VaultUnlockWithUserKeyRequestDto,
) -> Result<(), String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_unlock_with_user_key", error))?;
    let key = parse_user_key(&request.user_key)
        .map_err(|error| log_command_error("vault_unlock_with_user_key", error))?;

    state
        .set_vault_user_key(account_id.clone(), key)
        .map_err(|error| log_command_error("vault_unlock_with_user_key", error))?;

    log::info!(
        target: "vanguard::tauri::vault",
        "vault unlocked in memory account_id={}",
        account_id
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock_with_password(
    state: State<'_, AppState>,
    request: VaultUnlockWithPasswordRequestDto,
) -> Result<(), String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_unlock_with_password", error))?;
    let prelogin = state
        .auth_service()
        .prelogin(PreloginQuery {
            base_url: request.base_url,
            email: request.email.clone(),
        })
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

    let master_key = derive_master_key(
        &request.email,
        &request.password,
        prelogin.kdf,
        prelogin.kdf_iterations,
        prelogin.kdf_memory,
        prelogin.kdf_parallelism,
    )
    .map_err(|error| {
        log_command_error(
            "vault_unlock_with_password",
            AppError::validation(format!("failed to derive master key: {error}")),
        )
    })?;

    let encrypted_user_key = state
        .sync_service()
        .load_live_user_decryption(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))
        .and_then(|value| {
            extract_encrypted_user_key(value)
                .map_err(|error| log_command_error("vault_unlock_with_password", error))
        })?;

    let user_key = decrypt_user_key_with_master_key(&encrypted_user_key, &master_key)
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;
    state
        .set_vault_user_key(account_id.clone(), user_key)
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

    log::info!(
        target: "vanguard::tauri::vault",
        "vault unlocked with password in memory account_id={}",
        account_id
    );

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_lock(
    state: State<'_, AppState>,
    request: VaultLockRequestDto,
) -> Result<(), String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_lock", error))?;

    state
        .remove_vault_user_key(&account_id)
        .map_err(|error| log_command_error("vault_lock", error))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_view_data(
    state: State<'_, AppState>,
    request: VaultViewDataRequestDto,
) -> Result<VaultViewDataResponseDto, String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_get_view_data", error))?;

    let page = normalize_page(request.page);
    let page_size = normalize_page_size(request.page_size);
    let offset =
        (u64::from(page.saturating_sub(1)) * u64::from(page_size)).min(u64::from(u32::MAX)) as u32;

    let context = state
        .sync_service()
        .sync_status(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_get_view_data", error))?;
    let metrics = state
        .sync_service()
        .sync_metrics(account_id.clone())
        .map_err(|error| log_command_error("vault_get_view_data", error))?;

    let folders = state
        .sync_service()
        .list_live_folders(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_get_view_data", error))?;
    let ciphers = state
        .sync_service()
        .list_live_ciphers(account_id.clone(), offset, page_size)
        .await
        .map_err(|error| log_command_error("vault_get_view_data", error))?;
    let total_ciphers = state
        .sync_service()
        .count_live_ciphers(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_get_view_data", error))?;
    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("vault_get_view_data", error))?;

    let mut has_encrypted_fields = false;
    let mut has_undecrypted_fields = false;
    let folder_items = folders
        .into_iter()
        .map(|folder| {
            let value = maybe_decrypt_field(folder.name, user_key.as_ref());
            has_encrypted_fields |= value.is_encrypted;
            has_undecrypted_fields |= value.is_encrypted && !value.is_decrypted;
            VaultFolderItemDto {
                id: folder.id,
                name: value.plaintext,
                encrypted_name: value.encrypted,
                is_name_encrypted: value.is_encrypted,
            }
        })
        .collect();
    let cipher_items = ciphers
        .into_iter()
        .map(|cipher| {
            let value = maybe_decrypt_field(cipher.name, user_key.as_ref());
            has_encrypted_fields |= value.is_encrypted;
            has_undecrypted_fields |= value.is_encrypted && !value.is_decrypted;
            VaultCipherItemDto {
                id: cipher.id,
                folder_id: cipher.folder_id,
                organization_id: cipher.organization_id,
                r#type: cipher.r#type,
                name: value.plaintext,
                encrypted_name: value.encrypted,
                is_name_encrypted: value.is_encrypted,
                revision_date: cipher.revision_date,
                deleted_date: cipher.deleted_date,
                attachment_count: cipher.attachments.len().min(u32::MAX as usize) as u32,
            }
        })
        .collect();

    Ok(VaultViewDataResponseDto {
        account_id,
        sync_status: mapping::to_sync_status_response_dto(context, Some(metrics)),
        decryption_status: if has_encrypted_fields && has_undecrypted_fields {
            VaultDecryptionStatusDto::Locked
        } else {
            VaultDecryptionStatusDto::Unlocked
        },
        folders: folder_items,
        ciphers: cipher_items,
        total_ciphers,
        page,
        page_size,
    })
}

fn normalize_page(page: Option<u32>) -> u32 {
    page.unwrap_or(DEFAULT_PAGE).max(1)
}

fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE)
}

#[derive(Debug)]
struct DecryptedField {
    plaintext: Option<String>,
    encrypted: Option<String>,
    is_encrypted: bool,
    is_decrypted: bool,
}

fn maybe_decrypt_field(value: Option<String>, user_key: Option<&VaultUserKey>) -> DecryptedField {
    match value {
        None => DecryptedField {
            plaintext: None,
            encrypted: None,
            is_encrypted: false,
            is_decrypted: false,
        },
        Some(raw) => {
            if !looks_like_cipher_string(&raw) {
                return DecryptedField {
                    plaintext: Some(raw),
                    encrypted: None,
                    is_encrypted: false,
                    is_decrypted: false,
                };
            }

            let Some(key) = user_key else {
                return DecryptedField {
                    plaintext: None,
                    encrypted: Some(raw),
                    is_encrypted: true,
                    is_decrypted: false,
                };
            };

            match decrypt_cipher_string(&raw, key) {
                Ok(plaintext) => DecryptedField {
                    plaintext: Some(plaintext),
                    encrypted: Some(raw),
                    is_encrypted: true,
                    is_decrypted: true,
                },
                Err(error) => {
                    log::warn!(
                        target: "vanguard::tauri::vault",
                        "failed to decrypt field: [{}] {}",
                        error.code(),
                        error.log_message()
                    );
                    DecryptedField {
                        plaintext: None,
                        encrypted: Some(raw),
                        is_encrypted: true,
                        is_decrypted: false,
                    }
                }
            }
        }
    }
}

fn parse_user_key(raw: &str) -> Result<VaultUserKey, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("user_key cannot be empty"));
    }

    if let Some((enc, mac)) = trimmed.split_once('|') {
        let enc_key = decode_base64_flexible(enc.trim(), "user_key.enc_key")?;
        let mac_key = decode_base64_flexible(mac.trim(), "user_key.mac_key")?;
        validate_key_lengths(&enc_key, Some(&mac_key))?;
        return Ok(VaultUserKey {
            enc_key,
            mac_key: Some(mac_key),
        });
    }

    let raw_bytes = decode_base64_flexible(trimmed, "user_key")?;
    match raw_bytes.len() {
        32 => Ok(VaultUserKey {
            enc_key: raw_bytes,
            mac_key: None,
        }),
        64 => Ok(VaultUserKey {
            enc_key: raw_bytes[..32].to_vec(),
            mac_key: Some(raw_bytes[32..].to_vec()),
        }),
        len => Err(AppError::validation(format!(
            "user_key length must be 32 or 64 bytes after base64 decode, got {len}"
        ))),
    }
}

fn extract_encrypted_user_key(
    value: Option<crate::application::dto::sync::SyncUserDecryption>,
) -> Result<String, AppError> {
    let value = value.ok_or_else(|| {
        AppError::validation("missing local user_decryption data; run vault sync first")
    })?;
    let unlock = value.master_password_unlock.ok_or_else(|| {
        AppError::validation("missing master_password_unlock in local vault metadata")
    })?;

    unlock
        .master_key_wrapped_user_key
        .or(unlock.master_key_encrypted_user_key)
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .ok_or_else(|| {
            AppError::validation("encrypted user key is missing in local vault metadata")
        })
}

fn decrypt_user_key_with_master_key(
    encrypted_user_key: &str,
    master_key: &[u8],
) -> Result<VaultUserKey, AppError> {
    let trimmed = encrypted_user_key.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("encrypted_user_key cannot be empty"));
    }

    if !looks_like_cipher_string(trimmed) {
        return parse_user_key(trimmed);
    }

    for candidate in candidate_keys_from_master_key(master_key) {
        if let Ok(plaintext_user_key) = decrypt_cipher_string(trimmed, &candidate) {
            if let Ok(user_key) = parse_user_key(&plaintext_user_key) {
                return Ok(user_key);
            }
        }
    }

    Err(AppError::validation(
        "unable to unlock encrypted_user_key with provided password",
    ))
}

fn candidate_keys_from_master_key(master_key: &[u8]) -> Vec<VaultUserKey> {
    let mut candidates = Vec::new();

    if master_key.len() == 32 {
        candidates.push(VaultUserKey {
            enc_key: master_key.to_vec(),
            mac_key: None,
        });
        candidates.push(VaultUserKey {
            enc_key: master_key.to_vec(),
            mac_key: Some(hmac_derive(master_key, b"mac")),
        });
        candidates.push(VaultUserKey {
            enc_key: hkdf_expand(master_key, b"enc", 32),
            mac_key: Some(hkdf_expand(master_key, b"mac", 32)),
        });
    } else if master_key.len() == 64 {
        candidates.push(VaultUserKey {
            enc_key: master_key[..32].to_vec(),
            mac_key: Some(master_key[32..].to_vec()),
        });
    }

    candidates
}

fn hmac_derive(key: &[u8], label: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key must be valid");
    mac.update(label);
    mac.finalize().into_bytes().to_vec()
}

fn hkdf_expand(ikm: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let salt = [0u8; 32];
    let mut extract = HmacSha256::new_from_slice(&salt).expect("hkdf salt");
    extract.update(ikm);
    let prk = extract.finalize().into_bytes();

    let mut okm = Vec::with_capacity(len);
    let mut previous = Vec::new();
    let mut counter: u8 = 1;

    while okm.len() < len {
        let mut expand = HmacSha256::new_from_slice(&prk).expect("hkdf prk");
        if !previous.is_empty() {
            expand.update(&previous);
        }
        expand.update(info);
        expand.update(&[counter]);
        previous = expand.finalize().into_bytes().to_vec();
        okm.extend_from_slice(&previous);
        counter = counter.saturating_add(1);
        if counter == 0 {
            break;
        }
    }

    okm.truncate(len);
    okm
}

fn validate_key_lengths(enc_key: &[u8], mac_key: Option<&[u8]>) -> Result<(), AppError> {
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

fn decrypt_cipher_string(value: &str, key: &VaultUserKey) -> Result<String, AppError> {
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

fn decrypt_aes_cbc(iv: &[u8], ciphertext: &[u8], enc_key: &[u8]) -> Result<String, AppError> {
    let mut buffer = ciphertext.to_vec();
    let decryptor = Aes256CbcDecryptor::new_from_slices(enc_key, iv)
        .map_err(|error| AppError::validation(format!("invalid aes key/iv: {error}")))?;
    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| AppError::validation("ciphertext decryption failed"))?;
    String::from_utf8(plaintext.to_vec())
        .map_err(|error| AppError::validation(format!("plaintext is not utf-8: {error}")))
}

fn decode_base64_flexible(value: &str, label: &str) -> Result<Vec<u8>, AppError> {
    STANDARD
        .decode(value)
        .or_else(|_| STANDARD_NO_PAD.decode(value))
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .map_err(|_| AppError::validation(format!("{label} is not valid base64")))
}

fn looks_like_cipher_string(value: &str) -> bool {
    let trimmed = value.trim();
    let Some((enc_type, payload)) = trimmed.split_once('.') else {
        return false;
    };
    !enc_type.is_empty()
        && enc_type.chars().all(|char| char.is_ascii_digit())
        && !payload.is_empty()
        && payload.contains('|')
}

#[cfg(test)]
mod tests {
    use super::*;
    use cbc::cipher::{BlockEncryptMut, KeyIvInit};

    type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;

    #[test]
    fn parse_user_key_supports_64_byte_material() {
        let mut key_material = vec![0u8; 64];
        key_material
            .iter_mut()
            .enumerate()
            .for_each(|(idx, value)| *value = idx as u8);
        let encoded = STANDARD.encode(&key_material);

        let parsed = parse_user_key(&encoded).expect("parse user key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn decrypt_type2_cipher_string_roundtrip() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKey {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let plaintext = decrypt_cipher_string(&cipher, &key).expect("decrypt type2");
        assert_eq!(plaintext, "hello-vault");
    }

    #[test]
    fn decrypt_type2_without_mac_key_is_rejected() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let key = VaultUserKey {
            enc_key: enc_key.to_vec(),
            mac_key: None,
        };

        let cipher = encrypt_type2("hello-vault", &enc_key, &mac_key);
        let error = decrypt_cipher_string(&cipher, &key).expect_err("must fail");
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn decrypt_user_key_with_master_key_supports_hmac_derived_mac_key() {
        let master_key = [7u8; 32];
        let enc_key = master_key;
        let mac_key_vec = hmac_derive(&master_key, b"mac");
        let mac_key: [u8; 32] = mac_key_vec
            .try_into()
            .expect("hmac output should be 32 bytes");
        let plain_user_key = STANDARD.encode([3u8; 64]);
        let encrypted_user_key = encrypt_type2(&plain_user_key, &enc_key, &mac_key);

        let parsed = decrypt_user_key_with_master_key(&encrypted_user_key, &master_key)
            .expect("unlock with password candidate key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn extract_encrypted_user_key_prefers_wrapped_variant() {
        let extracted =
            extract_encrypted_user_key(Some(crate::application::dto::sync::SyncUserDecryption {
                master_password_unlock: Some(
                    crate::application::dto::sync::SyncMasterPasswordUnlock {
                        kdf: None,
                        master_key_encrypted_user_key: Some(String::from(
                            "2.encrypted|payload|mac",
                        )),
                        master_key_wrapped_user_key: Some(String::from("2.wrapped|payload|mac")),
                        salt: None,
                    },
                ),
            }))
            .expect("extract wrapped");

        assert_eq!(extracted, "2.wrapped|payload|mac");
    }

    #[test]
    fn extract_encrypted_user_key_fallbacks_to_encrypted_variant() {
        let extracted =
            extract_encrypted_user_key(Some(crate::application::dto::sync::SyncUserDecryption {
                master_password_unlock: Some(
                    crate::application::dto::sync::SyncMasterPasswordUnlock {
                        kdf: None,
                        master_key_encrypted_user_key: Some(String::from(
                            "2.encrypted|payload|mac",
                        )),
                        master_key_wrapped_user_key: None,
                        salt: None,
                    },
                ),
            }))
            .expect("extract encrypted");

        assert_eq!(extracted, "2.encrypted|payload|mac");
    }

    fn encrypt_type2(plaintext: &str, enc_key: &[u8; 32], mac_key: &[u8; 32]) -> String {
        let iv = [9u8; 16];
        let mut buffer = plaintext.as_bytes().to_vec();
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
}
