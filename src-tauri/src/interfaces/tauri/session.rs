use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::dto::auth::{RefreshTokenCommand, SessionInfo};
use crate::bootstrap::app_state::{AppState, AuthSession};
use crate::interfaces::tauri::account_id;
use crate::support::error::AppError;
use crate::support::result::AppResult;

const REFRESH_GRACE_PERIOD_MS: i64 = 60_000;

pub fn build_auth_session(
    base_url: String,
    email: String,
    account_id: String,
    session: SessionInfo,
) -> AppResult<AuthSession> {
    Ok(AuthSession {
        account_id,
        base_url,
        email,
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at_ms: calc_expires_at_ms(session.expires_in)?,
        kdf: session.kdf,
        kdf_iterations: session.kdf_iterations,
        kdf_memory: session.kdf_memory,
        kdf_parallelism: session.kdf_parallelism,
    })
}

pub async fn ensure_fresh_auth_session(state: &AppState) -> AppResult<AuthSession> {
    refresh_auth_session(state, false).await
}

pub async fn force_refresh_auth_session(state: &AppState) -> AppResult<AuthSession> {
    refresh_auth_session(state, true).await
}

async fn refresh_auth_session(state: &AppState, force: bool) -> AppResult<AuthSession> {
    let current = state.require_auth_session()?;
    if !force && !current.is_expiring_within(REFRESH_GRACE_PERIOD_MS) {
        return Ok(current);
    }

    let refresh_token = current.refresh_token.clone().ok_or_else(|| {
        AppError::validation("current session missing refresh token, please login again")
    })?;

    let refreshed = match state
        .auth_service()
        .refresh_token(RefreshTokenCommand {
            base_url: current.base_url.clone(),
            refresh_token,
        })
        .await
    {
        Ok(value) => value,
        Err(error) => {
            if matches!(error.status(), Some(401 | 403)) {
                let _ = state
                    .sync_service()
                    .stop_polling_for_account(&current.account_id);
                let _ = state
                    .realtime_sync_service()
                    .stop_for_account(&current.account_id)
                    .await;
                let _ = state.clear_auth_session();
            }
            return Err(error);
        }
    };

    let account_id = account_id::derive_account_id_from_access_token(
        &current.base_url,
        &refreshed.access_token,
    )?;
    let next = AuthSession {
        account_id,
        base_url: current.base_url,
        email: current.email,
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token.or(current.refresh_token),
        expires_at_ms: calc_expires_at_ms(refreshed.expires_in)?,
        kdf: refreshed.kdf.or(current.kdf),
        kdf_iterations: refreshed.kdf_iterations.or(current.kdf_iterations),
        kdf_memory: refreshed.kdf_memory.or(current.kdf_memory),
        kdf_parallelism: refreshed.kdf_parallelism.or(current.kdf_parallelism),
    };

    state.set_auth_session(next.clone())?;
    start_background_sync(state, &next).await;
    Ok(next)
}

pub async fn start_background_sync(state: &AppState, session: &AuthSession) {
    if let Err(error) = state.sync_service().start_revision_polling(
        session.account_id.clone(),
        session.base_url.clone(),
        session.access_token.clone(),
    ) {
        log::warn!(
            target: "vanguard::tauri::session",
            "failed to start revision polling account_id={}: [{}] {}",
            session.account_id,
            error.code(),
            error.log_message()
        );
    }

    if let Err(error) = state
        .realtime_sync_service()
        .start_for_account(
            session.account_id.clone(),
            session.base_url.clone(),
            session.access_token.clone(),
        )
        .await
    {
        log::warn!(
            target: "vanguard::tauri::session",
            "failed to start realtime sync account_id={}: [{}] {}",
            session.account_id,
            error.code(),
            error.log_message()
        );
    }
}

fn calc_expires_at_ms(expires_in_seconds: i64) -> AppResult<i64> {
    let now_ms = now_unix_ms()?;
    let ttl_ms = expires_in_seconds.max(0).saturating_mul(1000);
    Ok(now_ms.saturating_add(ttl_ms))
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(format!("system clock before unix epoch: {error}")))?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}
