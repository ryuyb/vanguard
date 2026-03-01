use std::sync::Arc;

use tauri::State;

use crate::application::dto::auth::PasswordLoginOutcome;
use crate::application::dto::sync::SyncVaultCommand;
use crate::application::services::sync_service::SyncService;
use crate::bootstrap::app_state::AppState;
use crate::domain::sync::SyncTrigger;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::auth::{
    PasswordLoginRequestDto, PasswordLoginResponseDto, PreloginRequestDto, PreloginResponseDto,
    RefreshTokenRequestDto, SendEmailLoginRequestDto, SessionResponseDto,
    VerifyEmailTokenRequestDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::AppError;
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: AppError) -> String {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::auth",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload.message
}

#[tauri::command]
#[specta::specta]
pub async fn auth_prelogin(
    state: State<'_, AppState>,
    request: PreloginRequestDto,
) -> Result<PreloginResponseDto, String> {
    let query = mapping::to_prelogin_query(request);
    let result = state
        .auth_service()
        .prelogin(query)
        .await
        .map_err(|error| log_command_error("auth_prelogin", error))?;

    Ok(mapping::to_prelogin_response_dto(result))
}

#[tauri::command]
#[specta::specta]
pub async fn auth_login_with_password(
    state: State<'_, AppState>,
    request: PasswordLoginRequestDto,
) -> Result<PasswordLoginResponseDto, String> {
    let command = mapping::to_password_login_command(request);
    let base_url = command.base_url.clone();
    let result = state
        .auth_service()
        .login_with_password(command)
        .await
        .map_err(|error| log_command_error("auth_login_with_password", error))?;

    if let PasswordLoginOutcome::Authenticated(session) = &result {
        match account_id::derive_account_id_from_access_token(&base_url, &session.access_token) {
            Ok(account_id) => {
                if let Err(error) = state.sync_service().start_revision_polling(
                    account_id.clone(),
                    base_url.clone(),
                    session.access_token.clone(),
                ) {
                    log::warn!(
                        target: "vanguard::tauri::auth",
                        "failed to start revision polling after login: [{}] {}",
                        error.code(),
                        error.log_message()
                    );
                }
                if let Err(error) = state
                    .realtime_sync_service()
                    .start_for_account(
                        account_id.clone(),
                        base_url.clone(),
                        session.access_token.clone(),
                    )
                    .await
                {
                    log::warn!(
                        target: "vanguard::tauri::auth",
                        "failed to start realtime sync after login: [{}] {}",
                        error.code(),
                        error.log_message()
                    );
                }
                trigger_sync_after_login(
                    state.sync_service(),
                    account_id,
                    base_url,
                    session.access_token.clone(),
                );
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::tauri::auth",
                    "skip auto sync after login: [{}] {}",
                    error.code(),
                    error.log_message()
                );
            }
        }
    }

    Ok(mapping::to_password_login_response_dto(result))
}

#[tauri::command]
#[specta::specta]
pub async fn auth_refresh_token(
    state: State<'_, AppState>,
    request: RefreshTokenRequestDto,
) -> Result<SessionResponseDto, String> {
    let command = mapping::to_refresh_token_command(request);
    let base_url = command.base_url.clone();
    let result = state
        .auth_service()
        .refresh_token(command)
        .await
        .map_err(|error| log_command_error("auth_refresh_token", error))?;

    match account_id::derive_account_id_from_access_token(&base_url, &result.access_token) {
        Ok(account_id) => {
            let polling_account_id = account_id.clone();
            let polling_base_url = base_url.clone();
            if let Err(error) = state.sync_service().start_revision_polling(
                polling_account_id,
                polling_base_url,
                result.access_token.clone(),
            ) {
                log::warn!(
                    target: "vanguard::tauri::auth",
                    "failed to restart revision polling after refresh: [{}] {}",
                    error.code(),
                    error.log_message()
                );
            }
            if let Err(error) = state
                .realtime_sync_service()
                .start_for_account(account_id.clone(), base_url, result.access_token.clone())
                .await
            {
                log::warn!(
                    target: "vanguard::tauri::auth",
                    "failed to restart realtime sync after refresh: [{}] {}",
                    error.code(),
                    error.log_message()
                );
            }
        }
        Err(error) => {
            log::warn!(
                target: "vanguard::tauri::auth",
                "failed to derive account id from refreshed token: [{}] {}",
                error.code(),
                error.log_message()
            );
        }
    }

    Ok(mapping::to_session_response_dto(result))
}

#[tauri::command]
#[specta::specta]
pub async fn auth_send_email_login(
    state: State<'_, AppState>,
    request: SendEmailLoginRequestDto,
) -> Result<(), String> {
    let command = mapping::to_send_email_login_command(request);
    state
        .auth_service()
        .send_email_login(command)
        .await
        .map_err(|error| log_command_error("auth_send_email_login", error))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn auth_verify_email_token(
    state: State<'_, AppState>,
    request: VerifyEmailTokenRequestDto,
) -> Result<(), String> {
    let command = mapping::to_verify_email_token_command(request);
    state
        .auth_service()
        .verify_email_token(command)
        .await
        .map_err(|error| log_command_error("auth_verify_email_token", error))?;
    Ok(())
}

fn trigger_sync_after_login(
    sync_service: Arc<SyncService>,
    account_id: String,
    base_url: String,
    access_token: String,
) {
    let endpoint = sync_endpoint(&base_url);
    log::info!(
        target: "vanguard::tauri::auth",
        "login succeeded, triggering initial sync account_id={} endpoint={}",
        account_id,
        endpoint
    );

    tokio::spawn(async move {
        let command = SyncVaultCommand {
            account_id: account_id.clone(),
            base_url: base_url.clone(),
            access_token,
            exclude_domains: false,
            trigger: SyncTrigger::Startup,
        };

        if let Err(error) = sync_service.sync_now(command).await {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auto sync after login failed account_id={} endpoint={} status={} error_code={} message={}",
                account_id,
                sync_endpoint(&base_url),
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
        }
    });
}

fn sync_endpoint(base_url: &str) -> String {
    format!("{}/api/sync", base_url.trim_end_matches('/'))
}
