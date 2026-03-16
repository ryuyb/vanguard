use tauri::State;

use crate::application::dto::vault::{
    CopyCipherFieldCommand, GetCipherDetailQuery, GetCipherTotpCodeCommand, VaultCopyField,
    VaultUserKeyMaterial,
};
use crate::application::use_cases::copy_cipher_field_use_case::CopyCipherFieldUseCase;
use crate::application::use_cases::get_cipher_totp_code_use_case::GetCipherTotpCodeUseCase;
use crate::application::use_cases::get_vault_view_data_use_case::GetVaultViewDataUseCase;
use crate::application::use_cases::list_ciphers_use_case::ListCiphersUseCase;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::interfaces::tauri::dto::cipher::{
    CipherMutationResponseDto, CreateCipherRequestDto, DeleteCipherRequestDto,
    RestoreCipherRequestDto, SoftDeleteCipherRequestDto, UpdateCipherRequestDto,
    VaultCipherDetailRequestDto, VaultCipherDetailResponseDto, VaultCipherItemDto,
    VaultCipherTotpCodeRequestDto, VaultCipherTotpCodeResponseDto,
    VaultCopyCipherFieldRequestDto, VaultCopyCipherFieldResponseDto, VaultCopyFieldDto,
    VaultFolderItemDto, VaultViewDataResponseDto,
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
            .map_err(|error| log_command_error("vault_copy_cipher_field", &error))?;

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
pub async fn vault_get_icon_server(state: State<'_, AppState>) -> Result<String, ErrorPayload> {
    let session = state
        .auth_session()
        .map_err(|error| log_command_error("vault_get_icon_server", &error))?;

    let base_url = match session {
        Some(session) => session.base_url,
        None => {
            let context = state
                .persisted_auth_context()
                .map_err(|error| log_command_error("vault_get_icon_server", &error))?
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
