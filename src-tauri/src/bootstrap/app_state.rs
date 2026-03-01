use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::bootstrap::auth_persistence::{
    decrypt_refresh_token, encrypt_refresh_token, encrypt_refresh_token_with_runtime,
    PersistedAuthState, SessionWrapRuntime,
};
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

#[derive(Debug, Clone)]
pub struct PersistedAuthContext {
    pub account_id: String,
    pub base_url: String,
    pub email: String,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct PersistedAuthSecret {
    pub context: PersistedAuthContext,
    pub refresh_token: String,
}

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
    sync_service: Arc<SyncService>,
    realtime_sync_service: Arc<RealtimeSyncService>,
    vault_user_keys: Arc<Mutex<HashMap<String, VaultUserKey>>>,
    auth_session: Arc<Mutex<Option<AuthSession>>>,
    auth_state_path: Arc<PathBuf>,
    persisted_auth_state: Arc<Mutex<Option<PersistedAuthState>>>,
    auth_wrap_runtime: Arc<Mutex<Option<SessionWrapRuntime>>>,
}

impl AppState {
    pub fn new(
        auth_service: Arc<AuthService>,
        sync_service: Arc<SyncService>,
        realtime_sync_service: Arc<RealtimeSyncService>,
        auth_state_path: PathBuf,
    ) -> Self {
        let persisted_auth_state = match load_persisted_auth_state_from_disk(&auth_state_path) {
            Ok(value) => value,
            Err(error) => {
                log::warn!(
                    target: "vanguard::bootstrap",
                    "failed to load persisted auth state {}: [{}] {}",
                    auth_state_path.display(),
                    error.code(),
                    error.log_message()
                );
                None
            }
        };

        Self {
            auth_service,
            sync_service,
            realtime_sync_service,
            vault_user_keys: Arc::new(Mutex::new(HashMap::new())),
            auth_session: Arc::new(Mutex::new(None)),
            auth_state_path: Arc::new(auth_state_path),
            persisted_auth_state: Arc::new(Mutex::new(persisted_auth_state)),
            auth_wrap_runtime: Arc::new(Mutex::new(None)),
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
        self.clear_auth_wrap_runtime()?;

        Ok(())
    }

    pub fn clear_all_auth_state(&self) -> AppResult<()> {
        self.clear_auth_session()?;
        self.clear_persisted_auth_state()
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

    pub fn active_account_id(&self) -> AppResult<String> {
        if let Some(session) = self.auth_session()? {
            return Ok(session.account_id);
        }
        self.persisted_auth_context()?
            .map(|value| value.account_id)
            .ok_or_else(|| {
                AppError::validation("no cached account context in backend, please login first")
            })
    }

    pub fn persist_auth_state(
        &self,
        session: &AuthSession,
        master_password: &str,
    ) -> AppResult<()> {
        let refresh_token = session.refresh_token.clone().ok_or_else(|| {
            AppError::validation("refresh_token is missing, cannot persist auth state")
        })?;
        let (encrypted_session, runtime) = encrypt_refresh_token(
            master_password,
            &session.account_id,
            &session.base_url,
            &session.email,
            &refresh_token,
        )?;
        self.set_auth_wrap_runtime(Some(runtime))?;
        let persisted = PersistedAuthState::new(
            session.account_id.clone(),
            session.base_url.clone(),
            session.email.clone(),
            session.kdf,
            session.kdf_iterations,
            session.kdf_memory,
            session.kdf_parallelism,
            encrypted_session,
        )?;
        self.store_persisted_auth_state(Some(persisted))
    }

    pub fn persist_auth_state_with_cached_wrap(&self, session: &AuthSession) -> AppResult<()> {
        let refresh_token = session.refresh_token.clone().ok_or_else(|| {
            AppError::validation("refresh_token is missing, cannot persist auth state")
        })?;
        let runtime = self.auth_wrap_runtime()?.ok_or_else(|| {
            AppError::validation(
                "session wrap runtime is missing, cannot persist auth state without master password",
            )
        })?;
        let encrypted_session = encrypt_refresh_token_with_runtime(
            &runtime,
            &session.account_id,
            &session.base_url,
            &session.email,
            &refresh_token,
        )?;
        let persisted = PersistedAuthState::new(
            session.account_id.clone(),
            session.base_url.clone(),
            session.email.clone(),
            session.kdf,
            session.kdf_iterations,
            session.kdf_memory,
            session.kdf_parallelism,
            encrypted_session,
        )?;
        self.store_persisted_auth_state(Some(persisted))
    }

    pub fn clear_persisted_auth_state(&self) -> AppResult<()> {
        self.clear_auth_wrap_runtime()?;
        self.store_persisted_auth_state(None)
    }

    pub fn persisted_auth_context(&self) -> AppResult<Option<PersistedAuthContext>> {
        let state = self.load_cached_persisted_auth_state()?;
        Ok(state.map(|value| PersistedAuthContext {
            account_id: value.account_id,
            base_url: value.base_url,
            email: value.email,
            kdf: value.kdf,
            kdf_iterations: value.kdf_iterations,
            kdf_memory: value.kdf_memory,
            kdf_parallelism: value.kdf_parallelism,
        }))
    }

    pub fn decrypt_persisted_auth_secret(
        &self,
        master_password: &str,
    ) -> AppResult<Option<PersistedAuthSecret>> {
        let state = match self.load_cached_persisted_auth_state()? {
            Some(value) => value,
            None => return Ok(None),
        };

        let (refresh_token, runtime) = decrypt_refresh_token(
            master_password,
            &state.account_id,
            &state.base_url,
            &state.email,
            &state.encrypted_session,
        )?;
        self.set_auth_wrap_runtime(Some(runtime))?;

        Ok(Some(PersistedAuthSecret {
            context: PersistedAuthContext {
                account_id: state.account_id,
                base_url: state.base_url,
                email: state.email,
                kdf: state.kdf,
                kdf_iterations: state.kdf_iterations,
                kdf_memory: state.kdf_memory,
                kdf_parallelism: state.kdf_parallelism,
            },
            refresh_token,
        }))
    }

    pub fn auth_wrap_runtime(&self) -> AppResult<Option<SessionWrapRuntime>> {
        let store = self
            .auth_wrap_runtime
            .lock()
            .map_err(|_| AppError::internal("failed to lock auth wrap runtime store"))?;
        Ok(store.clone())
    }

    pub fn set_auth_wrap_runtime(&self, value: Option<SessionWrapRuntime>) -> AppResult<()> {
        let mut store = self
            .auth_wrap_runtime
            .lock()
            .map_err(|_| AppError::internal("failed to lock auth wrap runtime store"))?;
        *store = value;
        Ok(())
    }

    fn clear_auth_wrap_runtime(&self) -> AppResult<()> {
        self.set_auth_wrap_runtime(None)
    }

    fn load_cached_persisted_auth_state(&self) -> AppResult<Option<PersistedAuthState>> {
        let store = self
            .persisted_auth_state
            .lock()
            .map_err(|_| AppError::internal("failed to lock persisted auth state store"))?;
        Ok(store.clone())
    }

    fn store_persisted_auth_state(&self, value: Option<PersistedAuthState>) -> AppResult<()> {
        persist_persisted_auth_state_to_disk(self.auth_state_path.as_ref(), value.as_ref())?;
        let mut store = self
            .persisted_auth_state
            .lock()
            .map_err(|_| AppError::internal("failed to lock persisted auth state store"))?;
        *store = value;
        Ok(())
    }
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(format!("system clock before unix epoch: {error}")))?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}

fn load_persisted_auth_state_from_disk(path: &Path) -> AppResult<Option<PersistedAuthState>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).map_err(|error| {
        AppError::internal(format!(
            "failed to read persisted auth state {}: {error}",
            path.display()
        ))
    })?;
    let parsed = serde_json::from_str::<PersistedAuthState>(&raw).map_err(|error| {
        AppError::internal(format!(
            "failed to parse persisted auth state {}: {error}",
            path.display()
        ))
    })?;
    Ok(Some(parsed))
}

fn persist_persisted_auth_state_to_disk(
    path: &Path,
    value: Option<&PersistedAuthState>,
) -> AppResult<()> {
    match value {
        None => {
            if path.exists() {
                std::fs::remove_file(path).map_err(|error| {
                    AppError::internal(format!(
                        "failed to delete persisted auth state {}: {error}",
                        path.display()
                    ))
                })?;
            }
            Ok(())
        }
        Some(value) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    AppError::internal(format!(
                        "failed to create auth state dir {}: {error}",
                        parent.display()
                    ))
                })?;
            }
            let serialized = serde_json::to_vec_pretty(value).map_err(|error| {
                AppError::internal(format!(
                    "failed to serialize persisted auth state {}: {error}",
                    path.display()
                ))
            })?;
            let mut temp_path = path.to_path_buf();
            temp_path.set_extension("json.tmp");
            std::fs::write(&temp_path, serialized).map_err(|error| {
                AppError::internal(format!(
                    "failed to write temp auth state {}: {error}",
                    temp_path.display()
                ))
            })?;
            std::fs::rename(&temp_path, path).map_err(|error| {
                AppError::internal(format!(
                    "failed to persist auth state {}: {error}",
                    path.display()
                ))
            })?;
            Ok(())
        }
    }
}
