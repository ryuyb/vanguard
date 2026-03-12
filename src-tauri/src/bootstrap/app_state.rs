use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::application::ports::clipboard_port::ClipboardPort;
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::pin_unlock_port::PinUnlockPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::create_cipher_use_case::CreateCipherUseCase;
use crate::application::use_cases::delete_cipher_use_case::DeleteCipherUseCase;
use crate::application::use_cases::fetch_cipher_use_case::FetchCipherUseCase;
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::application::use_cases::update_cipher_use_case::UpdateCipherUseCase;
use crate::bootstrap::auth_persistence::{
    decrypt_refresh_token, encrypt_refresh_token, encrypt_refresh_token_with_runtime,
    PersistedAuthState, PersistedAuthStateContext, SessionWrapRuntime,
};
use crate::infrastructure::vaultwarden::VaultwardenClient;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
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
    vaultwarden_client: VaultwardenClient,
    master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>,
    pin_unlock_port: Arc<dyn PinUnlockPort>,
    biometric_unlock_port: Arc<dyn BiometricUnlockPort>,
    clipboard_port: Arc<dyn ClipboardPort>,
    get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>,
    create_cipher_use_case: Arc<CreateCipherUseCase>,
    update_cipher_use_case: Arc<UpdateCipherUseCase>,
    delete_cipher_use_case: Arc<DeleteCipherUseCase>,
    fetch_cipher_use_case: Arc<FetchCipherUseCase>,
    vault_user_keys: Arc<Mutex<HashMap<String, VaultUserKey>>>,
    auth_session: Arc<Mutex<Option<AuthSession>>>,
    auth_state_path: Arc<PathBuf>,
    auth_state_persist_lock: Arc<Mutex<()>>,
    persisted_auth_state: Arc<Mutex<Option<PersistedAuthState>>>,
    auth_wrap_runtime: Arc<Mutex<Option<SessionWrapRuntime>>>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_service: Arc<AuthService>,
        sync_service: Arc<SyncService>,
        realtime_sync_service: Arc<RealtimeSyncService>,
        vaultwarden_client: VaultwardenClient,
        master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>,
        pin_unlock_port: Arc<dyn PinUnlockPort>,
        biometric_unlock_port: Arc<dyn BiometricUnlockPort>,
        clipboard_port: Arc<dyn ClipboardPort>,
        get_cipher_detail_use_case: Arc<GetCipherDetailUseCase>,
        create_cipher_use_case: Arc<CreateCipherUseCase>,
        update_cipher_use_case: Arc<UpdateCipherUseCase>,
        delete_cipher_use_case: Arc<DeleteCipherUseCase>,
        fetch_cipher_use_case: Arc<FetchCipherUseCase>,
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
            vaultwarden_client,
            master_password_unlock_data_port,
            pin_unlock_port,
            biometric_unlock_port,
            clipboard_port,
            get_cipher_detail_use_case,
            create_cipher_use_case,
            update_cipher_use_case,
            delete_cipher_use_case,
            fetch_cipher_use_case,
            vault_user_keys: Arc::new(Mutex::new(HashMap::new())),
            auth_session: Arc::new(Mutex::new(None)),
            auth_state_path: Arc::new(auth_state_path),
            auth_state_persist_lock: Arc::new(Mutex::new(())),
            persisted_auth_state: Arc::new(Mutex::new(persisted_auth_state)),
            auth_wrap_runtime: Arc::new(Mutex::new(None)),
        }
    }

    pub fn vaultwarden_client(&self) -> &VaultwardenClient {
        &self.vaultwarden_client
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

    pub fn biometric_unlock_port(&self) -> Arc<dyn BiometricUnlockPort> {
        Arc::clone(&self.biometric_unlock_port)
    }

    pub fn master_password_unlock_data_port(&self) -> Arc<dyn MasterPasswordUnlockDataPort> {
        Arc::clone(&self.master_password_unlock_data_port)
    }

    pub fn pin_unlock_port(&self) -> Arc<dyn PinUnlockPort> {
        Arc::clone(&self.pin_unlock_port)
    }

    pub fn clipboard_port(&self) -> Arc<dyn ClipboardPort> {
        Arc::clone(&self.clipboard_port)
    }

    pub fn get_cipher_detail_use_case(&self) -> Arc<GetCipherDetailUseCase> {
        Arc::clone(&self.get_cipher_detail_use_case)
    }

    pub fn create_cipher_use_case(&self) -> Arc<CreateCipherUseCase> {
        Arc::clone(&self.create_cipher_use_case)
    }

    pub fn update_cipher_use_case(&self) -> Arc<UpdateCipherUseCase> {
        Arc::clone(&self.update_cipher_use_case)
    }

    pub fn delete_cipher_use_case(&self) -> Arc<DeleteCipherUseCase> {
        Arc::clone(&self.delete_cipher_use_case)
    }

    pub fn fetch_cipher_use_case(&self) -> Arc<FetchCipherUseCase> {
        Arc::clone(&self.fetch_cipher_use_case)
    }

    pub fn set_vault_user_key(&self, account_id: String, key: VaultUserKey) -> AppResult<()> {
        let mut store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock vault user key store".to_string(),
            })?;
        store.insert(account_id, key);
        Ok(())
    }

    pub fn remove_vault_user_key(&self, account_id: &str) -> AppResult<()> {
        let mut store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock vault user key store".to_string(),
            })?;
        store.remove(account_id);
        Ok(())
    }

    pub fn get_vault_user_key(&self, account_id: &str) -> AppResult<Option<VaultUserKey>> {
        let store = self
            .vault_user_keys
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock vault user key store".to_string(),
            })?;
        Ok(store.get(account_id).cloned())
    }

    pub fn set_auth_session(&self, session: AuthSession) -> AppResult<()> {
        let previous_account_id = {
            let mut store = self
                .auth_session
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock auth session store".to_string(),
                })?;
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
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock auth session store".to_string(),
                })?;
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
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock auth session store".to_string(),
            })?;
        Ok(store.clone())
    }

    pub fn require_auth_session(&self) -> AppResult<AuthSession> {
        self.auth_session()?
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "no authenticated session in backend, please login first".to_string(),
            })
    }

    pub fn active_account_id(&self) -> AppResult<String> {
        if let Some(session) = self.auth_session()? {
            return Ok(session.account_id);
        }
        self.persisted_auth_context()?
            .map(|value| value.account_id)
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "no cached account context in backend, please login first".to_string(),
            })
    }

    pub fn persist_auth_state(
        &self,
        session: &AuthSession,
        master_password: &str,
    ) -> AppResult<()> {
        let refresh_token =
            session
                .refresh_token
                .clone()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "refresh_token is missing, cannot persist auth state".to_string(),
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
            persisted_auth_state_context_from_session(session),
            encrypted_session,
        )?;
        self.store_persisted_auth_state(Some(persisted))
    }

    pub fn persist_auth_state_with_cached_wrap(&self, session: &AuthSession) -> AppResult<()> {
        let refresh_token =
            session
                .refresh_token
                .clone()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "refresh_token is missing, cannot persist auth state".to_string(),
                })?;
        let runtime = self.auth_wrap_runtime()?.ok_or_else(|| {
            AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: "session wrap runtime is missing, cannot persist auth state without master password".to_string(),
            }
        })?;
        let encrypted_session = encrypt_refresh_token_with_runtime(
            &runtime,
            &session.account_id,
            &session.base_url,
            &session.email,
            &refresh_token,
        )?;
        let persisted = PersistedAuthState::new(
            persisted_auth_state_context_from_session(session),
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
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock auth wrap runtime store".to_string(),
            })?;
        Ok(store.clone())
    }

    pub fn set_auth_wrap_runtime(&self, value: Option<SessionWrapRuntime>) -> AppResult<()> {
        let mut store =
            self.auth_wrap_runtime
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock auth wrap runtime store".to_string(),
                })?;
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
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock persisted auth state store".to_string(),
            })?;
        Ok(store.clone())
    }

    fn store_persisted_auth_state(&self, value: Option<PersistedAuthState>) -> AppResult<()> {
        let _persist_guard =
            self.auth_state_persist_lock
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock auth state persistence".to_string(),
                })?;
        persist_persisted_auth_state_to_disk(self.auth_state_path.as_ref(), value.as_ref())?;
        let mut store =
            self.persisted_auth_state
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock persisted auth state store".to_string(),
                })?;
        *store = value;
        Ok(())
    }
}

impl VaultRuntimePort for AppState {
    fn active_account_id(&self) -> AppResult<String> {
        AppState::active_account_id(self)
    }

    fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>> {
        self.auth_session().map(|value| {
            value.map(|session| VaultUnlockContext {
                account_id: session.account_id,
                base_url: session.base_url,
                email: session.email,
                kdf: session.kdf,
                kdf_iterations: session.kdf_iterations,
                kdf_memory: session.kdf_memory,
                kdf_parallelism: session.kdf_parallelism,
            })
        })
    }

    fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>> {
        AppState::persisted_auth_context(self).map(|value| {
            value.map(|persisted| VaultUnlockContext {
                account_id: persisted.account_id,
                base_url: persisted.base_url,
                email: persisted.email,
                kdf: persisted.kdf,
                kdf_iterations: persisted.kdf_iterations,
                kdf_memory: persisted.kdf_memory,
                kdf_parallelism: persisted.kdf_parallelism,
            })
        })
    }

    fn get_vault_user_key_material(
        &self,
        account_id: &str,
    ) -> AppResult<Option<VaultUserKeyMaterial>> {
        self.get_vault_user_key(account_id).map(|value| {
            value.map(|key| VaultUserKeyMaterial {
                enc_key: key.enc_key.clone(),
                mac_key: key.mac_key.clone(),
            })
        })
    }

    fn set_vault_user_key_material(
        &self,
        account_id: String,
        key: VaultUserKeyMaterial,
    ) -> AppResult<()> {
        self.set_vault_user_key(
            account_id,
            VaultUserKey {
                enc_key: key.enc_key,
                mac_key: key.mac_key,
            },
        )
    }

    fn remove_vault_user_key_material(&self, account_id: &str) -> AppResult<()> {
        self.remove_vault_user_key(account_id)
    }
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::InternalUnexpected {
            message: format!("system clock before unix epoch: {error}"),
        })?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}

fn load_persisted_auth_state_from_disk(path: &Path) -> AppResult<Option<PersistedAuthState>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).map_err(|error| AppError::InternalUnexpected {
        message: format!(
            "failed to read persisted auth state {}: {error}",
            path.display()
        ),
    })?;
    let parsed = serde_json::from_str::<PersistedAuthState>(&raw).map_err(|error| {
        AppError::InternalUnexpected {
            message: format!(
                "failed to parse persisted auth state {}: {error}",
                path.display()
            ),
        }
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
                std::fs::remove_file(path).map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to delete persisted auth state {}: {error}",
                        path.display()
                    ),
                })?;
            }
            Ok(())
        }
        Some(value) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to create auth state dir {}: {error}",
                        parent.display()
                    ),
                })?;
            }
            let serialized =
                serde_json::to_vec_pretty(value).map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to serialize persisted auth state {}: {error}",
                        path.display()
                    ),
                })?;
            let temp_path = build_temp_auth_state_path(path);
            std::fs::write(&temp_path, serialized).map_err(|error| {
                AppError::InternalUnexpected {
                    message: format!(
                        "failed to write temp auth state {}: {error}",
                        temp_path.display()
                    ),
                }
            })?;
            std::fs::rename(&temp_path, path).map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to persist auth state {}: {error}", path.display()),
            })?;
            Ok(())
        }
    }
}

static AUTH_STATE_TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_auth_state_temp_file_id() -> u64 {
    AUTH_STATE_TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn build_temp_auth_state_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("auth-state.json");
    let temp_file_name = format!(
        "{file_name}.tmp.{}.{}",
        std::process::id(),
        next_auth_state_temp_file_id()
    );
    match path.parent() {
        Some(parent) => parent.join(temp_file_name),
        None => PathBuf::from(temp_file_name),
    }
}

fn persisted_auth_state_context_from_session(session: &AuthSession) -> PersistedAuthStateContext {
    PersistedAuthStateContext {
        account_id: session.account_id.clone(),
        base_url: session.base_url.clone(),
        email: session.email.clone(),
        kdf: session.kdf,
        kdf_iterations: session.kdf_iterations,
        kdf_memory: session.kdf_memory,
        kdf_parallelism: session.kdf_parallelism,
    }
}
