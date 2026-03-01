use tauri::State;

use crate::bootstrap::app_state::AppState;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::sync::{
    SyncNowRequestDto, SyncStatusRequestDto, SyncStatusResponseDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::AppError;

fn log_command_error(command: &str, error: AppError) -> String {
    let payload = error.to_payload();
    log::error!(
        target: "vanguard::tauri::sync",
        "{command} failed: [{}] {}",
        payload.code,
        payload.message
    );
    payload.message
}

#[tauri::command]
#[specta::specta]
pub async fn vault_sync_now(
    state: State<'_, AppState>,
    request: SyncNowRequestDto,
) -> Result<SyncStatusResponseDto, String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_sync_now", error))?;
    let command = mapping::to_sync_vault_command(request, account_id);
    let outcome = state
        .sync_service()
        .sync_now(command)
        .await
        .map_err(|error| log_command_error("vault_sync_now", error))?;

    Ok(mapping::to_sync_outcome_dto(outcome))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_sync_status(
    state: State<'_, AppState>,
    request: SyncStatusRequestDto,
) -> Result<SyncStatusResponseDto, String> {
    let account_id =
        account_id::derive_account_id_from_access_token(&request.base_url, &request.access_token)
            .map_err(|error| log_command_error("vault_sync_status", error))?;
    let context = state
        .sync_service()
        .sync_status(account_id)
        .await
        .map_err(|error| log_command_error("vault_sync_status", error))?;

    Ok(mapping::to_sync_status_response_dto(context))
}
