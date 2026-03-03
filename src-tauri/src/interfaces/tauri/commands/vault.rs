use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use tauri::State;

use crate::application::dto::vault::{
    GetCipherDetailQuery, UnlockVaultWithPasswordCommand, VaultUserKeyMaterial,
};
use crate::application::use_cases::unlock_vault_with_password_use_case::{
    has_master_password_unlock_material, UnlockVaultWithPasswordUseCase,
};
use crate::application::vault_crypto;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::infrastructure::security::biometric_store;
use crate::interfaces::tauri::dto::vault::{
    VaultBiometricStatusResponseDto, VaultCipherDetailRequestDto, VaultCipherDetailResponseDto,
    VaultCipherItemDto, VaultDisableBiometricUnlockRequestDto,
    VaultEnableBiometricUnlockRequestDto, VaultFolderItemDto, VaultLockRequestDto,
    VaultUnlockWithPasswordRequestDto, VaultViewDataRequestDto, VaultViewDataResponseDto,
};
use crate::interfaces::tauri::{mapping, session};
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;

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

    has_master_password_unlock_material(user_decryption)
        .map_err(|error| log_command_error("vault_can_unlock", error))
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
    let master_password = request.master_password.trim().to_string();
    let unlocked = UnlockVaultWithPasswordUseCase::new(state.auth_service(), state.sync_service())
        .execute(
            &*state,
            UnlockVaultWithPasswordCommand {
                master_password: master_password.clone(),
            },
        )
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

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
                unlocked.account_id,
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
    if !has_master_password_unlock_material(user_decryption)
        .map_err(|error| log_command_error("vault_can_unlock_with_biometric", error))?
    {
        return Ok(false);
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
    let user_key_material = to_vault_user_key_material(&user_key);

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
                name: vault_crypto::decrypt_optional_field(
                    folder.name,
                    &user_key_material,
                    "folder.name",
                )?,
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
                name: vault_crypto::decrypt_optional_field(
                    cipher.name,
                    &user_key_material,
                    "cipher.name",
                )?,
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
        .get_cipher_detail_use_case()
        .execute(GetCipherDetailQuery {
            account_id: account_id.clone(),
            cipher_id: String::from(cipher_id),
            user_key: to_vault_user_key_material(&user_key),
        })
        .await
        .map_err(|error| log_command_error("vault_get_cipher_detail", error))?;
    let cipher = mapping::to_vault_cipher_detail_dto(cipher)
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

fn to_vault_user_key_material(user_key: &VaultUserKey) -> VaultUserKeyMaterial {
    VaultUserKeyMaterial {
        enc_key: user_key.enc_key.clone(),
        mac_key: user_key.mac_key.clone(),
    }
}

fn vault_user_key_to_biometric_bundle(
    account_id: &str,
    user_key: &VaultUserKey,
) -> Result<biometric_store::BiometricUnlockBundle, AppError> {
    vault_crypto::validate_key_lengths(&user_key.enc_key, user_key.mac_key.as_deref())?;
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
    let enc_key =
        vault_crypto::decode_base64_flexible(&bundle.enc_key_b64, "biometric.enc_key_b64")?;
    let mac_key = bundle
        .mac_key_b64
        .as_ref()
        .map(|value| vault_crypto::decode_base64_flexible(value, "biometric.mac_key_b64"))
        .transpose()?;
    vault_crypto::validate_key_lengths(&enc_key, mac_key.as_deref())?;
    Ok(VaultUserKey { enc_key, mac_key })
}
