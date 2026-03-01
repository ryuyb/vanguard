use std::sync::Arc;

use tauri::State;

use crate::application::dto::auth::PasswordLoginOutcome;
use crate::application::dto::sync::SyncVaultCommand;
use crate::application::services::sync_service::SyncService;
use crate::bootstrap::app_state::AppState;
use crate::domain::sync::SyncTrigger;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::auth::{
    LogoutRequestDto, PasswordLoginRequestDto, PasswordLoginResponseDto,
    RestoreAuthStateRequestDto, RestoreAuthStateResponseDto, RestoreAuthStateStatusDto,
    SendEmailLoginRequestDto, VerifyEmailTokenRequestDto,
};
use crate::interfaces::tauri::mapping;
use crate::interfaces::tauri::session;
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
pub async fn auth_login_with_password(
    state: State<'_, AppState>,
    request: PasswordLoginRequestDto,
) -> Result<PasswordLoginResponseDto, String> {
    let base_url = request.base_url.clone();
    let email = request.email.clone();
    let master_password = request.master_password.clone();
    let command = mapping::to_password_login_command(request);
    let result = state
        .auth_service()
        .login_with_password(command)
        .await
        .map_err(|error| log_command_error("auth_login_with_password", error))?;

    if !matches!(result, PasswordLoginOutcome::Authenticated(_)) {
        let _ = state.clear_auth_session();
    }

    if let PasswordLoginOutcome::Authenticated(session) = &result {
        match account_id::derive_account_id_from_access_token(&base_url, &session.access_token) {
            Ok(account_id) => {
                let auth_session = session::build_auth_session(
                    base_url.clone(),
                    email.clone(),
                    account_id.clone(),
                    session.clone(),
                )
                .and_then(|value| {
                    state
                        .set_auth_session(value.clone())
                        .map(|_| value)
                        .map_err(|error| {
                            AppError::internal(format!(
                                "failed to store authenticated session: {}",
                                error.log_message()
                            ))
                        })
                });

                match auth_session {
                    Ok(auth_session) => {
                        if let Err(error) =
                            state.persist_auth_state(&auth_session, &master_password)
                        {
                            log::warn!(
                                target: "vanguard::tauri::auth",
                                "failed to persist encrypted auth state account_id={}: [{}] {}",
                                auth_session.account_id,
                                error.code(),
                                error.log_message()
                            );
                        }
                        session::start_background_sync(&state, &auth_session).await;
                        trigger_sync_after_login(
                            state.sync_service(),
                            auth_session.account_id,
                            auth_session.base_url,
                            auth_session.access_token,
                        );
                    }
                    Err(error) => {
                        log::warn!(
                            target: "vanguard::tauri::auth",
                            "failed to initialize authenticated session state: [{}] {}",
                            error.code(),
                            error.log_message()
                        );
                    }
                }
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
pub async fn auth_restore_state(
    state: State<'_, AppState>,
    _request: RestoreAuthStateRequestDto,
) -> Result<RestoreAuthStateResponseDto, String> {
    if let Some(session) = state
        .auth_session()
        .map_err(|error| log_command_error("auth_restore_state", error))?
    {
        return Ok(RestoreAuthStateResponseDto {
            status: RestoreAuthStateStatusDto::Authenticated,
            account_id: Some(session.account_id),
            base_url: Some(session.base_url),
            email: Some(session.email),
        });
    }

    if let Some(context) = state
        .persisted_auth_context()
        .map_err(|error| log_command_error("auth_restore_state", error))?
    {
        return Ok(RestoreAuthStateResponseDto {
            status: RestoreAuthStateStatusDto::Locked,
            account_id: Some(context.account_id),
            base_url: Some(context.base_url),
            email: Some(context.email),
        });
    }

    Ok(RestoreAuthStateResponseDto {
        status: RestoreAuthStateStatusDto::NeedsLogin,
        account_id: None,
        base_url: None,
        email: None,
    })
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

#[tauri::command]
#[specta::specta]
pub async fn auth_logout(
    state: State<'_, AppState>,
    _request: LogoutRequestDto,
) -> Result<(), String> {
    let active_session_account_id = state
        .auth_session()
        .map_err(|error| log_command_error("auth_logout", error))?
        .map(|value| value.account_id);
    let persisted_account_id = state
        .persisted_auth_context()
        .map_err(|error| log_command_error("auth_logout", error))?
        .map(|value| value.account_id);
    let account_id = active_session_account_id.or(persisted_account_id);

    if let Some(account_id) = account_id {
        if let Err(error) = state.sync_service().stop_polling_for_account(&account_id) {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to stop polling account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        }
        if let Err(error) = state
            .realtime_sync_service()
            .stop_for_account(&account_id)
            .await
        {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to stop realtime sync account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        }
    }

    state
        .clear_all_auth_state()
        .map_err(|error| log_command_error("auth_logout", error))?;
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
