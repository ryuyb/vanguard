use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::dto::auth::{RefreshTokenCommand, SessionInfo};
use crate::bootstrap::app_state::{AppState, AuthSession};
use crate::interfaces::tauri::account_id;
use crate::support::error::AppError;
use crate::support::result::AppResult;

const REFRESH_GRACE_PERIOD_MS: i64 = 60_000;
type RefreshSingleflightLock = Arc<tokio::sync::Mutex<()>>;
static REFRESH_SINGLEFLIGHT_LOCKS: OnceLock<Mutex<HashMap<String, RefreshSingleflightLock>>> =
    OnceLock::new();

enum RefreshDecision {
    Retry,
    Return(AuthSession),
}

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

pub async fn restore_auth_session_with_master_password(
    state: &AppState,
    master_password: &str,
) -> AppResult<AuthSession> {
    let persisted = state
        .decrypt_persisted_auth_secret(master_password)?
        .ok_or_else(|| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "no persisted login state found in backend, please login first".to_string(),
        })?;
    let refresh_token = persisted.refresh_token.clone();
    let refreshed = match state
        .auth_service()
        .refresh_token(RefreshTokenCommand {
            base_url: persisted.context.base_url.clone(),
            refresh_token,
        })
        .await
    {
        Ok(value) => value,
        Err(error) => {
            if matches!(error.status(), Some(401 | 403)) {
                let _ = state
                    .sync_service()
                    .stop_polling_for_account(&persisted.context.account_id);
                let _ = state
                    .realtime_sync_service()
                    .stop_for_account(&persisted.context.account_id)
                    .await;
                let _ = state.clear_all_auth_state().await;
            }
            return Err(error);
        }
    };

    let account_id = account_id::derive_account_id_from_access_token(
        &persisted.context.base_url,
        &refreshed.access_token,
    )?;
    let next = AuthSession {
        account_id,
        base_url: persisted.context.base_url,
        email: persisted.context.email,
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token.or(Some(persisted.refresh_token)),
        expires_at_ms: calc_expires_at_ms(refreshed.expires_in)?,
        kdf: refreshed.kdf.or(persisted.context.kdf),
        kdf_iterations: refreshed
            .kdf_iterations
            .or(persisted.context.kdf_iterations),
        kdf_memory: refreshed.kdf_memory.or(persisted.context.kdf_memory),
        kdf_parallelism: refreshed
            .kdf_parallelism
            .or(persisted.context.kdf_parallelism),
    };

    state.set_auth_session(next.clone()).await?;
    if let Err(error) = state.persist_auth_state(&next, master_password) {
        log::warn!(
            target: "vanguard::tauri::session",
            "failed to refresh persisted auth state account_id={}: [{}] {}",
            next.account_id,
            error.code(),
            error.log_message()
        );
    }
    start_background_sync(state, &next).await;
    Ok(next)
}

async fn refresh_auth_session(state: &AppState, force: bool) -> AppResult<AuthSession> {
    loop {
        let current = state.require_auth_session().await?;
        if !force && !current.is_expiring_within(REFRESH_GRACE_PERIOD_MS) {
            return Ok(current);
        }

        let account_id = current.account_id.clone();
        let singleflight_lock = acquire_refresh_singleflight_lock(&account_id)?;
        let decision = {
            let _guard = singleflight_lock.lock().await;
            let latest = state.require_auth_session().await?;
            if latest.account_id != account_id {
                RefreshDecision::Retry
            } else if !force && !latest.is_expiring_within(REFRESH_GRACE_PERIOD_MS) {
                RefreshDecision::Return(latest)
            } else {
                let refreshed = refresh_auth_session_locked(state, latest).await?;
                RefreshDecision::Return(refreshed)
            }
        };
        cleanup_refresh_singleflight_lock(&account_id, &singleflight_lock);

        match decision {
            RefreshDecision::Retry => continue,
            RefreshDecision::Return(session) => return Ok(session),
        }
    }
}

async fn refresh_auth_session_locked(
    state: &AppState,
    current: AuthSession,
) -> AppResult<AuthSession> {
    let refresh_token =
        current
            .refresh_token
            .clone()
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "current session missing refresh token, please login again".to_string(),
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
                let _ = state.clear_all_auth_state().await;
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

    state.set_auth_session(next.clone()).await?;
    // Only persist if we have auth_wrap_runtime (i.e., unlocked with master password)
    if state.auth_wrap_runtime()?.is_some() {
        if let Err(error) = state.persist_auth_state_with_cached_wrap(&next) {
            log::warn!(
                target: "vanguard::tauri::session",
                "skip persisted auth refresh due to missing/invalid wrap runtime account_id={}: [{}] {}",
                next.account_id,
                error.code(),
                error.log_message()
            );
        }
    } else {
        log::debug!(
            target: "vanguard::tauri::session",
            "skip persist auth state (no wrap runtime) account_id={}",
            next.account_id
        );
    }
    start_background_sync(state, &next).await;
    Ok(next)
}

fn acquire_refresh_singleflight_lock(account_id: &str) -> AppResult<RefreshSingleflightLock> {
    let lock_store = REFRESH_SINGLEFLIGHT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locks = lock_store
        .lock()
        .map_err(|_| AppError::InternalUnexpected {
            message: "failed to lock refresh singleflight store".to_string(),
        })?;
    Ok(locks
        .entry(account_id.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone())
}

fn cleanup_refresh_singleflight_lock(account_id: &str, lock: &RefreshSingleflightLock) {
    if Arc::strong_count(lock) > 2 {
        return;
    }

    let Some(lock_store) = REFRESH_SINGLEFLIGHT_LOCKS.get() else {
        return;
    };

    if let Ok(mut locks) = lock_store.lock() {
        let should_remove = locks
            .get(account_id)
            .map(|existing| Arc::ptr_eq(existing, lock) && Arc::strong_count(existing) <= 2)
            .unwrap_or(false);
        if should_remove {
            locks.remove(account_id);
        }
    }
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

/// Restore auth session using a refresh token directly (for PIN/biometric unlock)
pub async fn restore_auth_session_with_refresh_token(
    state: &AppState,
    refresh_token: &str,
) -> AppResult<AuthSession> {
    let persisted_context =
        state
            .persisted_auth_context()?
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "no persisted login state found in backend, please login first"
                    .to_string(),
            })?;

    let refreshed = match state
        .auth_service()
        .refresh_token(RefreshTokenCommand {
            base_url: persisted_context.base_url.clone(),
            refresh_token: refresh_token.to_string(),
        })
        .await
    {
        Ok(value) => value,
        Err(error) => {
            if matches!(error.status(), Some(401 | 403)) {
                let _ = state
                    .sync_service()
                    .stop_polling_for_account(&persisted_context.account_id);
                let _ = state
                    .realtime_sync_service()
                    .stop_for_account(&persisted_context.account_id)
                    .await;
                let _ = state.clear_all_auth_state().await;
            }
            return Err(error);
        }
    };

    let account_id = account_id::derive_account_id_from_access_token(
        &persisted_context.base_url,
        &refreshed.access_token,
    )?;
    let next = AuthSession {
        account_id,
        base_url: persisted_context.base_url,
        email: persisted_context.email,
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token.or(Some(refresh_token.to_string())),
        expires_at_ms: calc_expires_at_ms(refreshed.expires_in)?,
        kdf: refreshed.kdf.or(persisted_context.kdf),
        kdf_iterations: refreshed
            .kdf_iterations
            .or(persisted_context.kdf_iterations),
        kdf_memory: refreshed.kdf_memory.or(persisted_context.kdf_memory),
        kdf_parallelism: refreshed
            .kdf_parallelism
            .or(persisted_context.kdf_parallelism),
    };

    state.set_auth_session(next.clone()).await?;
    // Only persist if we have auth_wrap_runtime (i.e., unlocked with master password)
    if state.auth_wrap_runtime()?.is_some() {
        if let Err(error) = state.persist_auth_state_with_cached_wrap(&next) {
            log::warn!(
                target: "vanguard::tauri::session",
                "failed to refresh persisted auth state account_id={}: [{}] {}",
                next.account_id,
                error.code(),
                error.log_message()
            );
        }
    } else {
        log::debug!(
            target: "vanguard::tauri::session",
            "skip persist auth state (no wrap runtime) account_id={}",
            next.account_id
        );
    }
    start_background_sync(state, &next).await;
    Ok(next)
}

fn calc_expires_at_ms(expires_in_seconds: i64) -> AppResult<i64> {
    let now_ms = now_unix_ms()?;
    let ttl_ms = expires_in_seconds.max(0).saturating_mul(1000);
    Ok(now_ms.saturating_add(ttl_ms))
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::InternalUnexpected {
            message: format!("system clock before unix epoch: {error}"),
        })?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}
