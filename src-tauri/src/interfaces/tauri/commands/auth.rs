use tauri::State;

use crate::application::crypto::key_derivation::{
    derive_master_key_pbkdf2, derive_stretched_master_key,
};
use crate::application::dto::auth::{PasswordLoginOutcome, SessionInfo};
use crate::application::dto::unlock::VaultUserKeyMaterial;
use crate::application::ports::unlock_context_port::UnlockContextProvider;
use crate::bootstrap::app_state::{AppState, VaultUserKey};
use crate::infrastructure::vaultwarden::registration_adapter::VaultwardenRegistrationAdapter;
use crate::interfaces::tauri::account_id;
use crate::interfaces::tauri::dto::auth::{
    LogoutRequestDto, PasswordLoginRequestDto, PasswordLoginResponseDto, RegisterFinishRequestDto,
    RestoreAuthStateRequestDto, RestoreAuthStateResponseDto, RestoreAuthStateStatusDto,
    SendEmailLoginRequestDto, SendVerificationEmailRequestDto, SendVerificationEmailResponseDto,
    VerifyEmailTokenRequestDto,
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

/// Decrypt vault key from encrypted key (SessionInfo.key) using master password
fn decrypt_vault_key(
    master_password: &str,
    email: &str,
    encrypted_key: &str,
    kdf: i32,
    kdf_iterations: i32,
    _kdf_memory: Option<i32>,
    _kdf_parallelism: Option<i32>,
) -> Result<VaultUserKey, AppError> {
    use crate::application::vault_crypto;

    // Currently only support PBKDF2 (kdf=0)
    if kdf != 0 {
        return Err(AppError::ValidationFieldError {
            field: "kdf".to_string(),
            message: format!("unsupported KDF type: {}", kdf),
        });
    }

    // Derive master key from password + email
    let master_key = derive_master_key_pbkdf2(master_password, email, Some(kdf_iterations as u32))?;

    // Derive stretching key from master key (for decrypting the user key)
    let stretched = derive_stretched_master_key(&master_key)?;
    let wrapping_key = VaultUserKeyMaterial {
        enc_key: stretched.enc_key,
        mac_key: stretched.mac_key,
        refresh_token: None,
    };

    // Decrypt the encrypted user key
    let plaintext_user_key = vault_crypto::decrypt_cipher_bytes(encrypted_key, &wrapping_key)
        .map_err(|error| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("failed to decrypt user key: {}", error.message()),
        })?;

    // Parse the decrypted user key
    let user_key = vault_crypto::parse_user_key_material(&plaintext_user_key).map_err(|error| {
        AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("failed to parse user key: {}", error.message()),
        }
    })?;

    Ok(VaultUserKey {
        enc_key: user_key.enc_key,
        mac_key: user_key.mac_key,
    })
}

async fn initialize_authenticated_session(
    state: &AppState,
    base_url: &str,
    email: &str,
    master_password: &str,
    session_info: &SessionInfo,
) -> Result<(), AppError> {
    use crate::bootstrap::unlock_state::{AccountContext, SessionContext, VaultKeyMaterial};

    let account_id =
        account_id::derive_account_id_from_access_token(base_url, &session_info.access_token)?;

    // Build AuthSession for persistence and background sync
    let auth_session = session::build_auth_session(
        base_url.to_string(),
        email.to_string(),
        account_id.clone(),
        session_info.clone(),
    )?;

    // Decrypt vault key from encrypted key
    let kdf = session_info.kdf.unwrap_or(0);
    let kdf_iterations = session_info.kdf_iterations.unwrap_or(100000);
    let kdf_memory = session_info.kdf_memory;
    let kdf_parallelism = session_info.kdf_parallelism;

    let encrypted_key =
        session_info
            .key
            .as_ref()
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "key".to_string(),
                message: "session key is missing".to_string(),
            })?;

    let user_key = decrypt_vault_key(
        master_password,
        email,
        encrypted_key,
        kdf,
        kdf_iterations,
        kdf_memory,
        kdf_parallelism,
    )?;

    // Update unlock_manager directly
    let unlock_manager = state.unlock_manager();
    let account_ctx = AccountContext {
        account_id: account_id.clone(),
        email: email.to_string(),
        base_url: base_url.to_string(),
        kdf: auth_session.kdf,
        kdf_iterations: auth_session.kdf_iterations,
        kdf_memory: auth_session.kdf_memory,
        kdf_parallelism: auth_session.kdf_parallelism,
    };
    let session_ctx = SessionContext {
        access_token: session_info.access_token.clone(),
        refresh_token: session_info.refresh_token.clone(),
        expires_at: std::time::Instant::now()
            + std::time::Duration::from_secs(session_info.expires_in.max(0) as u64),
        last_activity: std::time::Instant::now(),
    };
    unlock_manager.set_account_context(account_ctx).await?;
    unlock_manager.set_session_context(session_ctx).await?;

    // Set vault key material to unlock the vault
    let key_material = VaultKeyMaterial {
        enc_key: user_key.enc_key.clone(),
        mac_key: user_key.mac_key.clone(),
    };
    unlock_manager.set_key_material(key_material).await?;

    // Initialize icon downloader
    state.icon_service().set_downloader(base_url).await;

    // Persist auth state via unlock_manager
    // Note: refresh_token may be None if the server doesn't support it
    if session_info.refresh_token.is_some() {
        if let Err(error) = unlock_manager.persist(Some(master_password)).await {
            log::warn!(
                target: "vanguard::tauri::auth",
                "failed to persist encrypted auth state account_id={}: [{}] {}",
                account_id,
                error.code(),
                error.log_message()
            );
        } else {
            log::info!(
                target: "vanguard::tauri::auth",
                "auth state persisted successfully account_id={}",
                account_id
            );
        }
    } else {
        log::warn!(
            target: "vanguard::tauri::auth",
            "no refresh_token in session response, auth state will not be persisted. \
             Master password unlock will require re-login. account_id={}",
            account_id
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

    let unlock_manager = state.unlock_manager();
    if !matches!(result, PasswordLoginOutcome::Authenticated(_)) {
        let _ = unlock_manager.logout().await;
    }

    if let PasswordLoginOutcome::Authenticated(session) = &result {
        if let Err(error) =
            initialize_authenticated_session(&state, &base_url, &email, &master_password, session)
                .await
        {
            if let Err(clear_error) = unlock_manager.logout().await {
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
    let unlock_manager = state.unlock_manager();

    // Check if fully unlocked (has valid session)
    if let Some(ctx) = unlock_manager.try_get_fully_unlocked().await {
        return Ok(RestoreAuthStateResponseDto {
            status: RestoreAuthStateStatusDto::Authenticated,
            account_id: Some(ctx.account.account_id),
            base_url: Some(ctx.account.base_url),
            email: Some(ctx.account.email),
        });
    }

    // Check if vault is unlocked but session expired
    if let Some(ctx) = unlock_manager.try_get_unlocked().await {
        return Ok(RestoreAuthStateResponseDto {
            status: RestoreAuthStateStatusDto::Locked,
            account_id: Some(ctx.account.account_id),
            base_url: Some(ctx.account.base_url),
            email: Some(ctx.account.email),
        });
    }

    // Check persisted auth context (for restore after app restart)
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
    let unlock_manager = state.unlock_manager();

    // Try to get account_id from unlock_manager first, then from persisted context
    let active_account_id = unlock_manager
        .get_account_context()
        .await
        .map(|ctx| ctx.account_id);
    let persisted_account_id = state
        .persisted_auth_context()
        .map_err(|error| log_command_error("auth_logout", error))?
        .map(|value| value.account_id);
    let account_id = active_account_id.or(persisted_account_id);

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
        .unlock_manager()
        .logout()
        .await
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
        match initialize_authenticated_session(
            &state,
            &request.base_url,
            &request.email,
            &request.master_password,
            &session,
        )
        .await
        {
            Ok(_) => {}
            Err(error) => {
                let _ = state.unlock_manager().logout().await;
                return Err(log_command_error("auth_register_finish", error));
            }
        }
    }

    Ok(())
}
