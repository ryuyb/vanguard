use specta::specta;
use tauri::State;

use crate::application::dto::sync::SyncVaultCommand;
use crate::application::dto::vault::{
    CreateFolderRequest, DeleteFolderRequest, RenameFolderRequest,
};
use crate::application::vault_crypto;
use crate::bootstrap::app_state::AppState;
use crate::domain::sync::SyncTrigger;
use crate::interfaces::tauri::dto::folder::FolderDto;
use crate::interfaces::tauri::session;
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: &AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::folder",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

#[specta]
#[tauri::command]
pub async fn list_folders(state: State<'_, AppState>) -> Result<Vec<FolderDto>, ErrorPayload> {
    // 获取当前 session
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("list_folders", &error))?;

    // 获取 vault_user_key 用于解密
    let user_key = state
        .get_vault_user_key(&auth_session.account_id)
        .await
        .map_err(|error| log_command_error("list_folders", &error))?
        .ok_or_else(|| {
            log_command_error(
                "list_folders",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock first".to_string(),
                },
            )
        })?;

    // 从本地数据库获取 folders
    let folders = state
        .sync_service()
        .vault_repository()
        .list_live_folders(&auth_session.account_id)
        .await
        .map_err(|error| log_command_error("list_folders", &error))?;

    // 解密 folder 名称
    let mut result = Vec::new();
    for folder in folders {
        if let Some(encrypted_name) = folder.name {
            match vault_crypto::decrypt_cipher_string(&encrypted_name, &(&user_key).into()) {
                Ok(decrypted_name) => {
                    result.push(FolderDto {
                        id: folder.id,
                        name: decrypted_name,
                    });
                }
                Err(error) => {
                    log::warn!(
                        target: "vanguard::tauri::folder",
                        "failed to decrypt folder name for id={}: {}",
                        folder.id,
                        error
                    );
                    // 跳过无法解密的 folder
                    continue;
                }
            }
        }
    }

    Ok(result)
}

#[specta]
#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    request: CreateFolderRequest,
) -> Result<(), ErrorPayload> {
    // 获取当前 session
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("create_folder", &error))?;

    // 获取 vault_user_key 用于加密
    let user_key = state
        .get_vault_user_key(&auth_session.account_id)
        .await
        .map_err(|error| log_command_error("create_folder", &error))?
        .ok_or_else(|| {
            log_command_error(
                "create_folder",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock first".to_string(),
                },
            )
        })?;

    // 加密文件夹名称
    let encrypted_name = vault_crypto::encrypt_cipher_string(&request.name, &(&user_key).into())
        .map_err(|error| log_command_error("create_folder", &error))?;

    // 调用 Vaultwarden API 创建文件夹
    state
        .vaultwarden_client()
        .create_folder(
            &auth_session.base_url,
            &auth_session.access_token,
            encrypted_name,
        )
        .await
        .map_err(|error| {
            log_command_error(
                "create_folder",
                &AppError::NetworkRemoteError {
                    status: 500,
                    message: error.to_string(),
                },
            )
        })?;

    // 触发 folders-only sync 同步数据到本地数据库
    state
        .sync_service()
        .sync_folders_only(SyncVaultCommand {
            account_id: auth_session.account_id.clone(),
            base_url: auth_session.base_url.clone(),
            access_token: auth_session.access_token.clone(),
            exclude_domains: false,
            trigger: SyncTrigger::Manual,
        })
        .await
        .map_err(|error| log_command_error("create_folder", &error))?;

    Ok(())
}

#[specta]
#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    request: RenameFolderRequest,
) -> Result<(), ErrorPayload> {
    // 获取当前 session
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("rename_folder", &error))?;

    // 获取 vault_user_key 用于加密
    let user_key = state
        .get_vault_user_key(&auth_session.account_id)
        .await
        .map_err(|error| log_command_error("rename_folder", &error))?
        .ok_or_else(|| {
            log_command_error(
                "rename_folder",
                &AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "vault is locked, please unlock first".to_string(),
                },
            )
        })?;

    // 加密新文件夹名称
    let encrypted_name =
        vault_crypto::encrypt_cipher_string(&request.new_name, &(&user_key).into())
            .map_err(|error| log_command_error("rename_folder", &error))?;

    // 调用 Vaultwarden API 更新文件夹
    state
        .vaultwarden_client()
        .update_folder(
            &auth_session.base_url,
            &auth_session.access_token,
            &request.folder_id,
            encrypted_name,
        )
        .await
        .map_err(|error| {
            log_command_error(
                "rename_folder",
                &AppError::NetworkRemoteError {
                    status: 500,
                    message: error.to_string(),
                },
            )
        })?;

    // 触发 folders-only sync 同步数据到本地数据库
    state
        .sync_service()
        .sync_folders_only(SyncVaultCommand {
            account_id: auth_session.account_id.clone(),
            base_url: auth_session.base_url.clone(),
            access_token: auth_session.access_token.clone(),
            exclude_domains: false,
            trigger: SyncTrigger::Manual,
        })
        .await
        .map_err(|error| log_command_error("rename_folder", &error))?;

    Ok(())
}

#[specta]
#[tauri::command]
pub async fn delete_folder(
    state: State<'_, AppState>,
    request: DeleteFolderRequest,
) -> Result<(), ErrorPayload> {
    // 获取当前 session
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("delete_folder", &error))?;

    // 调用 Vaultwarden API 删除文件夹
    state
        .vaultwarden_client()
        .delete_folder(
            &auth_session.base_url,
            &auth_session.access_token,
            &request.folder_id,
        )
        .await
        .map_err(|error| {
            log_command_error(
                "delete_folder",
                &AppError::NetworkRemoteError {
                    status: 500,
                    message: error.to_string(),
                },
            )
        })?;

    // 触发 folders-only sync 同步数据到本地数据库
    state
        .sync_service()
        .sync_folders_only(SyncVaultCommand {
            account_id: auth_session.account_id.clone(),
            base_url: auth_session.base_url.clone(),
            access_token: auth_session.access_token.clone(),
            exclude_domains: false,
            trigger: SyncTrigger::Manual,
        })
        .await
        .map_err(|error| log_command_error("delete_folder", &error))?;

    Ok(())
}
