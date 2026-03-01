use aes::Aes256;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::application::dto::auth::PreloginQuery;
use crate::bootstrap::app_state::{AppState, PersistedAuthContext, VaultUserKey};
use crate::infrastructure::vaultwarden::password_hash::derive_master_key;
use crate::interfaces::tauri::dto::vault::{
    VaultCipherDetailDto, VaultCipherDetailRequestDto, VaultCipherDetailResponseDto,
    VaultCipherItemDto, VaultFolderItemDto, VaultLockRequestDto, VaultUnlockWithPasswordRequestDto,
    VaultViewDataRequestDto, VaultViewDataResponseDto,
};
use crate::interfaces::tauri::{mapping, session};
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;
use crate::support::result::AppResult;

type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 200;
const BITWARDEN_HKDF_SALT: &[u8] = b"bitwarden";

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
pub async fn vault_unlock_with_password(
    state: State<'_, AppState>,
    request: VaultUnlockWithPasswordRequestDto,
) -> Result<(), String> {
    let master_password = request.master_password.trim().to_string();
    if master_password.is_empty() {
        return Err(log_command_error(
            "vault_unlock_with_password",
            AppError::validation("master_password cannot be empty"),
        ));
    }
    let unlock_context = resolve_unlock_context(&state, &master_password)
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;
    let account_id = unlock_context.account_id.clone();

    let unlock_material = state
        .sync_service()
        .load_live_user_decryption(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))
        .and_then(|value| {
            extract_unlock_material(value)
                .map_err(|error| log_command_error("vault_unlock_with_password", error))
        })?;

    let master_keys = derive_master_key_candidates_for_unlock(
        &state,
        &unlock_context,
        &master_password,
        &unlock_material,
    )
    .await
    .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

    let user_key =
        decrypt_user_key_with_master_keys(&unlock_material.encrypted_user_keys, &master_keys)
            .map_err(|error| log_command_error("vault_unlock_with_password", error))?;
    state
        .set_vault_user_key(account_id.clone(), user_key)
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

    log::info!(
        target: "vanguard::tauri::vault",
        "vault unlocked with password in memory account_id={}",
        account_id
    );

    if state
        .auth_session()
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?
        .is_none()
    {
        if let Err(error) =
            session::restore_auth_session_with_master_password(&state, &master_password).await
        {
            log::warn!(
                target: "vanguard::tauri::vault",
                "vault unlock completed but auth session restore failed account_id={} status={} error_code={} message={}",
                unlock_context.account_id,
                error
                    .status()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error.log_message()
            );
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_lock(
    state: State<'_, AppState>,
    _request: VaultLockRequestDto,
) -> Result<(), String> {
    let account_id = state
        .active_account_id()
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
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("vault_get_view_data", error))?;

    let page = normalize_page(request.page);
    let page_size = normalize_page_size(request.page_size);
    let offset =
        (u64::from(page.saturating_sub(1)) * u64::from(page_size)).min(u64::from(u32::MAX)) as u32;
    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("vault_get_view_data", error))?
        .ok_or_else(|| {
            log_command_error(
                "vault_get_view_data",
                AppError::validation("vault is locked, please unlock with master password first"),
            )
        })?;

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

    let folder_items: Result<Vec<VaultFolderItemDto>, AppError> = folders
        .into_iter()
        .map(|folder| {
            Ok(VaultFolderItemDto {
                id: folder.id,
                name: decrypt_field_value(folder.name, &user_key, "folder.name")?,
            })
        })
        .collect();
    let folder_items =
        folder_items.map_err(|error| log_command_error("vault_get_view_data", error))?;
    let cipher_items: Result<Vec<VaultCipherItemDto>, AppError> = ciphers
        .into_iter()
        .map(|cipher| {
            Ok(VaultCipherItemDto {
                id: cipher.id,
                folder_id: cipher.folder_id,
                organization_id: cipher.organization_id,
                r#type: cipher.r#type,
                name: decrypt_field_value(cipher.name, &user_key, "cipher.name")?,
                revision_date: cipher.revision_date,
                deleted_date: cipher.deleted_date,
                attachment_count: cipher.attachments.len().min(u32::MAX as usize) as u32,
            })
        })
        .collect();
    let cipher_items =
        cipher_items.map_err(|error| log_command_error("vault_get_view_data", error))?;

    Ok(VaultViewDataResponseDto {
        account_id,
        sync_status: mapping::to_sync_status_response_dto(context, Some(metrics)),
        folders: folder_items,
        ciphers: cipher_items,
        total_ciphers,
        page,
        page_size,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_cipher_detail(
    state: State<'_, AppState>,
    request: VaultCipherDetailRequestDto,
) -> Result<VaultCipherDetailResponseDto, String> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("vault_get_cipher_detail", error))?;
    let cipher_id = request.cipher_id.trim();
    if cipher_id.is_empty() {
        return Err(log_command_error(
            "vault_get_cipher_detail",
            AppError::validation("cipher_id cannot be empty"),
        ));
    }

    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("vault_get_cipher_detail", error))?
        .ok_or_else(|| {
            log_command_error(
                "vault_get_cipher_detail",
                AppError::validation("vault is locked, please unlock with master password first"),
            )
        })?;

    let cipher = state
        .sync_service()
        .get_live_cipher(account_id.clone(), String::from(cipher_id))
        .await
        .map_err(|error| log_command_error("vault_get_cipher_detail", error))?
        .ok_or_else(|| {
            log_command_error(
                "vault_get_cipher_detail",
                AppError::validation(format!("cipher not found: {cipher_id}")),
            )
        })?;

    let cipher = decrypt_cipher_detail(cipher, &user_key)
        .map_err(|error| log_command_error("vault_get_cipher_detail", error))?;

    Ok(VaultCipherDetailResponseDto { account_id, cipher })
}

fn normalize_page(page: Option<u32>) -> u32 {
    page.unwrap_or(DEFAULT_PAGE).max(1)
}

fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE)
}

fn decrypt_field_value(
    value: Option<String>,
    user_key: &VaultUserKey,
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

fn decrypt_cipher_detail(
    cipher: crate::application::dto::sync::SyncCipher,
    user_key: &VaultUserKey,
) -> Result<VaultCipherDetailDto, AppError> {
    let mut raw = serde_json::to_value(cipher).map_err(|error| {
        AppError::internal(format!("failed to serialize cipher detail: {error}"))
    })?;
    decrypt_strings_in_json_value(&mut raw, user_key, "$")?;
    convert_json_object_keys_to_camel_case(&mut raw);
    let detail = serde_json::from_value::<VaultCipherDetailDto>(raw).map_err(|error| {
        AppError::internal(format!(
            "failed to deserialize decrypted cipher detail: {error}"
        ))
    })?;

    Ok(detail)
}

fn decrypt_strings_in_json_value(
    value: &mut serde_json::Value,
    user_key: &VaultUserKey,
    path: &str,
) -> Result<(), AppError> {
    match value {
        serde_json::Value::String(raw) => {
            if looks_like_cipher_string(raw) {
                let plaintext = decrypt_cipher_string(raw, user_key).map_err(|error| {
                    AppError::validation(format!(
                        "failed to decrypt field `{path}`: {}",
                        error.message()
                    ))
                })?;
                *value = serde_json::Value::String(plaintext);
            }
            Ok(())
        }
        serde_json::Value::Array(values) => {
            for (index, item) in values.iter_mut().enumerate() {
                let child_path = format!("{path}[{index}]");
                decrypt_strings_in_json_value(item, user_key, &child_path)?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, entry) in map.iter_mut() {
                let child_path = if path == "$" {
                    format!("$.{key}")
                } else {
                    format!("{path}.{key}")
                };
                decrypt_strings_in_json_value(entry, user_key, &child_path)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn convert_json_object_keys_to_camel_case(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for item in values {
                convert_json_object_keys_to_camel_case(item);
            }
        }
        serde_json::Value::Object(map) => {
            let original = std::mem::take(map);
            for (key, mut entry) in original {
                convert_json_object_keys_to_camel_case(&mut entry);
                map.insert(snake_to_camel_case(&key), entry);
            }
        }
        _ => {}
    }
}

fn snake_to_camel_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut upper_next = false;
    for ch in value.chars() {
        if ch == '_' {
            upper_next = true;
            continue;
        }
        if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
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

#[derive(Debug, Clone)]
struct UnlockKdfParams {
    kdf_type: i32,
    iterations: i32,
    memory: Option<i32>,
    parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
struct UnlockMaterial {
    encrypted_user_keys: Vec<String>,
    kdf: Option<UnlockKdfParams>,
    salt: Option<String>,
}

#[derive(Debug, Clone)]
struct UnlockContext {
    account_id: String,
    base_url: String,
    email: String,
    kdf: Option<i32>,
    kdf_iterations: Option<i32>,
    kdf_memory: Option<i32>,
    kdf_parallelism: Option<i32>,
}

impl UnlockContext {
    fn from_auth_session(value: &crate::bootstrap::app_state::AuthSession) -> Self {
        Self {
            account_id: value.account_id.clone(),
            base_url: value.base_url.clone(),
            email: value.email.clone(),
            kdf: value.kdf,
            kdf_iterations: value.kdf_iterations,
            kdf_memory: value.kdf_memory,
            kdf_parallelism: value.kdf_parallelism,
        }
    }

    fn from_persisted_context(value: &PersistedAuthContext) -> Self {
        Self {
            account_id: value.account_id.clone(),
            base_url: value.base_url.clone(),
            email: value.email.clone(),
            kdf: value.kdf,
            kdf_iterations: value.kdf_iterations,
            kdf_memory: value.kdf_memory,
            kdf_parallelism: value.kdf_parallelism,
        }
    }
}

fn extract_unlock_material(
    value: Option<crate::application::dto::sync::SyncUserDecryption>,
) -> Result<UnlockMaterial, AppError> {
    let value = value.ok_or_else(|| {
        AppError::validation("missing local user_decryption data; run vault sync first")
    })?;
    let unlock = value.master_password_unlock.ok_or_else(|| {
        AppError::validation("missing master_password_unlock in local vault metadata")
    })?;

    let mut encrypted_user_keys = Vec::new();
    if let Some(value) = unlock.master_key_wrapped_user_key {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            encrypted_user_keys.push(String::from(trimmed));
        }
    }
    if let Some(value) = unlock.master_key_encrypted_user_key {
        let trimmed = value.trim();
        if !trimmed.is_empty() && encrypted_user_keys.iter().all(|item| item != trimmed) {
            encrypted_user_keys.push(String::from(trimmed));
        }
    }
    if encrypted_user_keys.is_empty() {
        return Err(AppError::validation(
            "encrypted user key is missing in local vault metadata",
        ));
    }

    let kdf = unlock.kdf.and_then(|value| {
        Some(UnlockKdfParams {
            kdf_type: value.kdf_type?,
            iterations: value.iterations?,
            memory: value.memory,
            parallelism: value.parallelism,
        })
    });

    let salt = unlock
        .salt
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(UnlockMaterial {
        encrypted_user_keys,
        kdf,
        salt,
    })
}

async fn derive_master_key_candidates_for_unlock(
    state: &AppState,
    unlock_context: &UnlockContext,
    master_password: &str,
    unlock_material: &UnlockMaterial,
) -> Result<Vec<Vec<u8>>, AppError> {
    let mut candidates = Vec::new();

    if let Some(kdf) = &unlock_material.kdf {
        let salt = unlock_material
            .salt
            .clone()
            .unwrap_or_else(|| unlock_context.email.clone());
        maybe_push_master_key(
            &mut candidates,
            &salt,
            master_password,
            kdf.kdf_type,
            kdf.iterations,
            kdf.memory,
            kdf.parallelism,
        )?;
    }

    if let (Some(kdf), Some(iterations)) = (unlock_context.kdf, unlock_context.kdf_iterations) {
        maybe_push_master_key(
            &mut candidates,
            &unlock_context.email,
            master_password,
            kdf,
            iterations,
            unlock_context.kdf_memory,
            unlock_context.kdf_parallelism,
        )?;
    }

    match state
        .auth_service()
        .prelogin(PreloginQuery {
            base_url: unlock_context.base_url.clone(),
            email: unlock_context.email.clone(),
        })
        .await
    {
        Ok(prelogin) => {
            maybe_push_master_key(
                &mut candidates,
                &unlock_context.email,
                master_password,
                prelogin.kdf,
                prelogin.kdf_iterations,
                prelogin.kdf_memory,
                prelogin.kdf_parallelism,
            )?;
        }
        Err(error) => {
            log::warn!(
                target: "vanguard::tauri::vault",
                "skip prelogin master key candidate account_id={} status={} error_code={} message={}",
                unlock_context.account_id,
                error
                    .status()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error.log_message()
            );
        }
    }

    if candidates.is_empty() {
        return Err(AppError::validation(
            "unable to derive any master key candidates for unlock",
        ));
    }

    Ok(candidates)
}

async fn resolve_unlock_context(
    state: &AppState,
    master_password: &str,
) -> AppResult<UnlockContext> {
    if let Some(auth_session) = state.auth_session()? {
        return Ok(UnlockContext::from_auth_session(&auth_session));
    }

    let persisted_context = state.persisted_auth_context()?.ok_or_else(|| {
        AppError::validation(
            "no authenticated or persisted account state found, please login first",
        )
    })?;

    match session::restore_auth_session_with_master_password(state, master_password).await {
        Ok(auth_session) => Ok(UnlockContext::from_auth_session(&auth_session)),
        Err(error) => {
            log::warn!(
                target: "vanguard::tauri::vault",
                "fallback to persisted account context for local unlock account_id={} status={} error_code={} message={}",
                persisted_context.account_id,
                error
                    .status()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error.log_message()
            );
            Ok(UnlockContext::from_persisted_context(&persisted_context))
        }
    }
}

fn maybe_push_master_key(
    candidates: &mut Vec<Vec<u8>>,
    email_or_salt: &str,
    master_password: &str,
    kdf: i32,
    kdf_iterations: i32,
    kdf_memory: Option<i32>,
    kdf_parallelism: Option<i32>,
) -> Result<(), AppError> {
    let key = derive_master_key(
        email_or_salt,
        master_password,
        kdf,
        kdf_iterations,
        kdf_memory,
        kdf_parallelism,
    )
    .map_err(|error| {
        AppError::validation(format!(
            "failed to derive master key with provided kdf params: {error}"
        ))
    })?;
    if candidates.iter().all(|existing| existing != &key) {
        candidates.push(key);
    }
    Ok(())
}

fn decrypt_user_key_with_master_keys(
    encrypted_user_keys: &[String],
    master_keys: &[Vec<u8>],
) -> Result<VaultUserKey, AppError> {
    if encrypted_user_keys.is_empty() {
        return Err(AppError::validation(
            "encrypted_user_key list cannot be empty",
        ));
    }

    for encrypted_user_key in encrypted_user_keys {
        let trimmed = encrypted_user_key.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !looks_like_cipher_string(trimmed) {
            if let Ok(parsed) = parse_user_key(trimmed) {
                return Ok(parsed);
            }
            continue;
        }

        for master_key in master_keys {
            for candidate in candidate_keys_from_master_key(master_key) {
                if let Ok(plaintext_user_key) = decrypt_cipher_bytes(trimmed, &candidate) {
                    if let Ok(user_key) = parse_user_key_material(&plaintext_user_key) {
                        return Ok(user_key);
                    }
                }
            }
        }
    }

    let enc_types: Vec<String> = encrypted_user_keys
        .iter()
        .filter_map(|value| value.trim().split_once('.').map(|(enc_type, _)| enc_type))
        .map(String::from)
        .collect();

    log::warn!(
        target: "vanguard::tauri::vault",
        "failed to decrypt encrypted_user_key candidates_count={} master_key_candidates={} enc_types={:?}",
        encrypted_user_keys.len(),
        master_keys.len(),
        enc_types
    );

    Err(AppError::validation(
        "unable to unlock encrypted_user_key with provided password",
    ))
}

fn candidate_keys_from_master_key(master_key: &[u8]) -> Vec<VaultUserKey> {
    let mut candidates = Vec::new();

    if master_key.len() == 32 {
        candidates.push(VaultUserKey {
            enc_key: hkdf_expand_from_prk(master_key, b"enc", 32),
            mac_key: Some(hkdf_expand_from_prk(master_key, b"mac", 32)),
        });
        candidates.push(VaultUserKey {
            enc_key: master_key.to_vec(),
            mac_key: None,
        });
        candidates.push(VaultUserKey {
            enc_key: master_key.to_vec(),
            mac_key: Some(hmac_derive(master_key, b"mac")),
        });
        candidates.push(VaultUserKey {
            enc_key: hkdf_expand_with_salt(master_key, &[0u8; 32], b"enc", 32),
            mac_key: Some(hkdf_expand_with_salt(master_key, &[0u8; 32], b"mac", 32)),
        });
        candidates.push(VaultUserKey {
            enc_key: hkdf_expand_with_salt(master_key, BITWARDEN_HKDF_SALT, b"enc", 32),
            mac_key: Some(hkdf_expand_with_salt(
                master_key,
                BITWARDEN_HKDF_SALT,
                b"mac",
                32,
            )),
        });
        let hashed = Sha256::digest(master_key).to_vec();
        candidates.push(VaultUserKey {
            enc_key: hashed.clone(),
            mac_key: Some(hmac_derive(&hashed, b"mac")),
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

fn hkdf_expand_with_salt(ikm: &[u8], salt: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let mut extract = HmacSha256::new_from_slice(salt).expect("hkdf salt");
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

fn hkdf_expand_from_prk(prk: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let mut okm = Vec::with_capacity(len);
    let mut previous = Vec::new();
    let mut counter: u8 = 1;

    while okm.len() < len {
        let mut expand = HmacSha256::new_from_slice(prk).expect("hkdf prk");
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
    let plaintext = decrypt_cipher_bytes(value, key)?;
    String::from_utf8(plaintext)
        .map_err(|error| AppError::validation(format!("plaintext is not utf-8: {error}")))
}

fn decrypt_cipher_bytes(value: &str, key: &VaultUserKey) -> Result<Vec<u8>, AppError> {
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

fn decrypt_aes_cbc(iv: &[u8], ciphertext: &[u8], enc_key: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut buffer = ciphertext.to_vec();
    let decryptor = Aes256CbcDecryptor::new_from_slices(enc_key, iv)
        .map_err(|error| AppError::validation(format!("invalid aes key/iv: {error}")))?;
    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| AppError::validation("ciphertext decryption failed"))?;
    Ok(plaintext.to_vec())
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

fn parse_user_key_material(raw: &[u8]) -> Result<VaultUserKey, AppError> {
    match raw.len() {
        32 => {
            return Ok(VaultUserKey {
                enc_key: raw.to_vec(),
                mac_key: None,
            });
        }
        64 => {
            return Ok(VaultUserKey {
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
    fn decrypt_strings_in_json_value_decrypts_nested_strings() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKey {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
        };
        let secret = encrypt_type2("nested-secret", &enc_key, &mac_key);
        let mut payload = serde_json::json!({
            "plain": "hello",
            "nested": { "secret": secret },
            "list": [encrypt_type2("array-secret", &enc_key, &mac_key)]
        });

        decrypt_strings_in_json_value(&mut payload, &user_key, "$").expect("must decrypt");

        assert_eq!(payload["plain"], serde_json::json!("hello"));
        assert_eq!(
            payload["nested"]["secret"],
            serde_json::json!("nested-secret")
        );
        assert_eq!(payload["list"][0], serde_json::json!("array-secret"));
    }

    #[test]
    fn decrypt_strings_in_json_value_returns_error_when_decryption_fails() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKey {
            enc_key: [7u8; 32].to_vec(),
            mac_key: Some([8u8; 32].to_vec()),
        };
        let secret = encrypt_type2("nested-secret", &enc_key, &mac_key);
        let mut payload = serde_json::json!({
            "nested": { "secret": secret }
        });

        let error = decrypt_strings_in_json_value(&mut payload, &user_key, "$")
            .expect_err("must fail on decryption failure");
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn decrypt_cipher_detail_supports_snake_case_fields() {
        let user_key = VaultUserKey {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
        };
        let cipher = crate::application::dto::sync::SyncCipher {
            id: String::from("cipher-1"),
            organization_id: None,
            folder_id: None,
            r#type: Some(1),
            name: Some(String::from("demo")),
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
            password_history: vec![crate::application::dto::sync::SyncCipherPasswordHistory {
                password: Some(String::from("history-pass")),
                last_used_date: Some(String::from("2026-03-01T00:00:00Z")),
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

        let detail = decrypt_cipher_detail(cipher, &user_key).expect("detail deserialize");
        assert_eq!(detail.id, "cipher-1");
        assert_eq!(detail.password_history.len(), 1);
        assert_eq!(
            detail.password_history[0].last_used_date.as_deref(),
            Some("2026-03-01T00:00:00Z")
        );
    }

    #[test]
    fn snake_to_camel_case_converts_expected_keys() {
        assert_eq!(snake_to_camel_case("password_history"), "passwordHistory");
        assert_eq!(snake_to_camel_case("fido2_credentials"), "fido2Credentials");
        assert_eq!(snake_to_camel_case("id"), "id");
    }

    #[test]
    fn decrypt_user_key_with_master_keys_supports_hmac_derived_mac_key() {
        let master_key = [7u8; 32];
        let enc_key = master_key;
        let mac_key_vec = hmac_derive(&master_key, b"mac");
        let mac_key: [u8; 32] = mac_key_vec
            .try_into()
            .expect("hmac output should be 32 bytes");
        let plain_user_key = STANDARD.encode([3u8; 64]);
        let encrypted_user_key = encrypt_type2(&plain_user_key, &enc_key, &mac_key);

        let parsed =
            decrypt_user_key_with_master_keys(&[encrypted_user_key], &[master_key.to_vec()])
                .expect("unlock with password candidate key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn decrypt_user_key_with_master_keys_supports_raw_64_byte_material() {
        let master_key = [7u8; 32];
        let enc_key = hkdf_expand_from_prk(&master_key, b"enc", 32);
        let mac_key_vec = hkdf_expand_from_prk(&master_key, b"mac", 32);
        let mac_key: [u8; 32] = mac_key_vec
            .try_into()
            .expect("hkdf output should be 32 bytes");
        let plain_user_key = vec![3u8; 64];
        let encrypted_user_key = encrypt_type2_bytes(&plain_user_key, &enc_key, &mac_key);

        let parsed =
            decrypt_user_key_with_master_keys(&[encrypted_user_key], &[master_key.to_vec()])
                .expect("unlock with password candidate key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn extract_unlock_material_prefers_wrapped_variant() {
        let extracted =
            extract_unlock_material(Some(crate::application::dto::sync::SyncUserDecryption {
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

        assert_eq!(
            extracted.encrypted_user_keys,
            vec![
                String::from("2.wrapped|payload|mac"),
                String::from("2.encrypted|payload|mac")
            ]
        );
    }

    #[test]
    fn extract_unlock_material_fallbacks_to_encrypted_variant() {
        let extracted =
            extract_unlock_material(Some(crate::application::dto::sync::SyncUserDecryption {
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

        assert_eq!(
            extracted.encrypted_user_keys,
            vec![String::from("2.encrypted|payload|mac")]
        );
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
}
