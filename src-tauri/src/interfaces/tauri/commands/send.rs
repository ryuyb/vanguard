use tauri::State;

use crate::application::ports::unlock_context_port::UnlockContextProvider;
use crate::application::use_cases::list_sends_use_case::ListSendsUseCase;
use crate::bootstrap::app_state::AppState;
use crate::interfaces::tauri::dto::send::{
    to_send_item_dto, CreateSendRequestDto, DeleteSendRequestDto, SendItemDto,
    SendMutationResponseDto, UpdateSendRequestDto,
};
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: &AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::send",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

#[tauri::command]
#[specta::specta]
pub async fn list_sends(state: State<'_, AppState>) -> Result<Vec<SendItemDto>, ErrorPayload> {
    let unlock_manager = state.unlock_manager();
    let ctx = unlock_manager
        .require_fully_unlocked()
        .await
        .map_err(|e| log_command_error("list_sends", &e))?;

    let account_id = ctx.account.account_id;
    let user_key = ctx.key.to_dto();

    let sends = ListSendsUseCase::new(state.vault_repository())
        .execute(account_id, user_key)
        .await
        .map_err(|e| log_command_error("list_sends", &e))?;

    Ok(sends.into_iter().map(to_send_item_dto).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn create_send(
    request: CreateSendRequestDto,
    state: State<'_, AppState>,
) -> Result<SendMutationResponseDto, ErrorPayload> {
    let unlock_manager = state.unlock_manager();
    let ctx = unlock_manager
        .require_fully_unlocked()
        .await
        .map_err(|e| log_command_error("create_send", &e))?;

    let account_id = ctx.account.account_id;
    let base_url = ctx.account.base_url;
    let access_token = ctx.session.access_token;
    let user_key = ctx.key.to_dto();

    let result = state
        .create_send_use_case()
        .execute(
            account_id,
            base_url,
            access_token,
            request.send,
            user_key,
            request.file_data,
        )
        .await
        .map_err(|e| log_command_error("create_send", &e))?;

    Ok(SendMutationResponseDto {
        send_id: result.send_id,
        revision_date: result.revision_date,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn update_send(
    request: UpdateSendRequestDto,
    state: State<'_, AppState>,
) -> Result<SendMutationResponseDto, ErrorPayload> {
    let unlock_manager = state.unlock_manager();
    let ctx = unlock_manager
        .require_fully_unlocked()
        .await
        .map_err(|e| log_command_error("update_send", &e))?;

    let account_id = ctx.account.account_id;
    let base_url = ctx.account.base_url;
    let access_token = ctx.session.access_token;
    let user_key = ctx.key.to_dto();

    let result = state
        .update_send_use_case()
        .execute(
            account_id,
            base_url,
            access_token,
            request.send_id,
            request.send,
            user_key,
        )
        .await
        .map_err(|e| log_command_error("update_send", &e))?;

    Ok(SendMutationResponseDto {
        send_id: result.send_id,
        revision_date: result.revision_date,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn delete_send(
    request: DeleteSendRequestDto,
    state: State<'_, AppState>,
) -> Result<(), ErrorPayload> {
    let unlock_manager = state.unlock_manager();
    let ctx = unlock_manager
        .require_fully_unlocked()
        .await
        .map_err(|e| log_command_error("delete_send", &e))?;

    let account_id = ctx.account.account_id;
    let base_url = ctx.account.base_url;
    let access_token = ctx.session.access_token;

    state
        .delete_send_use_case()
        .execute(account_id, base_url, access_token, request.send_id)
        .await
        .map_err(|e| log_command_error("delete_send", &e))?;

    Ok(())
}
