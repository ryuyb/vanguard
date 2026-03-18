use tauri::State;

use crate::application::dto::auth::{PasswordLoginOutcome, SessionInfo};
use crate::bootstrap::app_state::AppState;
use crate::infrastructure::vaultwarden::registration_adapter::VaultwardenRegistrationAdapter;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::auth::{
    LogoutRequestDto, PasswordLoginRequestDto, PasswordLoginResponseDto,
    RegisterFinishRequestDto, RestoreAuthStateRequestDto, RestoreAuthStateResponseDto,
    RestoreAuthStateStatusDto, SendEmailLoginRequestDto, SendVerificationEmailRequestDto,
    SendVerificationEmailResponseDto, VerifyEmailTokenRequestDto,
};
use crate::interfaces::tauri::mapping;
use crate::interfaces::tauri::session;
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

fn log_command_error(command: &str, error: AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::auth",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
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
) -> Result<PasswordLoginResponseDto, ErrorPayload> {
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
) -> Result<(), ErrorPayload> {
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
) -> Result<RestoreAuthStateResponseDto, ErrorPayload> {
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
) -> Result<(), ErrorPayload> {
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
) -> Result<(), ErrorPayload> {
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

        // Clear PIN unlock data
        if let Err(error) = state
            .pin_unlock_port()
            .delete_pin_envelope(&account_id, crate::domain::unlock::PinLockType::Persistent)
            .await
        {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to clear persistent PIN account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        }
        if let Err(error) = state
            .pin_unlock_port()
            .delete_pin_envelope(&account_id, crate::domain::unlock::PinLockType::Ephemeral)
            .await
        {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to clear ephemeral PIN account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        }

        // Clear biometric unlock data
        if let Err(error) = state
            .biometric_unlock_port()
            .delete_unlock_bundle(&account_id)
        {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to clear biometric unlock account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        }

        // Delete account database
        if let Err(error) = state
            .sync_service()
            .vault_repository()
            .delete_account_database(&account_id)
            .await
        {
            log::warn!(
                target: "vanguard::tauri::auth",
                "auth_logout failed to delete database account_id={}: [{}] {}",
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

#[tauri::command]
#[specta::specta]
pub async fn auth_send_verification_email(
    state: State<'_, AppState>,
    request: SendVerificationEmailRequestDto,
) -> Result<SendVerificationEmailResponseDto, ErrorPayload> {
    use crate::application::dto::auth::{RegistrationOutcome, SendVerificationEmailCommand};

    let adapter = VaultwardenRegistrationAdapter::new(state.vaultwarden_client().clone());
    let command = SendVerificationEmailCommand {
        base_url: request.base_url,
        email: request.email,
        name: request.name,
    };

    let outcome = adapter
        .send_verification_email(command)
        .await
        .map_err(|error| log_command_error("auth_send_verification_email", error))?;

    Ok(match outcome {
        RegistrationOutcome::Disabled { message } => {
            SendVerificationEmailResponseDto::Disabled { message }
        }
        RegistrationOutcome::EmailVerificationRequired => {
            SendVerificationEmailResponseDto::EmailVerificationRequired
        }
        RegistrationOutcome::DirectRegistration { token } => {
            SendVerificationEmailResponseDto::DirectRegistration { token }
        }
    })
}

#[tauri::command]
#[specta::specta]
pub async fn auth_register_finish(
    state: State<'_, AppState>,
    request: RegisterFinishRequestDto,
) -> Result<(), ErrorPayload> {
    use crate::application::dto::auth::RegisterFinishCommand;
    use crate::application::services::registration_service::derive_registration_keys;
    use crate::infrastructure::vaultwarden::models::{RegisterFinishRequest, RegisterKeys};

    let command = RegisterFinishCommand {
        base_url: request.base_url.clone(),
        email: request.email.clone(),
        name: request.name.clone(),
        master_password: request.master_password.clone(),
        master_password_hint: request.master_password_hint.clone(),
        token: request.token.clone(),
        kdf: request.kdf,
        kdf_iterations: request.kdf_iterations,
        kdf_memory: request.kdf_memory,
        kdf_parallelism: request.kdf_parallelism,
    };

    // Step 1: Derive all cryptographic keys
    let keys = derive_registration_keys(&command)
        .map_err(|error| log_command_error("auth_register_finish", error))?;

    // Step 2: Call register/finish API
    let api_request = RegisterFinishRequest {
        email: command.email.clone(),
        master_password_hash: keys.master_password_hash,
        master_password_hint: command.master_password_hint,
        user_symmetric_key: keys.encrypted_symmetric_key,
        user_asymmetric_keys: RegisterKeys {
            public_key: keys.public_key_b64,
            encrypted_private_key: keys.encrypted_private_key,
        },
        kdf: command.kdf,
        kdf_iterations: command.kdf_iterations,
        kdf_memory: command.kdf_memory,
        kdf_parallelism: command.kdf_parallelism,
        email_verification_token: Some(command.token),
    };

    state
        .vaultwarden_client()
        .register_finish(&request.base_url, api_request)
        .await
        .map_err(|error| {
            let (status, message) = match &error {
                crate::infrastructure::vaultwarden::error::VaultwardenError::ApiError {
                    status,
                    message,
                    ..
                } => (*status, message.clone()),
                other => (0, format!("{other}")),
            };
            log_command_error(
                "auth_register_finish",
                AppError::NetworkRemoteError { status, message },
            )
        })?;

    // Step 3: Auto-login after successful registration
    let login_result = state
        .auth_service()
        .login_with_password(crate::application::dto::auth::PasswordLoginCommand {
            base_url: request.base_url.clone(),
            username: request.email.clone(),
            password: request.master_password.clone(),
            two_factor_provider: None,
            two_factor_token: None,
            two_factor_remember: None,
            authrequest: None,
        })
        .await
        .map_err(|error| log_command_error("auth_register_finish", error))?;

    if let crate::application::dto::auth::PasswordLoginOutcome::Authenticated(session) =
        login_result
    {
        initialize_authenticated_session(
            &state,
            &request.base_url,
            &request.email,
            &request.master_password,
            &session,
        )
        .await
        .map_err(|error| {
            let _ = state.clear_auth_session();
            log_command_error("auth_register_finish", error)
        })?;
    }

    Ok(())
}
