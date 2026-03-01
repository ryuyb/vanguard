use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
    sync_service: Arc<SyncService>,
    realtime_sync_service: Arc<RealtimeSyncService>,
    vault_user_keys: Arc<Mutex<HashMap<String, VaultUserKey>>>,
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
}
