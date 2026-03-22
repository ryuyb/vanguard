use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::application::dto::vault::{EnablePinUnlockCommand, UnlockVaultCommand};
use crate::application::use_cases::master_password_unlock_use_case::MasterPasswordUnlockUseCase;
use crate::application::use_cases::unlock_vault_use_case::UnlockVaultUseCase;
use crate::application::use_cases::vault_biometric_use_case::VaultBiometricUseCase;
use crate::application::use_cases::vault_pin_use_case::VaultPinUseCase;
use crate::bootstrap::app_state::AppState;
use crate::domain::unlock::{PinLockType, UnlockMethod};
use crate::interfaces::tauri::dto::vault::{
    VaultBiometricStatusResponseDto, VaultDisableBiometricUnlockRequestDto,
    VaultDisablePinUnlockRequestDto, VaultEnableBiometricUnlockRequestDto,
    VaultEnablePinUnlockRequestDto, VaultLockRequestDto, VaultPinLockTypeDto,
    VaultPinStatusResponseDto, VaultUnlockMethodDto, VaultUnlockRequestDto,
};
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: &AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::vault",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

fn build_unlock_use_case(state: &AppState) -> UnlockVaultUseCase {
    UnlockVaultUseCase::new(
        Arc::new(MasterPasswordUnlockUseCase::new(
            state.master_password_unlock_data_port(),
        )),
        Arc::new(VaultPinUseCase::new(state.pin_unlock_port())),
        Arc::new(VaultBiometricUseCase::new(
            state.master_password_unlock_data_port(),
            state.biometric_unlock_port(),
        )),
    )
}

#[tauri::command]
#[specta::specta]
pub async fn has_local_unlock_data(state: State<'_, AppState>) -> Result<bool, ErrorPayload> {
    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(
            AppError::ValidationFieldError { .. }
            | AppError::ValidationFormatError { .. }
            | AppError::ValidationRequired { .. },
        ) => return Ok(false),
        Err(error) => {
            return Err(log_command_error("has_local_unlock_data", &error));
        }
    };

    let unlock_data = state
        .master_password_unlock_data_port()
        .load_master_password_unlock_data(&account_id)
        .await
        .map_err(|error| log_command_error("has_local_unlock_data", &error))?;

    Ok(unlock_data.is_some())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    request: VaultUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    let method = request.method.clone();

    let unlock_result = build_unlock_use_case(&state)
        .execute(
            &*state,
            UnlockVaultCommand {
                method: request.method.into(),
            },
        )
        .await
        .map_err(|error| log_command_error("vault_unlock", &error))?;

    // 如果使用主密码解锁，恢复 auth_session
    if let crate::interfaces::tauri::dto::vault::VaultUnlockMethodDto::MasterPassword { password } =
        method
    {
        if let Err(error) =
            crate::interfaces::tauri::session::restore_auth_session_with_master_password(
                &state, &password,
            )
            .await
        {
            log::warn!(
                target: "vanguard::tauri::vault",
                "vault_unlock succeeded but failed to restore auth session: [{}] {}",
                error.code(),
                error.log_message()
            );
        }
    } else {
        // PIN 或生物识别解锁时，使用存储的 refresh_token 恢复 session
        if let Some(refresh_token) = unlock_result.refresh_token {
            if let Err(error) =
                crate::interfaces::tauri::session::restore_auth_session_with_refresh_token(
                    &state,
                    &refresh_token,
                )
                .await
            {
                log::warn!(
                    target: "vanguard::tauri::vault",
                    "vault_unlock with PIN/biometric succeeded but failed to restore auth session: [{}] {}",
                    error.code(),
                    error.log_message()
                );
            }
        } else {
            // 向后兼容：旧数据没有存储 refresh_token
            log::warn!(
                target: "vanguard::tauri::vault",
                "vault unlocked with PIN/biometric but no refresh_token available, API calls may fail. Please re-enable PIN/biometric unlock."
            );
        }
    }

    // 更新托盘菜单状态
    #[cfg(desktop)]
    crate::interfaces::tauri::desktop::tray::TrayFeature::update_lock_menu_state(&app_handle);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_biometric_status(
    state: State<'_, AppState>,
) -> Result<VaultBiometricStatusResponseDto, ErrorPayload> {
    let status = VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .biometric_status(&*state)
    .await
    .map_err(|error| log_command_error("vault_get_biometric_status", &error))?;
    Ok(VaultBiometricStatusResponseDto {
        supported: status.supported,
        enabled: status.enabled,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_can_unlock_with_biometric(
    state: State<'_, AppState>,
) -> Result<bool, ErrorPayload> {
    VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .can_unlock_with_biometric(&*state)
    .await
    .map_err(|error| log_command_error("vault_can_unlock_with_biometric", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_enable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultEnableBiometricUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .enable_biometric_unlock(&*state)
    .map_err(|error| log_command_error("vault_enable_biometric_unlock", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_disable_biometric_unlock(
    state: State<'_, AppState>,
    _request: VaultDisableBiometricUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .disable_biometric_unlock(&*state)
    .map_err(|error| log_command_error("vault_disable_biometric_unlock", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_pin_status(
    state: State<'_, AppState>,
) -> Result<VaultPinStatusResponseDto, ErrorPayload> {
    let status = VaultPinUseCase::new(state.pin_unlock_port())
        .pin_status(&*state)
        .await
        .map_err(|error| log_command_error("vault_get_pin_status", &error))?;

    Ok(VaultPinStatusResponseDto {
        supported: status.supported,
        enabled: status.enabled,
        lock_type: status.lock_type.into(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_enable_pin_unlock(
    state: State<'_, AppState>,
    request: VaultEnablePinUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    VaultPinUseCase::new(state.pin_unlock_port())
        .enable_pin_unlock(
            &*state,
            EnablePinUnlockCommand {
                pin: request.pin,
                lock_type: request.lock_type.into(),
            },
        )
        .await
        .map_err(|error| log_command_error("vault_enable_pin_unlock", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_disable_pin_unlock(
    state: State<'_, AppState>,
    _request: VaultDisablePinUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    VaultPinUseCase::new(state.pin_unlock_port())
        .disable_pin_unlock(&*state)
        .await
        .map_err(|error| log_command_error("vault_disable_pin_unlock", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_lock(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    _request: VaultLockRequestDto,
) -> Result<(), ErrorPayload> {
    // Use unlock_manager to properly lock the vault and update state
    state
        .unlock_manager()
        .lock()
        .await
        .map_err(|error| log_command_error("vault_lock", &error))?;

    // 更新托盘菜单状态
    #[cfg(desktop)]
    crate::interfaces::tauri::desktop::tray::TrayFeature::update_lock_menu_state(&app_handle);

    Ok(())
}

impl From<VaultUnlockMethodDto> for UnlockMethod {
    fn from(method: VaultUnlockMethodDto) -> Self {
        match method {
            VaultUnlockMethodDto::MasterPassword { password } => {
                UnlockMethod::MasterPassword { password }
            }
            VaultUnlockMethodDto::Pin { pin } => UnlockMethod::Pin { pin },
            VaultUnlockMethodDto::Biometric => UnlockMethod::Biometric,
        }
    }
}

impl From<VaultPinLockTypeDto> for PinLockType {
    fn from(lock_type: VaultPinLockTypeDto) -> Self {
        match lock_type {
            VaultPinLockTypeDto::Disabled => PinLockType::Disabled,
            VaultPinLockTypeDto::Ephemeral => PinLockType::Ephemeral,
            VaultPinLockTypeDto::Persistent => PinLockType::Persistent,
        }
    }
}

impl From<PinLockType> for VaultPinLockTypeDto {
    fn from(lock_type: PinLockType) -> Self {
        match lock_type {
            PinLockType::Disabled => VaultPinLockTypeDto::Disabled,
            PinLockType::Ephemeral => VaultPinLockTypeDto::Ephemeral,
            PinLockType::Persistent => VaultPinLockTypeDto::Persistent,
        }
    }
}
