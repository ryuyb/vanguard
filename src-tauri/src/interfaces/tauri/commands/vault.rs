use std::sync::Arc;

use tauri::State;

use crate::application::dto::vault::{
    CopyCipherFieldCommand, EnablePinUnlockCommand, GetCipherDetailQuery, GetCipherTotpCodeCommand,
    UnlockVaultCommand, VaultCopyField, VaultUserKeyMaterial,
};
use crate::application::use_cases::copy_cipher_field_use_case::CopyCipherFieldUseCase;
use crate::application::use_cases::get_cipher_totp_code_use_case::GetCipherTotpCodeUseCase;
use crate::application::use_cases::get_vault_view_data_use_case::GetVaultViewDataUseCase;
use crate::application::use_cases::master_password_unlock_use_case::MasterPasswordUnlockUseCase;
use crate::application::use_cases::unlock_vault_use_case::UnlockVaultUseCase;
use crate::application::use_cases::vault_biometric_use_case::VaultBiometricUseCase;
use crate::application::use_cases::vault_pin_use_case::VaultPinUseCase;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::domain::unlock::{PinLockType, UnlockMethod};
use crate::interfaces::tauri::dto::vault::{
    VaultBiometricStatusResponseDto, VaultCipherDetailRequestDto, VaultCipherDetailResponseDto,
    VaultCipherItemDto, VaultCipherTotpCodeRequestDto, VaultCipherTotpCodeResponseDto,
    VaultCopyCipherFieldRequestDto, VaultCopyCipherFieldResponseDto, VaultCopyFieldDto,
    VaultDisableBiometricUnlockRequestDto, VaultDisablePinUnlockRequestDto,
    VaultEnableBiometricUnlockRequestDto, VaultEnablePinUnlockRequestDto, VaultFolderItemDto,
    VaultLockRequestDto, VaultPinLockTypeDto, VaultPinStatusResponseDto, VaultUnlockMethodDto,
    VaultUnlockRequestDto, VaultViewDataResponseDto,
};
use crate::interfaces::tauri::mapping;
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
pub async fn vault_can_unlock(state: State<'_, AppState>) -> Result<bool, ErrorPayload> {
    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(
            AppError::ValidationFieldError { .. }
            | AppError::ValidationFormatError { .. }
            | AppError::ValidationRequired { .. },
        ) => return Ok(false),
        Err(error) => {
            return Err(log_command_error("vault_can_unlock", &error));
        }
    };

    let unlock_data = state
        .master_password_unlock_data_port()
        .load_master_password_unlock_data(&account_id)
        .await
        .map_err(|error| log_command_error("vault_can_unlock", &error))?;

    Ok(unlock_data.is_some())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_is_unlocked(state: State<'_, AppState>) -> Result<bool, ErrorPayload> {
    let account_id = match state.active_account_id() {
        Ok(value) => value,
        Err(
            AppError::ValidationFieldError { .. }
            | AppError::ValidationFormatError { .. }
            | AppError::ValidationRequired { .. },
        ) => return Ok(false),
        Err(error) => {
            return Err(log_command_error("vault_is_unlocked", &error));
        }
    };

    state
        .get_vault_user_key(&account_id)
        .map(|value| value.is_some())
        .map_err(|error| log_command_error("vault_is_unlocked", &error))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_unlock(
    state: State<'_, AppState>,
    request: VaultUnlockRequestDto,
) -> Result<(), ErrorPayload> {
    build_unlock_use_case(&state)
        .execute(
            &*state,
            UnlockVaultCommand {
                method: request.method.into(),
            },
        )
        .await
        .map_err(|error| log_command_error("vault_unlock", &error))?;

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
    .map_err(|error| { log_command_error("vault_get_biometric_status", &error) })?;
    Ok(VaultBiometricStatusResponseDto {
        supported: status.supported,
        enabled: status.enabled,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_can_unlock_with_biometric(state: State<'_, AppState>) -> Result<bool, ErrorPayload> {
    VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .can_unlock_with_biometric(&*state)
    .await
    .map_err(|error| { log_command_error("vault_can_unlock_with_biometric", &error) })
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
    .map_err(|error| { log_command_error("vault_enable_biometric_unlock", &error) })
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
    .map_err(|error| { log_command_error("vault_disable_biometric_unlock", &error) })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_pin_status(
    state: State<'_, AppState>,
) -> Result<VaultPinStatusResponseDto, ErrorPayload> {
    let status = VaultPinUseCase::new(state.pin_unlock_port())
        .pin_status(&*state)
        .await
        .map_err(|error| { log_command_error("vault_get_pin_status", &error) })?;

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
        .map_err(|error| { log_command_error("vault_enable_pin_unlock", &error) })
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
        .map_err(|error| { log_command_error("vault_disable_pin_unlock", &error) })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_lock(
    state: State<'_, AppState>,
    _request: VaultLockRequestDto,
) -> Result<(), ErrorPayload> {
    VaultBiometricUseCase::new(
        state.master_password_unlock_data_port(),
        state.biometric_unlock_port(),
    )
    .lock(&*state)
    .map_err(|error| { log_command_error("vault_lock", &error) })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_view_data(
    state: State<'_, AppState>,
) -> Result<VaultViewDataResponseDto, ErrorPayload> {
    let view_data = GetVaultViewDataUseCase::new(state.sync_service())
        .execute(&*state)
        .await
        .map_err(|error| { log_command_error("vault_get_view_data", &error) })?;

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
                uris: cipher.uris,
                favorite: cipher.favorite,
                creation_date: cipher.creation_date,
                revision_date: cipher.revision_date,
                deleted_date: cipher.deleted_date,
                attachment_count: cipher.attachment_count,
            })
            .collect(),
        total_ciphers: view_data.total_ciphers,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_cipher_detail(
    state: State<'_, AppState>,
    request: VaultCipherDetailRequestDto,
) -> Result<VaultCipherDetailResponseDto, ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| { log_command_error("vault_get_cipher_detail", &error) })?;
    let cipher_id = request.cipher_id.trim();
    if cipher_id.is_empty() {
        return Err(log_command_error(
            "vault_get_cipher_detail",
            &AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "cipher_id cannot be empty".to_string(),
            },
        ));
    }

    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| { log_command_error("vault_get_cipher_detail", &error) })?
        .ok_or_else(|| {
            log_command_error(
                "vault_get_cipher_detail",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock with master password first"
                        .to_string(),
                },
            )
        })?;

    let cipher = state
        .get_cipher_detail_use_case()
        .execute(GetCipherDetailQuery {
            account_id: account_id.clone(),
            cipher_id: String::from(cipher_id),
            user_key: (&user_key).into(),
        })
        .await
        .map_err(|error| { log_command_error("vault_get_cipher_detail", &error) })?;
    let cipher = mapping::to_vault_cipher_detail_dto(cipher)
        .map_err(|error| { log_command_error("vault_get_cipher_detail", &error) })?;

    Ok(VaultCipherDetailResponseDto { account_id, cipher })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_copy_cipher_field(
    state: State<'_, AppState>,
    request: VaultCopyCipherFieldRequestDto,
) -> Result<VaultCopyCipherFieldResponseDto, ErrorPayload> {
    let result =
        CopyCipherFieldUseCase::new(state.get_cipher_detail_use_case(), state.clipboard_port())
            .execute(
                &*state,
                CopyCipherFieldCommand {
                    cipher_id: request.cipher_id,
                    field: request.field.into(),
                    clear_after_ms: request.clear_after_ms,
                },
            )
            .await
            .map_err(|error| { log_command_error("vault_copy_cipher_field", &error) })?;

    Ok(VaultCopyCipherFieldResponseDto {
        copied: result.copied,
        clear_after_ms: result.clear_after_ms,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_cipher_totp_code(
    state: State<'_, AppState>,
    request: VaultCipherTotpCodeRequestDto,
) -> Result<VaultCipherTotpCodeResponseDto, ErrorPayload> {
    let result = GetCipherTotpCodeUseCase::new(state.get_cipher_detail_use_case())
        .execute(
            &*state,
            GetCipherTotpCodeCommand {
                cipher_id: request.cipher_id,
            },
        )
        .await
        .map_err(|error| { log_command_error("vault_get_cipher_totp_code", &error) })?;

    Ok(VaultCipherTotpCodeResponseDto {
        code: result.code,
        period_seconds: result.period_seconds,
        remaining_seconds: result.remaining_seconds,
        expires_at_ms: result.expires_at_ms,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_icon_server(state: State<'_, AppState>) -> Result<String, ErrorPayload> {
    let session = state
        .auth_session()
        .map_err(|error| { log_command_error("vault_get_icon_server", &error) })?;

    let base_url = match session {
        Some(session) => session.base_url,
        None => {
            let context = state
                .persisted_auth_context()
                .map_err(|error| { log_command_error("vault_get_icon_server", &error) })?
                .ok_or_else(|| {
                    log_command_error(
                        "vault_get_icon_server",
                        &AppError::ValidationFieldError {
                            field: "unknown".to_string(),
                            message: "no authenticated session or persisted context found"
                                .to_string(),
                        },
                    )
                })?;
            context.base_url
        }
    };

    let normalized_url = base_url.trim().to_lowercase();

    if normalized_url.contains("bitwarden.com") || normalized_url.contains("bitwarden.eu") {
        Ok(String::from("https://icons.bitwarden.net"))
    } else {
        Ok(base_url)
    }
}

impl From<&VaultUserKey> for VaultUserKeyMaterial {
    fn from(user_key: &VaultUserKey) -> Self {
        Self {
            enc_key: user_key.enc_key.clone(),
            mac_key: user_key.mac_key.clone(),
        }
    }
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

impl From<VaultCopyFieldDto> for VaultCopyField {
    fn from(field: VaultCopyFieldDto) -> Self {
        match field {
            VaultCopyFieldDto::Username => VaultCopyField::Username,
            VaultCopyFieldDto::Password => VaultCopyField::Password,
            VaultCopyFieldDto::Totp => VaultCopyField::Totp,
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
