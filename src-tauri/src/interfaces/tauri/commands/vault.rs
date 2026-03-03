use aes::Aes256;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::application::dto::auth::PreloginQuery;
use crate::bootstrap::app_state::{AppState, PersistedAuthContext, VaultUserKey};
use crate::infrastructure::security::biometric_store;
use crate::infrastructure::vaultwarden::password_hash::derive_master_key;
use crate::interfaces::tauri::dto::vault::{
    VaultBiometricStatusResponseDto, VaultCipherDetailDto, VaultCipherDetailRequestDto,
    VaultCipherDetailResponseDto, VaultCipherItemDto, VaultCipherPermissionsDetailDto,
    VaultCipherSecureNoteDetailDto, VaultDisableBiometricUnlockRequestDto,
    VaultEnableBiometricUnlockRequestDto, VaultFolderItemDto, VaultLockRequestDto,
    VaultUnlockWithPasswordRequestDto, VaultViewDataRequestDto, VaultViewDataResponseDto,
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
pub async fn vault_can_unlock(state: State<'_, AppState>) -> Result<bool, String> {
    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(AppError::Validation(_)) => return Ok(false),
        Err(error) => return Err(log_command_error("vault_can_unlock", error)),
    };

    let user_decryption = state
        .sync_service()
        .load_live_user_decryption(account_id)
        .await
        .map_err(|error| log_command_error("vault_can_unlock", error))?;

    match extract_unlock_material(user_decryption) {
        Ok(_) => Ok(true),
        Err(error) if is_unlock_material_missing(&error) => Ok(false),
        Err(error) => Err(log_command_error("vault_can_unlock", error)),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn vault_is_unlocked(state: State<'_, AppState>) -> Result<bool, String> {
    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(AppError::Validation(_)) => return Ok(false),
        Err(error) => return Err(log_command_error("vault_is_unlocked", error)),
    };

    state
        .get_vault_user_key(&account_id)
        .map(|value| value.is_some())
        .map_err(|error| log_command_error("vault_is_unlocked", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock_with_password(
    state: State<'_, AppState>,
    request: VaultUnlockWithPasswordRequestDto,
) -> Result<(), String> {
    unlock_with_master_password(&state, &request.master_password)
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_biometric_status(
    state: State<'_, AppState>,
) -> Result<VaultBiometricStatusResponseDto, String> {
    if !biometric_store::is_supported() {
        return Ok(VaultBiometricStatusResponseDto {
            supported: false,
            enabled: false,
        });
    }

    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(AppError::Validation(_)) => {
            return Ok(VaultBiometricStatusResponseDto {
                supported: true,
                enabled: false,
            });
        }
        Err(error) => return Err(log_command_error("vault_get_biometric_status", error)),
    };
    let enabled = biometric_store::has_unlock_bundle(&account_id)
        .map_err(|error| log_command_error("vault_get_biometric_status", error))?;
    Ok(VaultBiometricStatusResponseDto {
        supported: true,
        enabled,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_can_unlock_with_biometric(state: State<'_, AppState>) -> Result<bool, String> {
    if !biometric_store::is_supported() {
        return Ok(false);
    }

    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(AppError::Validation(_)) => return Ok(false),
        Err(error) => return Err(log_command_error("vault_can_unlock_with_biometric", error)),
    };

    let user_decryption = state
        .sync_service()
        .load_live_user_decryption(account_id.clone())
        .await
        .map_err(|error| log_command_error("vault_can_unlock_with_biometric", error))?;
    match extract_unlock_material(user_decryption) {
        Ok(_) => {}
        Err(error) if is_unlock_material_missing(&error) => return Ok(false),
        Err(error) => return Err(log_command_error("vault_can_unlock_with_biometric", error)),
    }

    biometric_store::has_unlock_bundle(&account_id)
        .map_err(|error| log_command_error("vault_can_unlock_with_biometric", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_enable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultEnableBiometricUnlockRequestDto,
) -> Result<(), String> {
    if !biometric_store::is_supported() {
        return Err(log_command_error(
            "vault_enable_biometric_unlock",
            AppError::validation("biometric unlock is only supported on macOS"),
        ));
    }

    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))?;

    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))?
        .ok_or_else(|| {
            log_command_error(
                "vault_enable_biometric_unlock",
                AppError::validation(
                    "vault is locked, please unlock with password before enabling touch id",
                ),
            )
        })?;
    let bundle = vault_user_key_to_biometric_bundle(&account_id, &user_key)
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))?;

    biometric_store::save_unlock_bundle(&account_id, &bundle)
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))?;
    let verified_bundle = biometric_store::load_unlock_bundle(&account_id)
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))?;
    if verified_bundle.account_id != account_id {
        return Err(log_command_error(
            "vault_enable_biometric_unlock",
            AppError::internal("biometric verification returned mismatched account id"),
        ));
    }

    log::info!(
        target: "vanguard::tauri::vault",
        "biometric unlock enabled account_id={}",
        account_id
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_disable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultDisableBiometricUnlockRequestDto,
) -> Result<(), String> {
    if !biometric_store::is_supported() {
        return Ok(());
    }

    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(AppError::Validation(_)) => return Ok(()),
        Err(error) => return Err(log_command_error("vault_disable_biometric_unlock", error)),
    };

    biometric_store::delete_unlock_bundle(&account_id)
        .map_err(|error| log_command_error("vault_disable_biometric_unlock", error))?;
    log::info!(
        target: "vanguard::tauri::vault",
        "biometric unlock disabled account_id={}",
        account_id
    );
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock_with_biometric(state: State<'_, AppState>) -> Result<(), String> {
    if !biometric_store::is_supported() {
        return Err(log_command_error(
            "vault_unlock_with_biometric",
            AppError::validation("biometric unlock is only supported on macOS"),
        ));
    }

    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("vault_unlock_with_biometric", error))?;
    let bundle = biometric_store::load_unlock_bundle(&account_id)
        .map_err(|error| log_command_error("vault_unlock_with_biometric", error))?;
    if bundle.account_id != account_id {
        return Err(log_command_error(
            "vault_unlock_with_biometric",
            AppError::validation("biometric unlock account does not match current account"),
        ));
    }
    let user_key = biometric_bundle_to_vault_user_key(&bundle)
        .map_err(|error| log_command_error("vault_unlock_with_biometric", error))?;
    state
        .set_vault_user_key(account_id.clone(), user_key)
        .map_err(|error| log_command_error("vault_unlock_with_biometric", error))?;

    log::info!(
        target: "vanguard::tauri::vault",
        "vault unlocked with biometric account_id={}",
        account_id
    );
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

async fn unlock_with_master_password(state: &AppState, master_password: &str) -> AppResult<String> {
    let master_password = master_password.trim().to_string();
    if master_password.is_empty() {
        return Err(AppError::validation("master_password cannot be empty"));
    }

    let unlock_context = resolve_unlock_context(state, &master_password).await?;
    let account_id = unlock_context.account_id.clone();

    let unlock_material = state
        .sync_service()
        .load_live_user_decryption(account_id.clone())
        .await
        .and_then(extract_unlock_material)?;

    let master_keys = derive_master_key_candidates_for_unlock(
        state,
        &unlock_context,
        &master_password,
        &unlock_material,
    )
    .await?;

    let user_key =
        decrypt_user_key_with_master_keys(&unlock_material.encrypted_user_keys, &master_keys)?;
    state.set_vault_user_key(account_id.clone(), user_key)?;

    log::info!(
        target: "vanguard::tauri::vault",
        "vault unlocked with password in memory account_id={}",
        account_id
    );

    if state.auth_session()?.is_none() {
        if let Err(error) =
            session::restore_auth_session_with_master_password(state, &master_password).await
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

    Ok(account_id)
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
    let detail_key = resolve_cipher_decryption_key(cipher.key.as_deref(), user_key)?;

    Ok(VaultCipherDetailDto {
        id: cipher.id,
        organization_id: cipher.organization_id,
        folder_id: cipher.folder_id,
        r#type: cipher.r#type,
        name: decrypt_field_value(cipher.name, &detail_key, "cipher.name")?,
        notes: decrypt_field_value(cipher.notes, &detail_key, "cipher.notes")?,
        key: cipher.key,
        favorite: cipher.favorite,
        edit: cipher.edit,
        view_password: cipher.view_password,
        organization_use_totp: cipher.organization_use_totp,
        creation_date: cipher.creation_date,
        revision_date: cipher.revision_date,
        deleted_date: cipher.deleted_date,
        archived_date: cipher.archived_date,
        reprompt: cipher.reprompt,
        permissions: cipher
            .permissions
            .map(|permissions| VaultCipherPermissionsDetailDto {
                delete: permissions.delete,
                restore: permissions.restore,
            }),
        object: cipher.object,
        fields: decrypt_cipher_fields(cipher.fields, &detail_key, "cipher.fields")?,
        password_history: decrypt_password_history(
            cipher.password_history,
            &detail_key,
            "cipher.password_history",
        )?,
        collection_ids: cipher.collection_ids,
        data: decrypt_cipher_data_detail(cipher.data, &detail_key)?,
        login: decrypt_cipher_login_detail(cipher.login, &detail_key)?,
        secure_note: cipher
            .secure_note
            .map(|note| VaultCipherSecureNoteDetailDto {
                r#type: note.r#type,
            }),
        card: decrypt_cipher_card_detail(cipher.card, &detail_key)?,
        identity: decrypt_cipher_identity_detail(cipher.identity, &detail_key)?,
        ssh_key: decrypt_cipher_ssh_key_detail(cipher.ssh_key, &detail_key)?,
        attachments: decrypt_attachments(cipher.attachments, &detail_key)?,
    })
}

fn resolve_cipher_decryption_key(
    cipher_key: Option<&str>,
    user_key: &VaultUserKey,
) -> Result<VaultUserKey, AppError> {
    let Some(raw) = cipher_key else {
        return Ok(user_key.clone());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(user_key.clone());
    }

    if looks_like_cipher_string(trimmed) {
        let decrypted = decrypt_cipher_bytes(trimmed, user_key).map_err(|error| {
            AppError::validation(format!(
                "failed to decrypt field `cipher.key`: {}",
                error.message()
            ))
        })?;
        return parse_user_key_material(&decrypted).map_err(|error| {
            AppError::validation(format!(
                "failed to parse decrypted field `cipher.key`: {}",
                error.message()
            ))
        });
    }

    parse_user_key(trimmed).map_err(|error| {
        AppError::validation(format!(
            "failed to parse field `cipher.key`: {}",
            error.message()
        ))
    })
}

fn decrypt_cipher_fields(
    fields: Vec<crate::application::dto::sync::SyncCipherField>,
    user_key: &VaultUserKey,
    path: &str,
) -> Result<Vec<crate::interfaces::tauri::dto::vault::VaultCipherFieldDetailDto>, AppError> {
    fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let entry_path = format!("{path}[{index}]");
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherFieldDetailDto {
                    name: decrypt_field_value(field.name, user_key, &format!("{entry_path}.name"))?,
                    value: decrypt_field_value(
                        field.value,
                        user_key,
                        &format!("{entry_path}.value"),
                    )?,
                    r#type: field.r#type,
                    linked_id: field.linked_id,
                },
            )
        })
        .collect()
}

fn decrypt_password_history(
    entries: Vec<crate::application::dto::sync::SyncCipherPasswordHistory>,
    user_key: &VaultUserKey,
    path: &str,
) -> Result<Vec<crate::interfaces::tauri::dto::vault::VaultCipherPasswordHistoryDetailDto>, AppError>
{
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let entry_path = format!("{path}[{index}]");
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherPasswordHistoryDetailDto {
                    password: decrypt_field_value(
                        entry.password,
                        user_key,
                        &format!("{entry_path}.password"),
                    )?,
                    last_used_date: entry.last_used_date,
                },
            )
        })
        .collect()
}

fn decrypt_attachments(
    attachments: Vec<crate::application::dto::sync::SyncAttachment>,
    user_key: &VaultUserKey,
) -> Result<Vec<crate::interfaces::tauri::dto::vault::VaultAttachmentDetailDto>, AppError> {
    attachments
        .into_iter()
        .enumerate()
        .map(|(index, attachment)| {
            let entry_path = format!("cipher.attachments[{index}]");
            Ok(
                crate::interfaces::tauri::dto::vault::VaultAttachmentDetailDto {
                    id: attachment.id,
                    key: attachment.key,
                    file_name: decrypt_field_value(
                        attachment.file_name,
                        user_key,
                        &format!("{entry_path}.file_name"),
                    )?,
                    size: attachment.size,
                    size_name: attachment.size_name,
                    url: attachment.url,
                    object: attachment.object,
                },
            )
        })
        .collect()
}

fn decrypt_cipher_data_detail(
    data: Option<crate::application::dto::sync::SyncCipherData>,
    user_key: &VaultUserKey,
) -> Result<Option<crate::interfaces::tauri::dto::vault::VaultCipherDataDetailDto>, AppError> {
    data.map(|entry| {
        Ok(
            crate::interfaces::tauri::dto::vault::VaultCipherDataDetailDto {
                name: decrypt_field_value(entry.name, user_key, "cipher.data.name")?,
                notes: decrypt_field_value(entry.notes, user_key, "cipher.data.notes")?,
                fields: decrypt_cipher_fields(entry.fields, user_key, "cipher.data.fields")?,
                password_history: decrypt_password_history(
                    entry.password_history,
                    user_key,
                    "cipher.data.password_history",
                )?,
                uri: decrypt_field_value(entry.uri, user_key, "cipher.data.uri")?,
                uris: decrypt_login_uris(entry.uris, user_key, "cipher.data.uris")?,
                username: decrypt_field_value(entry.username, user_key, "cipher.data.username")?,
                password: decrypt_field_value(entry.password, user_key, "cipher.data.password")?,
                password_revision_date: entry.password_revision_date,
                totp: decrypt_field_value(entry.totp, user_key, "cipher.data.totp")?,
                autofill_on_page_load: entry.autofill_on_page_load,
                fido2_credentials: decrypt_fido2_credentials(
                    entry.fido2_credentials,
                    user_key,
                    "cipher.data.fido2_credentials",
                )?,
                r#type: entry.r#type,
                cardholder_name: decrypt_field_value(
                    entry.cardholder_name,
                    user_key,
                    "cipher.data.cardholder_name",
                )?,
                brand: decrypt_field_value(entry.brand, user_key, "cipher.data.brand")?,
                number: decrypt_field_value(entry.number, user_key, "cipher.data.number")?,
                exp_month: decrypt_field_value(entry.exp_month, user_key, "cipher.data.exp_month")?,
                exp_year: decrypt_field_value(entry.exp_year, user_key, "cipher.data.exp_year")?,
                code: decrypt_field_value(entry.code, user_key, "cipher.data.code")?,
                title: decrypt_field_value(entry.title, user_key, "cipher.data.title")?,
                first_name: decrypt_field_value(
                    entry.first_name,
                    user_key,
                    "cipher.data.first_name",
                )?,
                middle_name: decrypt_field_value(
                    entry.middle_name,
                    user_key,
                    "cipher.data.middle_name",
                )?,
                last_name: decrypt_field_value(entry.last_name, user_key, "cipher.data.last_name")?,
                address1: decrypt_field_value(entry.address1, user_key, "cipher.data.address1")?,
                address2: decrypt_field_value(entry.address2, user_key, "cipher.data.address2")?,
                address3: decrypt_field_value(entry.address3, user_key, "cipher.data.address3")?,
                city: decrypt_field_value(entry.city, user_key, "cipher.data.city")?,
                state: decrypt_field_value(entry.state, user_key, "cipher.data.state")?,
                postal_code: decrypt_field_value(
                    entry.postal_code,
                    user_key,
                    "cipher.data.postal_code",
                )?,
                country: decrypt_field_value(entry.country, user_key, "cipher.data.country")?,
                company: decrypt_field_value(entry.company, user_key, "cipher.data.company")?,
                email: decrypt_field_value(entry.email, user_key, "cipher.data.email")?,
                phone: decrypt_field_value(entry.phone, user_key, "cipher.data.phone")?,
                ssn: decrypt_field_value(entry.ssn, user_key, "cipher.data.ssn")?,
                passport_number: decrypt_field_value(
                    entry.passport_number,
                    user_key,
                    "cipher.data.passport_number",
                )?,
                license_number: decrypt_field_value(
                    entry.license_number,
                    user_key,
                    "cipher.data.license_number",
                )?,
                private_key: decrypt_field_value(
                    entry.private_key,
                    user_key,
                    "cipher.data.private_key",
                )?,
                public_key: decrypt_field_value(
                    entry.public_key,
                    user_key,
                    "cipher.data.public_key",
                )?,
                key_fingerprint: decrypt_field_value(
                    entry.key_fingerprint,
                    user_key,
                    "cipher.data.key_fingerprint",
                )?,
            },
        )
    })
    .transpose()
}

fn decrypt_cipher_login_detail(
    login: Option<crate::application::dto::sync::SyncCipherLogin>,
    user_key: &VaultUserKey,
) -> Result<Option<crate::interfaces::tauri::dto::vault::VaultCipherLoginDetailDto>, AppError> {
    login
        .map(|entry| {
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherLoginDetailDto {
                    uri: decrypt_field_value(entry.uri, user_key, "cipher.login.uri")?,
                    uris: decrypt_login_uris(entry.uris, user_key, "cipher.login.uris")?,
                    username: decrypt_field_value(
                        entry.username,
                        user_key,
                        "cipher.login.username",
                    )?,
                    password: decrypt_field_value(
                        entry.password,
                        user_key,
                        "cipher.login.password",
                    )?,
                    password_revision_date: entry.password_revision_date,
                    totp: decrypt_field_value(entry.totp, user_key, "cipher.login.totp")?,
                    autofill_on_page_load: entry.autofill_on_page_load,
                    fido2_credentials: decrypt_fido2_credentials(
                        entry.fido2_credentials,
                        user_key,
                        "cipher.login.fido2_credentials",
                    )?,
                },
            )
        })
        .transpose()
}

fn decrypt_login_uris(
    uris: Vec<crate::application::dto::sync::SyncCipherLoginUri>,
    user_key: &VaultUserKey,
    path: &str,
) -> Result<Vec<crate::interfaces::tauri::dto::vault::VaultCipherLoginUriDetailDto>, AppError> {
    uris.into_iter()
        .enumerate()
        .map(|(index, uri)| {
            let entry_path = format!("{path}[{index}]");
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherLoginUriDetailDto {
                    uri: decrypt_field_value(uri.uri, user_key, &format!("{entry_path}.uri"))?,
                    r#match: uri.r#match,
                    uri_checksum: uri.uri_checksum,
                },
            )
        })
        .collect()
}

fn decrypt_fido2_credentials(
    credentials: Vec<crate::application::dto::sync::SyncCipherLoginFido2Credential>,
    user_key: &VaultUserKey,
    path: &str,
) -> Result<
    Vec<crate::interfaces::tauri::dto::vault::VaultCipherLoginFido2CredentialDetailDto>,
    AppError,
> {
    credentials
        .into_iter()
        .enumerate()
        .map(|(index, credential)| {
            let entry_path = format!("{path}[{index}]");
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherLoginFido2CredentialDetailDto {
                    credential_id: decrypt_field_value(
                        credential.credential_id,
                        user_key,
                        &format!("{entry_path}.credential_id"),
                    )?,
                    key_type: decrypt_field_value(
                        credential.key_type,
                        user_key,
                        &format!("{entry_path}.key_type"),
                    )?,
                    key_algorithm: decrypt_field_value(
                        credential.key_algorithm,
                        user_key,
                        &format!("{entry_path}.key_algorithm"),
                    )?,
                    key_curve: decrypt_field_value(
                        credential.key_curve,
                        user_key,
                        &format!("{entry_path}.key_curve"),
                    )?,
                    key_value: decrypt_field_value(
                        credential.key_value,
                        user_key,
                        &format!("{entry_path}.key_value"),
                    )?,
                    rp_id: decrypt_field_value(
                        credential.rp_id,
                        user_key,
                        &format!("{entry_path}.rp_id"),
                    )?,
                    rp_name: decrypt_field_value(
                        credential.rp_name,
                        user_key,
                        &format!("{entry_path}.rp_name"),
                    )?,
                    counter: decrypt_field_value(
                        credential.counter,
                        user_key,
                        &format!("{entry_path}.counter"),
                    )?,
                    user_handle: decrypt_field_value(
                        credential.user_handle,
                        user_key,
                        &format!("{entry_path}.user_handle"),
                    )?,
                    user_name: decrypt_field_value(
                        credential.user_name,
                        user_key,
                        &format!("{entry_path}.user_name"),
                    )?,
                    user_display_name: decrypt_field_value(
                        credential.user_display_name,
                        user_key,
                        &format!("{entry_path}.user_display_name"),
                    )?,
                    discoverable: decrypt_field_value(
                        credential.discoverable,
                        user_key,
                        &format!("{entry_path}.discoverable"),
                    )?,
                    creation_date: decrypt_field_value(
                        credential.creation_date,
                        user_key,
                        &format!("{entry_path}.creation_date"),
                    )?,
                },
            )
        })
        .collect()
}

fn decrypt_cipher_card_detail(
    card: Option<crate::application::dto::sync::SyncCipherCard>,
    user_key: &VaultUserKey,
) -> Result<Option<crate::interfaces::tauri::dto::vault::VaultCipherCardDetailDto>, AppError> {
    card.map(|entry| {
        Ok(
            crate::interfaces::tauri::dto::vault::VaultCipherCardDetailDto {
                cardholder_name: decrypt_field_value(
                    entry.cardholder_name,
                    user_key,
                    "cipher.card.cardholder_name",
                )?,
                brand: decrypt_field_value(entry.brand, user_key, "cipher.card.brand")?,
                number: decrypt_field_value(entry.number, user_key, "cipher.card.number")?,
                exp_month: decrypt_field_value(entry.exp_month, user_key, "cipher.card.exp_month")?,
                exp_year: decrypt_field_value(entry.exp_year, user_key, "cipher.card.exp_year")?,
                code: decrypt_field_value(entry.code, user_key, "cipher.card.code")?,
            },
        )
    })
    .transpose()
}

fn decrypt_cipher_identity_detail(
    identity: Option<crate::application::dto::sync::SyncCipherIdentity>,
    user_key: &VaultUserKey,
) -> Result<Option<crate::interfaces::tauri::dto::vault::VaultCipherIdentityDetailDto>, AppError> {
    identity
        .map(|entry| {
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherIdentityDetailDto {
                    title: decrypt_field_value(entry.title, user_key, "cipher.identity.title")?,
                    first_name: decrypt_field_value(
                        entry.first_name,
                        user_key,
                        "cipher.identity.first_name",
                    )?,
                    middle_name: decrypt_field_value(
                        entry.middle_name,
                        user_key,
                        "cipher.identity.middle_name",
                    )?,
                    last_name: decrypt_field_value(
                        entry.last_name,
                        user_key,
                        "cipher.identity.last_name",
                    )?,
                    address1: decrypt_field_value(
                        entry.address1,
                        user_key,
                        "cipher.identity.address1",
                    )?,
                    address2: decrypt_field_value(
                        entry.address2,
                        user_key,
                        "cipher.identity.address2",
                    )?,
                    address3: decrypt_field_value(
                        entry.address3,
                        user_key,
                        "cipher.identity.address3",
                    )?,
                    city: decrypt_field_value(entry.city, user_key, "cipher.identity.city")?,
                    state: decrypt_field_value(entry.state, user_key, "cipher.identity.state")?,
                    postal_code: decrypt_field_value(
                        entry.postal_code,
                        user_key,
                        "cipher.identity.postal_code",
                    )?,
                    country: decrypt_field_value(
                        entry.country,
                        user_key,
                        "cipher.identity.country",
                    )?,
                    company: decrypt_field_value(
                        entry.company,
                        user_key,
                        "cipher.identity.company",
                    )?,
                    email: decrypt_field_value(entry.email, user_key, "cipher.identity.email")?,
                    phone: decrypt_field_value(entry.phone, user_key, "cipher.identity.phone")?,
                    ssn: decrypt_field_value(entry.ssn, user_key, "cipher.identity.ssn")?,
                    username: decrypt_field_value(
                        entry.username,
                        user_key,
                        "cipher.identity.username",
                    )?,
                    passport_number: decrypt_field_value(
                        entry.passport_number,
                        user_key,
                        "cipher.identity.passport_number",
                    )?,
                    license_number: decrypt_field_value(
                        entry.license_number,
                        user_key,
                        "cipher.identity.license_number",
                    )?,
                },
            )
        })
        .transpose()
}

fn decrypt_cipher_ssh_key_detail(
    ssh_key: Option<crate::application::dto::sync::SyncCipherSshKey>,
    user_key: &VaultUserKey,
) -> Result<Option<crate::interfaces::tauri::dto::vault::VaultCipherSshKeyDetailDto>, AppError> {
    ssh_key
        .map(|entry| {
            Ok(
                crate::interfaces::tauri::dto::vault::VaultCipherSshKeyDetailDto {
                    private_key: decrypt_field_value(
                        entry.private_key,
                        user_key,
                        "cipher.ssh_key.private_key",
                    )?,
                    public_key: decrypt_field_value(
                        entry.public_key,
                        user_key,
                        "cipher.ssh_key.public_key",
                    )?,
                    key_fingerprint: decrypt_field_value(
                        entry.key_fingerprint,
                        user_key,
                        "cipher.ssh_key.key_fingerprint",
                    )?,
                },
            )
        })
        .transpose()
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

fn vault_user_key_to_biometric_bundle(
    account_id: &str,
    user_key: &VaultUserKey,
) -> Result<biometric_store::BiometricUnlockBundle, AppError> {
    validate_key_lengths(&user_key.enc_key, user_key.mac_key.as_deref())?;
    Ok(biometric_store::BiometricUnlockBundle::new(
        String::from(account_id),
        STANDARD_NO_PAD.encode(&user_key.enc_key),
        user_key
            .mac_key
            .as_ref()
            .map(|value| STANDARD_NO_PAD.encode(value)),
    ))
}

fn biometric_bundle_to_vault_user_key(
    bundle: &biometric_store::BiometricUnlockBundle,
) -> Result<VaultUserKey, AppError> {
    if bundle.account_id.trim().is_empty() {
        return Err(AppError::validation(
            "biometric unlock bundle account_id is empty",
        ));
    }
    let enc_key = decode_base64_flexible(&bundle.enc_key_b64, "biometric.enc_key_b64")?;
    let mac_key = bundle
        .mac_key_b64
        .as_ref()
        .map(|value| decode_base64_flexible(value, "biometric.mac_key_b64"))
        .transpose()?;
    validate_key_lengths(&enc_key, mac_key.as_deref())?;
    Ok(VaultUserKey { enc_key, mac_key })
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

fn is_unlock_material_missing(error: &AppError) -> bool {
    let AppError::Validation(message) = error else {
        return false;
    };

    message.contains("missing local user_decryption data")
        || message.contains("missing master_password_unlock")
        || message.contains("encrypted user key is missing")
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
    fn decrypt_cipher_detail_decrypts_whitelisted_fields_only() {
        let enc_key = [1u8; 32];
        let mac_key = [2u8; 32];
        let user_key = VaultUserKey {
            enc_key: enc_key.to_vec(),
            mac_key: Some(mac_key.to_vec()),
        };
        let encrypted_name = encrypt_type2("demo", &enc_key, &mac_key);
        let encrypted_password = encrypt_type2("history-pass", &enc_key, &mac_key);
        let encrypted_login_uri = encrypt_type2("https://example.com/login", &enc_key, &mac_key);
        let cipher = crate::application::dto::sync::SyncCipher {
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
            password_history: vec![crate::application::dto::sync::SyncCipherPasswordHistory {
                password: Some(encrypted_password),
                last_used_date: Some(String::from("2026-03-01T00:00:00Z")),
            }],
            collection_ids: Vec::new(),
            data: None,
            login: Some(crate::application::dto::sync::SyncCipherLogin {
                uri: None,
                uris: vec![crate::application::dto::sync::SyncCipherLoginUri {
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
        assert_eq!(detail.name.as_deref(), Some("demo"));
        assert_eq!(detail.key, None);
        assert_eq!(detail.password_history.len(), 1);
        assert_eq!(
            detail.password_history[0].password.as_deref(),
            Some("history-pass")
        );
        assert_eq!(
            detail.password_history[0].last_used_date.as_deref(),
            Some("2026-03-01T00:00:00Z")
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0].uri.as_deref(),
            Some("https://example.com/login")
        );
        assert_eq!(
            detail.login.as_ref().expect("login").uris[0]
                .uri_checksum
                .as_deref(),
            Some("2.not-a-cipher|string|shape")
        );
    }

    #[test]
    fn decrypt_cipher_detail_uses_cipher_key_for_field_decryption() {
        let user_enc_key = [1u8; 32];
        let user_mac_key = [2u8; 32];
        let user_key = VaultUserKey {
            enc_key: user_enc_key.to_vec(),
            mac_key: Some(user_mac_key.to_vec()),
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

        let cipher = crate::application::dto::sync::SyncCipher {
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
            password_history: vec![crate::application::dto::sync::SyncCipherPasswordHistory {
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
        assert_eq!(detail.name.as_deref(), Some("cipher-name"));
        assert_eq!(
            detail.password_history[0].password.as_deref(),
            Some("cipher-pass")
        );
        assert_eq!(detail.key.as_deref(), Some(encrypted_cipher_key.as_str()));
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_on_whitelisted_field() {
        let user_key = VaultUserKey {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
        };
        let cipher = crate::application::dto::sync::SyncCipher {
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
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn decrypt_cipher_detail_rejects_invalid_cipher_key() {
        let user_key = VaultUserKey {
            enc_key: [1u8; 32].to_vec(),
            mac_key: Some([2u8; 32].to_vec()),
        };
        let cipher = crate::application::dto::sync::SyncCipher {
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
        assert_eq!(error.code(), "validation_error");
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
