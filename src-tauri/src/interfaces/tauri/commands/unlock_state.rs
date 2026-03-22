use tauri::State;

use crate::bootstrap::app_state::AppState;
use crate::bootstrap::unlock_state::UnlockStatus;
use crate::interfaces::tauri::dto::unlock_state::{
    AccountContextDto, GetUnlockStateRequestDto, RefreshSessionRequestDto,
    RefreshSessionResponseDto, SessionContextDto, UnlockMethodDto, UnlockStateResponseDto,
    UnlockStatusDto,
};
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

const REFRESH_GRACE_PERIOD_SECS: u64 = 60;

fn log_command_error(command: &str, error: &AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::unlock_state",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

/// Get current unlock state
#[tauri::command]
#[specta::specta]
pub async fn get_unlock_state(
    state: State<'_, AppState>,
    _request: GetUnlockStateRequestDto,
) -> Result<UnlockStateResponseDto, ErrorPayload> {
    let manager = state.unlock_manager();
    let current_state = manager.current_state().await;

    let account_dto = current_state.account_context.map(|ctx| AccountContextDto {
        account_id: ctx.account_id,
        email: ctx.email,
        base_url: ctx.base_url,
    });

    let session_dto = current_state.session_context.map(|session| {
        let is_valid = !session.is_expired();
        let is_expiring_soon =
            session.is_expiring_within(std::time::Duration::from_secs(REFRESH_GRACE_PERIOD_SECS));
        SessionContextDto {
            is_valid,
            is_expiring_soon,
        }
    });

    let unlock_method_dto = current_state.unlock_method.map(|method| match method {
        crate::domain::unlock::UnlockMethod::MasterPassword { .. } => {
            UnlockMethodDto::MasterPassword
        }
        crate::domain::unlock::UnlockMethod::Pin { .. } => UnlockMethodDto::Pin,
        crate::domain::unlock::UnlockMethod::Biometric => UnlockMethodDto::Biometric,
    });

    Ok(UnlockStateResponseDto {
        status: unlock_status_to_dto(current_state.status),
        account: account_dto,
        session: session_dto,
        has_key_material: current_state.key_material.is_some(),
        unlock_method: unlock_method_dto,
    })
}

/// Manually refresh the session
#[tauri::command]
#[specta::specta]
pub async fn refresh_session(
    state: State<'_, AppState>,
    _request: RefreshSessionRequestDto,
) -> Result<RefreshSessionResponseDto, ErrorPayload> {
    let manager = state.unlock_manager();

    // Check if we have a refresh token
    let refresh_token = match manager.refresh_token().await {
        Some(token) => token,
        None => {
            return Ok(RefreshSessionResponseDto {
                success: false,
                is_session_valid: false,
            })
        }
    };

    // Get account context for the refresh
    let account_context = match manager.account_context().await {
        Some(ctx) => ctx,
        None => {
            return Ok(RefreshSessionResponseDto {
                success: false,
                is_session_valid: false,
            })
        }
    };

    // Perform the refresh using the auth service
    let refresh_result = state
        .auth_service()
        .refresh_token(crate::application::dto::auth::RefreshTokenCommand {
            base_url: account_context.base_url.clone(),
            refresh_token: refresh_token.clone(),
        })
        .await;

    match refresh_result {
        Ok(refreshed) => {
            // Build new session context
            let session_context = manager
                .build_session_context(
                    refreshed.access_token,
                    refreshed.refresh_token.or(Some(refresh_token)),
                    refreshed.expires_in,
                )
                .map_err(|e| log_command_error("refresh_session", &e))?;

            // Update the session in the manager
            if let Err(e) = manager.update_session(session_context).await {
                log::warn!(
                    target: "vanguard::tauri::unlock_state",
                    "Failed to update session after refresh: {}",
                    e.log_message()
                );
                return Ok(RefreshSessionResponseDto {
                    success: false,
                    is_session_valid: false,
                });
            }

            Ok(RefreshSessionResponseDto {
                success: true,
                is_session_valid: true,
            })
        }
        Err(error) => {
            // Handle 401/403 errors by clearing session
            if matches!(error.status(), Some(401 | 403)) {
                let _ = manager.clear_session().await;
                let _ = state
                    .sync_service()
                    .stop_polling_for_account(&account_context.account_id);
                let _ = state
                    .realtime_sync_service()
                    .stop_for_account(&account_context.account_id)
                    .await;
            }

            Err(log_command_error("refresh_session", &error))
        }
    }
}

fn unlock_status_to_dto(status: UnlockStatus) -> UnlockStatusDto {
    match status {
        UnlockStatus::Locked => UnlockStatusDto::Locked,
        UnlockStatus::VaultUnlockedSessionExpired => UnlockStatusDto::VaultUnlockedSessionExpired,
        UnlockStatus::FullyUnlocked => UnlockStatusDto::FullyUnlocked,
        UnlockStatus::Unlocking => UnlockStatusDto::Unlocking,
    }
}
