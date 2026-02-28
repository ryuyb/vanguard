use tauri::State;

use crate::bootstrap::app_state::AppState;
use crate::interfaces::tauri::dto::auth::{
    PasswordLoginRequestDto, PasswordLoginResponseDto, PreloginRequestDto, PreloginResponseDto,
    RefreshTokenRequestDto, SendEmailLoginRequestDto, SessionResponseDto,
    VerifyEmailTokenRequestDto,
};
use crate::interfaces::tauri::mapping;
use crate::support::error::AppError;

fn log_command_error(command: &str, error: AppError) -> String {
    let payload = error.to_payload();
    log::error!(
        target: "vanguard::tauri::auth",
        "{command} failed: [{}] {}",
        payload.code,
        payload.message
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
    let result = state
        .auth_service()
        .login_with_password(command)
        .await
        .map_err(|error| log_command_error("auth_login_with_password", error))?;

    Ok(mapping::to_password_login_response_dto(result))
}

#[tauri::command]
#[specta::specta]
pub async fn auth_refresh_token(
    state: State<'_, AppState>,
    request: RefreshTokenRequestDto,
) -> Result<SessionResponseDto, String> {
    let command = mapping::to_refresh_token_command(request);
    let result = state
        .auth_service()
        .refresh_token(command)
        .await
        .map_err(|error| log_command_error("auth_refresh_token", error))?;

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
