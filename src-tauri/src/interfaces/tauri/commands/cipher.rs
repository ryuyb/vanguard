use tauri::State;

use crate::application::dto::vault::{
    CopyCipherFieldCommand, GetCipherDetailQuery, GetCipherTotpCodeCommand, VaultCopyField,
    VaultUserKeyMaterial,
};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::copy_cipher_field_use_case::CopyCipherFieldUseCase;
use crate::application::use_cases::get_cipher_totp_code_use_case::GetCipherTotpCodeUseCase;
use crate::application::use_cases::get_vault_view_data_use_case::GetVaultViewDataUseCase;
use crate::application::use_cases::list_ciphers_use_case::ListCiphersUseCase;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::interfaces::tauri::dto::cipher::{
    CipherMutationResponseDto, CreateCipherRequestDto, DeleteCipherRequestDto,
    RestoreCipherRequestDto, SoftDeleteCipherRequestDto, UpdateCipherRequestDto,
    VaultCipherDetailRequestDto, VaultCipherDetailResponseDto, VaultCipherItemDto,
    VaultCipherTotpCodeRequestDto, VaultCipherTotpCodeResponseDto, VaultCopyCipherFieldRequestDto,
    VaultCopyCipherFieldResponseDto, VaultCopyFieldDto, VaultExecuteAutofillRequestDto,
    VaultExecuteAutofillResponseDto, VaultFolderItemDto, VaultViewDataResponseDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: &AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::cipher",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_view_data(
    state: State<'_, AppState>,
) -> Result<VaultViewDataResponseDto, ErrorPayload> {
    let view_data = GetVaultViewDataUseCase::new(state.sync_service())
        .execute(&*state)
        .await
        .map_err(|error| log_command_error("vault_get_view_data", &error))?;

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
pub async fn vault_list_ciphers(
    state: State<'_, AppState>,
) -> Result<Vec<VaultCipherItemDto>, ErrorPayload> {
    let ciphers = ListCiphersUseCase::new(state.sync_service())
        .execute(&*state)
        .await
        .map_err(|error| log_command_error("vault_list_ciphers", &error))?;

    Ok(ciphers
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
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn vault_get_cipher_detail(
    state: State<'_, AppState>,
    request: VaultCipherDetailRequestDto,
) -> Result<VaultCipherDetailResponseDto, ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("vault_get_cipher_detail", &error))?;
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
        .map_err(|error| log_command_error("vault_get_cipher_detail", &error))?
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
        .map_err(|error| log_command_error("vault_get_cipher_detail", &error))?;
    let cipher = mapping::to_vault_cipher_detail_dto(cipher)
        .map_err(|error| log_command_error("vault_get_cipher_detail", &error))?;

    Ok(VaultCipherDetailResponseDto { account_id, cipher })
}

#[tauri::command]
#[specta::specta]
pub async fn vault_copy_cipher_field(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    request: VaultCopyCipherFieldRequestDto,
) -> Result<VaultCopyCipherFieldResponseDto, ErrorPayload> {
    use crate::bootstrap::config::AppConfig;

    // First, get the value to copy
    let result =
        CopyCipherFieldUseCase::new(state.get_cipher_detail_use_case(), state.clipboard_port())
            .execute(
                &*state,
                CopyCipherFieldCommand {
                    cipher_id: request.cipher_id.clone(),
                    field: request.field.into(),
                    clear_after_ms: request.clear_after_ms,
                },
            )
            .await
            .map_err(|error| log_command_error("vault_copy_cipher_field", &error))?;

    // Check if autofill is enabled
    let autofill_enabled = match AppConfig::load(&app_handle) {
        Ok(config) => config.spotlight_autofill,
        Err(error) => {
            log::warn!(
                target: "vanguard::tauri::cipher",
                "Failed to load config for autofill check: {}",
                error.log_message()
            );
            false
        }
    };

    // Attempt autofill if enabled
    let mut autofill_performed = false;
    let mut autofill_value: Option<String> = None;
    if autofill_enabled && result.copied {
        // Get the decrypted value for autofill
        if let Ok(account_id) = state.active_account_id() {
            if let Ok(Some(user_key)) = state.get_vault_user_key_material(&account_id) {
                if let Ok(cipher) = state
                    .get_cipher_detail_use_case()
                    .execute(GetCipherDetailQuery {
                        account_id,
                        cipher_id: request.cipher_id.clone(),
                        user_key,
                    })
                    .await
                {
                    if let Ok(value) = resolve_field_value(&cipher, request.field.into()) {
                        autofill_performed = true;
                        autofill_value = Some(value);
                    }
                }
            }
        }
    }

    Ok(VaultCopyCipherFieldResponseDto {
        copied: result.copied,
        clear_after_ms: result.clear_after_ms,
        autofill_performed,
        value: autofill_value,
    })
}

/// Execute autofill with the stored value
/// This should be called by frontend after spotlight is hidden
#[tauri::command]
#[specta::specta]
pub async fn vault_execute_autofill(
    state: State<'_, AppState>,
    request: VaultExecuteAutofillRequestDto,
) -> Result<VaultExecuteAutofillResponseDto, ErrorPayload> {
    log::info!(
        target: "vanguard::autofill",
        "Executing autofill with value length: {}",
        request.value.len()
    );

    // Small delay to ensure spotlight is fully closed and focus has returned
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    let success = perform_autofill(&state, &request.value).unwrap_or(false);

    Ok(VaultExecuteAutofillResponseDto { success })
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
        .map_err(|error| log_command_error("vault_get_cipher_totp_code", &error))?;

    Ok(VaultCipherTotpCodeResponseDto {
        code: result.code,
        period_seconds: result.period_seconds,
        remaining_seconds: result.remaining_seconds,
        expires_at_ms: result.expires_at_ms,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn create_cipher(
    request: CreateCipherRequestDto,
    state: State<'_, AppState>,
) -> Result<CipherMutationResponseDto, ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("create_cipher", &error))?;

    let session = state
        .auth_session()
        .map_err(|error| log_command_error("create_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "create_cipher",
                &AppError::ValidationRequired {
                    field: "session".to_string(),
                },
            )
        })?;

    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("create_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "create_cipher",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock first".to_string(),
                },
            )
        })?;

    let result = state
        .create_cipher_use_case()
        .execute(
            account_id,
            session.base_url,
            session.access_token,
            request.cipher,
            (&user_key).into(),
        )
        .await
        .map_err(|error| log_command_error("create_cipher", &error))?;

    Ok(CipherMutationResponseDto {
        cipher_id: result.cipher_id,
        revision_date: result.revision_date,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn update_cipher(
    request: UpdateCipherRequestDto,
    state: State<'_, AppState>,
) -> Result<CipherMutationResponseDto, ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("update_cipher", &error))?;

    let session = state
        .auth_session()
        .map_err(|error| log_command_error("update_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "update_cipher",
                &AppError::ValidationRequired {
                    field: "session".to_string(),
                },
            )
        })?;

    let user_key = state
        .get_vault_user_key(&account_id)
        .map_err(|error| log_command_error("update_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "update_cipher",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock first".to_string(),
                },
            )
        })?;

    let result = state
        .update_cipher_use_case()
        .execute(
            account_id,
            session.base_url,
            session.access_token,
            request.cipher_id,
            request.cipher,
            (&user_key).into(),
        )
        .await
        .map_err(|error| log_command_error("update_cipher", &error))?;

    Ok(CipherMutationResponseDto {
        cipher_id: result.cipher_id,
        revision_date: result.revision_date,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn delete_cipher(
    request: DeleteCipherRequestDto,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("delete_cipher", &error))?;

    let session = state
        .auth_session()
        .map_err(|error| log_command_error("delete_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "delete_cipher",
                &AppError::ValidationRequired {
                    field: "session".to_string(),
                },
            )
        })?;

    state
        .delete_cipher_use_case()
        .execute(
            account_id,
            session.base_url,
            session.access_token,
            request.cipher_id,
        )
        .await
        .map_err(|error| log_command_error("delete_cipher", &error))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn soft_delete_cipher(
    request: SoftDeleteCipherRequestDto,
    state: State<'_, AppState>,
) -> Result<CipherMutationResponseDto, ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("soft_delete_cipher", &error))?;

    let session = state
        .auth_session()
        .map_err(|error| log_command_error("soft_delete_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "soft_delete_cipher",
                &AppError::ValidationRequired {
                    field: "session".to_string(),
                },
            )
        })?;

    let result = state
        .soft_delete_cipher_use_case()
        .execute(
            account_id,
            session.base_url,
            session.access_token,
            request.cipher_id,
        )
        .await
        .map_err(|error| log_command_error("soft_delete_cipher", &error))?;

    Ok(CipherMutationResponseDto {
        cipher_id: result.cipher_id,
        revision_date: result.revision_date,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn restore_cipher(
    request: RestoreCipherRequestDto,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let account_id = state
        .active_account_id()
        .map_err(|error| log_command_error("restore_cipher", &error))?;

    let session = state
        .auth_session()
        .map_err(|error| log_command_error("restore_cipher", &error))?
        .ok_or_else(|| {
            log_command_error(
                "restore_cipher",
                &AppError::ValidationRequired {
                    field: "session".to_string(),
                },
            )
        })?;

    state
        .restore_cipher_use_case()
        .execute(
            account_id,
            session.base_url,
            session.access_token,
            request.cipher_id,
        )
        .await
        .map_err(|error| log_command_error("restore_cipher", &error))?;

    Ok(())
}

impl From<&VaultUserKey> for VaultUserKeyMaterial {
    fn from(user_key: &VaultUserKey) -> Self {
        Self {
            enc_key: user_key.enc_key.clone(),
            mac_key: user_key.mac_key.clone(),
        }
    }
}

impl From<VaultCopyFieldDto> for VaultCopyField {
    fn from(field: VaultCopyFieldDto) -> Self {
        match field {
            VaultCopyFieldDto::Username => VaultCopyField::Username,
            VaultCopyFieldDto::Password => VaultCopyField::Password,
            VaultCopyFieldDto::Totp => VaultCopyField::Totp,
            VaultCopyFieldDto::Notes => VaultCopyField::Notes,
            VaultCopyFieldDto::CustomField { index } => VaultCopyField::CustomField { index },
            VaultCopyFieldDto::Uri { index } => VaultCopyField::Uri { index },
            VaultCopyFieldDto::CardNumber => VaultCopyField::CardNumber,
            VaultCopyFieldDto::CardCode => VaultCopyField::CardCode,
            VaultCopyFieldDto::Email => VaultCopyField::Email,
            VaultCopyFieldDto::Phone => VaultCopyField::Phone,
            VaultCopyFieldDto::SshPrivateKey => VaultCopyField::SshPrivateKey,
            VaultCopyFieldDto::SshPublicKey => VaultCopyField::SshPublicKey,
        }
    }
}

/// Resolve the field value from cipher detail for autofill
fn resolve_field_value(
    cipher: &crate::application::dto::vault::VaultCipherDetail,
    field: VaultCopyField,
) -> Result<String, AppError> {
    use crate::application::dto::vault::VaultCopyField;
    use crate::application::totp::{current_unix_seconds, generate_current_totp};

    match field {
        VaultCopyField::Username => cipher
            .login
            .as_ref()
            .and_then(|login| login.username.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "username".to_string(),
                message: "No username available".to_string(),
            }),
        VaultCopyField::Password => cipher
            .login
            .as_ref()
            .and_then(|login| login.password.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "password".to_string(),
                message: "No password available".to_string(),
            }),
        VaultCopyField::Totp => {
            let unix_seconds = current_unix_seconds()?;
            cipher
                .login
                .as_ref()
                .and_then(|login| login.totp.clone())
                .and_then(|totp| generate_current_totp(&totp, unix_seconds).ok())
                .map(|snapshot| snapshot.code)
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "totp".to_string(),
                    message: "No TOTP available or failed to generate code".to_string(),
                })
        }
        VaultCopyField::Notes => {
            cipher
                .notes
                .clone()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "notes".to_string(),
                    message: "No notes available".to_string(),
                })
        }
        VaultCopyField::CustomField { index } => cipher
            .fields
            .get(index)
            .and_then(|f| f.value.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: format!("custom_field_{}", index),
                message: "Custom field not found or empty".to_string(),
            }),
        VaultCopyField::Uri { index } => cipher
            .login
            .as_ref()
            .and_then(|login| login.uris.get(index))
            .and_then(|u| u.uri.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: format!("uri_{}", index),
                message: "URI not found or empty".to_string(),
            }),
        VaultCopyField::CardNumber => cipher
            .card
            .as_ref()
            .and_then(|card| card.number.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "card_number".to_string(),
                message: "No card number available".to_string(),
            }),
        VaultCopyField::CardCode => cipher
            .card
            .as_ref()
            .and_then(|card| card.code.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "card_code".to_string(),
                message: "No card code available".to_string(),
            }),
        VaultCopyField::Email => cipher
            .login
            .as_ref()
            .and_then(|login| login.username.clone())
            .or_else(|| cipher.identity.as_ref().and_then(|i| i.email.clone()))
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "email".to_string(),
                message: "No email available".to_string(),
            }),
        VaultCopyField::Phone => cipher
            .identity
            .as_ref()
            .and_then(|i| i.phone.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "phone".to_string(),
                message: "No phone available".to_string(),
            }),
        VaultCopyField::SshPrivateKey => cipher
            .ssh_key
            .as_ref()
            .and_then(|ssh| ssh.private_key.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "ssh_private_key".to_string(),
                message: "No SSH private key available".to_string(),
            }),
        VaultCopyField::SshPublicKey => cipher
            .ssh_key
            .as_ref()
            .and_then(|ssh| ssh.public_key.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "ssh_public_key".to_string(),
                message: "No SSH public key available".to_string(),
            }),
    }
}

/// Perform autofill by typing text into the previously focused input
fn perform_autofill(state: &AppState, text: &str) -> Result<bool, AppError> {
    // Get the focus tracker
    let focus_tracker = state.focus_tracker();
    let tracker = focus_tracker
        .lock()
        .map_err(|_| AppError::InternalUnexpected {
            message: "Failed to lock focus tracker".to_string(),
        })?;

    let focus_info = match tracker.get_valid_focus() {
        Some(info) => info.clone(),
        None => {
            log::debug!(
                target: "vanguard::autofill",
                "No valid focus info available, skipping autofill"
            );
            return Ok(false);
        }
    };

    // Check if the focused element is a text input
    if !focus_info.is_text_input {
        log::debug!(
            target: "vanguard::autofill",
            "Focused element is not a text input, skipping autofill"
        );
        return Ok(false);
    }

    // Note: We assume the previous window is already focused
    // because Spotlight has been closed by the frontend before calling this function
    log::info!(
        target: "vanguard::autofill",
        "Filling into app: {} (pid: {})",
        focus_info.app_bundle_id,
        focus_info.pid
    );

    // Use text injection port to type the text
    let text_injection = state.text_injection_port();
    if !text_injection.is_available() {
        log::warn!(
            target: "vanguard::autofill",
            "Text injection not available on this platform"
        );
        return Ok(false);
    }

    match text_injection.type_text(text) {
        Ok(()) => {
            log::info!(
                target: "vanguard::autofill",
                "Successfully autofilled text into app: {}",
                focus_info.app_bundle_id
            );
            Ok(true)
        }
        Err(error) => {
            log::warn!(
                target: "vanguard::autofill",
                "Failed to autofill text: [{}] {}",
                error.code(),
                error.log_message()
            );
            Ok(false)
        }
    }
}
