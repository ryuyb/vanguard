use tauri::State;

use crate::application::dto::auth::{PasswordLoginOutcome, SessionInfo};
use crate::bootstrap::app_state::AppState;
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

async fn initialize_authenticated_session(
    state: &AppState,
    base_url: &str,
    email: &str,
    master_password: &str,
    session_info: &SessionInfo,
) -> Result<(), AppError> {
    let account_id =
        account_id::derive_account_id_from_access_token(base_url, &session_info.access_token)?;
    let auth_session = session::build_auth_session(
        base_url.to_string(),
        email.to_string(),
        account_id,
        session_info.clone(),
    )?;
    state.set_auth_session(auth_session.clone())?;
    if let Err(error) = state.persist_auth_state(&auth_session, master_password) {
        log::warn!(
            target: "vanguard::tauri::auth",
            "failed to persist encrypted auth state account_id={}: [{}] {}",
            auth_session.account_id,
            error.code(),
            error.log_message()
        );
    }
    session::start_background_sync(state, &auth_session).await;
    Ok(())
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
        if let Err(error) =
            initialize_authenticated_session(&state, &base_url, &email, &master_password, session)
                .await
        {
            if let Err(clear_error) = state.clear_auth_session() {
                log::warn!(
                    target: "vanguard::tauri::auth",
                    "failed to cleanup auth session after init error: [{}] {}",
                    clear_error.code(),
                    clear_error.log_message()
                );
            }
            return Err(log_command_error("auth_login_with_password", error));
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
