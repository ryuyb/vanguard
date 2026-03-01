use tauri::State;

use crate::application::dto::sync::SyncVaultCommand;
use crate::bootstrap::app_state::AppState;
use crate::domain::sync::SyncTrigger;
use crate::interfaces::tauri::dto::sync::{
    SyncNowRequestDto, SyncStatusRequestDto, SyncStatusResponseDto,
};
use crate::interfaces::tauri::{mapping, session};
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: AppError) -> String {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::sync",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload.message
}

#[tauri::command]
#[specta::specta]
pub async fn vault_sync_now(
    state: State<'_, AppState>,
    request: SyncNowRequestDto,
) -> Result<SyncStatusResponseDto, String> {
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("vault_sync_now", error))?;
    let exclude_domains = request.exclude_domains.unwrap_or(false);
    let command = build_sync_command(&auth_session, exclude_domains, SyncTrigger::Manual);
    let outcome = match state.sync_service().sync_now(command.clone()).await {
        Ok(value) => value,
        Err(error) if is_auth_status_error(&error) => {
            let refreshed = session::force_refresh_auth_session(&state)
                .await
                .map_err(|refresh_error| log_command_error("vault_sync_now", refresh_error))?;
            let retry_command =
                build_sync_command(&refreshed, exclude_domains, SyncTrigger::Manual);
            state
                .sync_service()
                .sync_now(retry_command)
                .await
                .map_err(|retry_error| log_command_error("vault_sync_now", retry_error))?
        }
        Err(error) => return Err(log_command_error("vault_sync_now", error)),
    };
    let metrics = state
        .sync_service()
        .sync_metrics(outcome.context.account_id.clone())
        .map_err(|error| log_command_error("vault_sync_now", error))?;

    Ok(mapping::to_sync_status_response_dto(
        outcome.context,
        Some(metrics),
    ))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_sync_status(
    state: State<'_, AppState>,
    _request: SyncStatusRequestDto,
) -> Result<SyncStatusResponseDto, String> {
    let account_id = state
        .require_auth_session()
        .map(|value| value.account_id)
        .map_err(|error| log_command_error("vault_sync_status", error))?;
    let context = state
        .sync_service()
        .sync_status(account_id)
        .await
        .map_err(|error| log_command_error("vault_sync_status", error))?;
    let metrics = state
        .sync_service()
        .sync_metrics(context.account_id.clone())
        .map_err(|error| log_command_error("vault_sync_status", error))?;

    Ok(mapping::to_sync_status_response_dto(context, Some(metrics)))
}

#[tauri::command]
#[specta::specta]
pub async fn vault_sync_check_revision(
    state: State<'_, AppState>,
    _request: SyncStatusRequestDto,
) -> Result<SyncStatusResponseDto, String> {
    let auth_session = session::ensure_fresh_auth_session(&state)
        .await
        .map_err(|error| log_command_error("vault_sync_check_revision", error))?;
    let mut account_id = auth_session.account_id.clone();
    let check_result = state
        .sync_service()
        .check_revision_now(
            auth_session.account_id,
            auth_session.base_url,
            auth_session.access_token,
            SyncTrigger::Foreground,
        )
        .await;
    if let Err(error) = check_result {
        if is_auth_status_error(&error) {
            let refreshed =
                session::force_refresh_auth_session(&state)
                    .await
                    .map_err(|refresh_error| {
                        log_command_error("vault_sync_check_revision", refresh_error)
                    })?;
            account_id = refreshed.account_id.clone();
            state
                .sync_service()
                .check_revision_now(
                    refreshed.account_id.clone(),
                    refreshed.base_url,
                    refreshed.access_token,
                    SyncTrigger::Foreground,
                )
                .await
                .map_err(|retry_error| {
                    log_command_error("vault_sync_check_revision", retry_error)
                })?;
        } else {
            return Err(log_command_error("vault_sync_check_revision", error));
        }
    }

    let context = state
        .sync_service()
        .sync_status(account_id)
        .await
        .map_err(|error| log_command_error("vault_sync_check_revision", error))?;
    let metrics = state
        .sync_service()
        .sync_metrics(context.account_id.clone())
        .map_err(|error| log_command_error("vault_sync_check_revision", error))?;

    Ok(mapping::to_sync_status_response_dto(context, Some(metrics)))
}

fn build_sync_command(
    session: &crate::bootstrap::app_state::AuthSession,
    exclude_domains: bool,
    trigger: SyncTrigger,
) -> SyncVaultCommand {
    SyncVaultCommand {
        account_id: session.account_id.clone(),
        base_url: session.base_url.clone(),
        access_token: session.access_token.clone(),
        exclude_domains,
        trigger,
    }
}

fn is_auth_status_error(error: &AppError) -> bool {
    matches!(error.status(), Some(401 | 403))
}
