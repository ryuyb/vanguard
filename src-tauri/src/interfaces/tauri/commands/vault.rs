use std::sync::Arc;

use tauri::State;

use crate::application::dto::vault::{
    GetCipherDetailQuery, GetVaultViewDataQuery, UnlockVaultCommand, VaultUserKeyMaterial,
};
use crate::application::use_cases::get_vault_view_data_use_case::GetVaultViewDataUseCase;
use crate::application::use_cases::unlock_vault_with_password_use_case::{
    UnlockVaultWithPasswordUseCase,
};
use crate::application::use_cases::unlock_vault_use_case::UnlockVaultUseCase;
use crate::application::use_cases::vault_biometric_use_case::VaultBiometricUseCase;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::domain::unlock::UnlockMethod;
use crate::interfaces::tauri::dto::vault::{
    VaultBiometricStatusResponseDto, VaultCipherDetailRequestDto, VaultCipherDetailResponseDto,
    VaultCipherItemDto, VaultDisableBiometricUnlockRequestDto,
    VaultEnableBiometricUnlockRequestDto, VaultFolderItemDto, VaultLockRequestDto,
    VaultUnlockWithPasswordRequestDto, VaultViewDataRequestDto, VaultViewDataResponseDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;

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

    let unlock_data = state
        .master_password_unlock_data_port()
        .load_master_password_unlock_data(&account_id)
        .await
        .map_err(|error| log_command_error("vault_can_unlock", error))?;

    Ok(unlock_data.is_some())
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
    UnlockVaultUseCase::with_master_password_executor(Arc::new(
        UnlockVaultWithPasswordUseCase::new(state.master_password_unlock_data_port()),
    ))
        .execute(
            &*state,
            UnlockVaultCommand {
                method: UnlockMethod::MasterPassword {
                    password: master_password,
                },
            },
        )
        .await
        .map_err(|error| log_command_error("vault_unlock_with_password", error))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_biometric_status(
    state: State<'_, AppState>,
) -> Result<VaultBiometricStatusResponseDto, String> {
    let status = VaultBiometricUseCase::new(state.sync_service(), state.biometric_unlock_port())
        .biometric_status(&*state)
        .await
        .map_err(|error| log_command_error("vault_get_biometric_status", error))?;
    Ok(VaultBiometricStatusResponseDto {
        supported: status.supported,
        enabled: status.enabled,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_can_unlock_with_biometric(state: State<'_, AppState>) -> Result<bool, String> {
    VaultBiometricUseCase::new(state.sync_service(), state.biometric_unlock_port())
        .can_unlock_with_biometric(&*state)
        .await
        .map_err(|error| log_command_error("vault_can_unlock_with_biometric", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_enable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultEnableBiometricUnlockRequestDto,
) -> Result<(), String> {
    VaultBiometricUseCase::new(state.sync_service(), state.biometric_unlock_port())
        .enable_biometric_unlock(&*state)
        .map_err(|error| log_command_error("vault_enable_biometric_unlock", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_disable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultDisableBiometricUnlockRequestDto,
) -> Result<(), String> {
    VaultBiometricUseCase::new(state.sync_service(), state.biometric_unlock_port())
        .disable_biometric_unlock(&*state)
        .map_err(|error| log_command_error("vault_disable_biometric_unlock", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock_with_biometric(state: State<'_, AppState>) -> Result<(), String> {
    UnlockVaultUseCase::with_master_and_biometric_executor(
        Arc::new(UnlockVaultWithPasswordUseCase::new(
            state.master_password_unlock_data_port(),
        )),
        Arc::new(VaultBiometricUseCase::new(
            state.sync_service(),
            state.biometric_unlock_port(),
        )),
    )
        .execute(
            &*state,
            UnlockVaultCommand {
                method: UnlockMethod::Biometric,
            },
        )
        .await
        .map_err(|error| log_command_error("vault_unlock_with_biometric", error))
        .map(|_| ())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_lock(
    state: State<'_, AppState>,
    _request: VaultLockRequestDto,
) -> Result<(), String> {
    VaultBiometricUseCase::new(state.sync_service(), state.biometric_unlock_port())
        .lock(&*state)
        .map_err(|error| log_command_error("vault_lock", error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_view_data(
    state: State<'_, AppState>,
    request: VaultViewDataRequestDto,
) -> Result<VaultViewDataResponseDto, String> {
    let view_data = GetVaultViewDataUseCase::new(state.sync_service())
        .execute(
            &*state,
            GetVaultViewDataQuery {
                page: request.page,
                page_size: request.page_size,
            },
        )
        .await
        .map_err(|error| log_command_error("vault_get_view_data", error))?;

    Ok(VaultViewDataResponseDto {
        account_id: view_data.account_id,
        sync_status: mapping::to_sync_status_response_dto(
            view_data.sync_context,
            Some(view_data.sync_metrics),
        ),
        folders: view_data
            .folders
            .into_iter()
            .map(|folder| VaultFolderItemDto {
                id: folder.id,
                name: folder.name,
            })
            .collect(),
        ciphers: view_data
            .ciphers
            .into_iter()
            .map(|cipher| VaultCipherItemDto {
                id: cipher.id,
                folder_id: cipher.folder_id,
                organization_id: cipher.organization_id,
                r#type: cipher.r#type,
                name: cipher.name,
                username: cipher.username,
                creation_date: cipher.creation_date,
                revision_date: cipher.revision_date,
                deleted_date: cipher.deleted_date,
                attachment_count: cipher.attachment_count,
            })
            .collect(),
        total_ciphers: view_data.total_ciphers,
        page: view_data.page,
        page_size: view_data.page_size,
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

fn to_vault_user_key_material(user_key: &VaultUserKey) -> VaultUserKeyMaterial {
    VaultUserKeyMaterial {
        enc_key: user_key.enc_key.clone(),
        mac_key: user_key.mac_key.clone(),
    }
}
