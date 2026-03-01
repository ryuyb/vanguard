use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Debug, Clone)]
pub struct VaultUserKey {
    pub enc_key: Vec<u8>,
    pub mac_key: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub account_id: String,
    pub base_url: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at_ms: i64,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

impl AuthSession {
    pub fn is_expiring_within(&self, duration_ms: i64) -> bool {
        match now_unix_ms() {
            Ok(now_ms) => now_ms.saturating_add(duration_ms) >= self.expires_at_ms,
            Err(_) => true,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
    sync_service: Arc<SyncService>,
    realtime_sync_service: Arc<RealtimeSyncService>,
    vault_user_keys: Arc<Mutex<HashMap<String, VaultUserKey>>>,
    auth_session: Arc<Mutex<Option<AuthSession>>>,
}

impl AppState {
    pub fn new(
        auth_service: Arc<AuthService>,
        sync_service: Arc<SyncService>,
        realtime_sync_service: Arc<RealtimeSyncService>,
    ) -> Self {
        Self {
            auth_service,
            sync_service,
            realtime_sync_service,
            vault_user_keys: Arc::new(Mutex::new(HashMap::new())),
            auth_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        Arc::clone(&self.auth_service)
    }

    pub fn sync_service(&self) -> Arc<SyncService> {
        Arc::clone(&self.sync_service)
    }

    pub fn realtime_sync_service(&self) -> Arc<RealtimeSyncService> {
        Arc::clone(&self.realtime_sync_service)
    }

    pub fn set_vault_user_key(&self, account_id: String, key: VaultUserKey) -> AppResult<()> {
        let mut store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::internal("failed to lock vault user key store"))?;
        store.insert(account_id, key);
        Ok(())
    }

    pub fn remove_vault_user_key(&self, account_id: &str) -> AppResult<()> {
        let mut store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::internal("failed to lock vault user key store"))?;
        store.remove(account_id);
        Ok(())
    }

    pub fn get_vault_user_key(&self, account_id: &str) -> AppResult<Option<VaultUserKey>> {
        let store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::internal("failed to lock vault user key store"))?;
        Ok(store.get(account_id).cloned())
    }

    pub fn set_auth_session(&self, session: AuthSession) -> AppResult<()> {
        let previous_account_id = {
            let mut store = self
                .auth_session
                .lock()
                .map_err(|_| AppError::internal("failed to lock auth session store"))?;
            let previous = store.as_ref().map(|value| value.account_id.clone());
            *store = Some(session.clone());
            previous
        };

        if let Some(previous_account_id) = previous_account_id {
            if previous_account_id != session.account_id {
                self.remove_vault_user_key(&previous_account_id)?;
            }
        }

        Ok(())
    }

    pub fn clear_auth_session(&self) -> AppResult<()> {
        let previous_account_id = {
            let mut store = self
                .auth_session
                .lock()
                .map_err(|_| AppError::internal("failed to lock auth session store"))?;
            let previous = store.as_ref().map(|value| value.account_id.clone());
            *store = None;
            previous
        };

        if let Some(account_id) = previous_account_id {
            self.remove_vault_user_key(&account_id)?;
        }

        Ok(())
    }

    pub fn auth_session(&self) -> AppResult<Option<AuthSession>> {
        let store = self
            .auth_session
            .lock()
            .map_err(|_| AppError::internal("failed to lock auth session store"))?;
        Ok(store.clone())
    }

    pub fn require_auth_session(&self) -> AppResult<AuthSession> {
        self.auth_session()?.ok_or_else(|| {
            AppError::validation("no authenticated session in backend, please login first")
        })
    }
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(format!("system clock before unix epoch: {error}")))?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}
