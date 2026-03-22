use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::application::ports::clipboard_port::ClipboardPort;
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::pin_unlock_port::PinUnlockPort;
use crate::application::ports::text_injection_port::TextInjectionPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::create_cipher_use_case::CreateCipherUseCase;
use crate::application::use_cases::delete_cipher_use_case::DeleteCipherUseCase;
use crate::application::use_cases::fetch_cipher_use_case::FetchCipherUseCase;
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::application::use_cases::restore_cipher_use_case::RestoreCipherUseCase;
use crate::application::use_cases::soft_delete_cipher_use_case::SoftDeleteCipherUseCase;
use crate::application::use_cases::update_cipher_use_case::UpdateCipherUseCase;
use crate::bootstrap::auth_persistence::{
    decrypt_refresh_token, encrypt_refresh_token, encrypt_refresh_token_with_runtime,
    PersistedAuthState, PersistedAuthStateContext, SessionWrapRuntime,
};
use crate::bootstrap::config::AppConfig;
use crate::bootstrap::unlock_state::{
    AccountContext, SessionContext, UnifiedUnlockManager, VaultKeyMaterial,
};
use crate::infrastructure::desktop::FocusTracker;
use crate::infrastructure::icon::IconService;
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

// Conversion helpers between legacy types and UnifiedUnlockManager types

impl From<AuthSession> for AccountContext {
    fn from(session: AuthSession) -> Self {
        Self {
            account_id: session.account_id,
            email: session.email,
            base_url: session.base_url,
            kdf: session.kdf,
            kdf_iterations: session.kdf_iterations,
            kdf_memory: session.kdf_memory,
            kdf_parallelism: session.kdf_parallelism,
        }
    }
}

impl From<&AuthSession> for AccountContext {
    fn from(session: &AuthSession) -> Self {
        Self {
            account_id: session.account_id.clone(),
            email: session.email.clone(),
            base_url: session.base_url.clone(),
            kdf: session.kdf,
            kdf_iterations: session.kdf_iterations,
            kdf_memory: session.kdf_memory,
            kdf_parallelism: session.kdf_parallelism,
        }
    }
}

impl From<AuthSession> for SessionContext {
    fn from(session: AuthSession) -> Self {
        use std::time::{Duration, Instant};
        let expires_at = Instant::now()
            + Duration::from_millis(
                session
                    .expires_at_ms
                    .saturating_sub(now_unix_ms().unwrap_or(0))
                    .max(0) as u64,
            );
        Self {
            access_token: session.access_token,
            refresh_token: session.refresh_token,
            expires_at,
            last_activity: Instant::now(),
        }
    }
}

impl From<&AuthSession> for SessionContext {
    fn from(session: &AuthSession) -> Self {
        use std::time::{Duration, Instant};
        let expires_at = Instant::now()
            + Duration::from_millis(
                session
                    .expires_at_ms
                    .saturating_sub(now_unix_ms().unwrap_or(0))
                    .max(0) as u64,
            );
        Self {
            access_token: session.access_token.clone(),
            refresh_token: session.refresh_token.clone(),
            expires_at,
            last_activity: Instant::now(),
        }
    }
}

impl From<VaultUserKey> for VaultKeyMaterial {
    fn from(key: VaultUserKey) -> Self {
        Self {
            enc_key: key.enc_key.clone(),
            mac_key: key.mac_key.clone(),
        }
    }
}

impl From<VaultKeyMaterial> for VaultUserKey {
    fn from(key: VaultKeyMaterial) -> Self {
        Self {
            enc_key: key.enc_key.clone(),
            mac_key: key.mac_key.clone(),
        }
    }
}

// Convert from SessionContext back to AuthSession (requires account context)
fn session_context_to_auth_session(
    session_ctx: &SessionContext,
    account_ctx: &AccountContext,
) -> AuthSession {
    // Calculate expires_at_ms based on the remaining duration from now
    let now_instant = Instant::now();
    let now_ms = now_unix_ms().unwrap_or(0);
    let expires_at_ms = if session_ctx.expires_at > now_instant {
        now_ms
            + session_ctx
                .expires_at
                .duration_since(now_instant)
                .as_millis() as i64
    } else {
        now_ms
    };

    AuthSession {
        account_id: account_ctx.account_id.clone(),
        base_url: account_ctx.base_url.clone(),
        email: account_ctx.email.clone(),
        access_token: session_ctx.access_token.clone(),
        refresh_token: session_ctx.refresh_token.clone(),
        expires_at_ms,
        kdf: account_ctx.kdf,
        kdf_iterations: account_ctx.kdf_iterations,
        kdf_memory: account_ctx.kdf_memory,
        kdf_parallelism: account_ctx.kdf_parallelism,
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
    soft_delete_cipher_use_case: Arc<SoftDeleteCipherUseCase>,
    restore_cipher_use_case: Arc<RestoreCipherUseCase>,
    fetch_cipher_use_case: Arc<FetchCipherUseCase>,
    text_injection_port: Arc<dyn TextInjectionPort>,
    // Unified unlock state manager - replaces vault_user_keys and auth_session
    unlock_manager: Arc<UnifiedUnlockManager>,
    auth_states_dir: Arc<PathBuf>,
    auth_state_persist_lock: Arc<Mutex<()>>,
    persisted_auth_state: Arc<Mutex<Option<PersistedAuthState>>>,
    auth_wrap_runtime: Arc<Mutex<Option<SessionWrapRuntime>>>,
    icon_service: Arc<IconService>,
    focus_tracker: Arc<Mutex<FocusTracker>>,
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
        soft_delete_cipher_use_case: Arc<SoftDeleteCipherUseCase>,
        restore_cipher_use_case: Arc<RestoreCipherUseCase>,
        fetch_cipher_use_case: Arc<FetchCipherUseCase>,
        text_injection_port: Arc<dyn TextInjectionPort>,
        auth_states_dir: PathBuf,
        config: AppConfig,
    ) -> Self {
        let persisted_auth_state = match load_active_persisted_auth_state(&auth_states_dir) {
            Ok(value) => value,
            Err(error) => {
                log::warn!(
                    target: "vanguard::bootstrap",
                    "failed to load active persisted auth state from {}: [{}] {}",
                    auth_states_dir.display(),
                    error.code(),
                    error.log_message()
                );
                None
            }
        };

        // Build initial account context from persisted auth state if available
        let initial_account = persisted_auth_state
            .as_ref()
            .map(|persisted| AccountContext {
                account_id: persisted.account_id.clone(),
                email: persisted.email.clone(),
                base_url: persisted.base_url.clone(),
                kdf: persisted.kdf,
                kdf_iterations: persisted.kdf_iterations,
                kdf_memory: persisted.kdf_memory,
                kdf_parallelism: persisted.kdf_parallelism,
            });

        // Initialize unified unlock manager with config and initial account context
        let unlock_manager = UnifiedUnlockManager::new(config, initial_account);

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
            soft_delete_cipher_use_case,
            restore_cipher_use_case,
            fetch_cipher_use_case,
            text_injection_port,
            unlock_manager,
            auth_states_dir: Arc::new(auth_states_dir),
            auth_state_persist_lock: Arc::new(Mutex::new(())),
            persisted_auth_state: Arc::new(Mutex::new(persisted_auth_state)),
            auth_wrap_runtime: Arc::new(Mutex::new(None)),
            icon_service: Arc::new(IconService::new().expect("Failed to create IconService")),
            focus_tracker: Arc::new(Mutex::new(FocusTracker::new())),
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

    pub fn soft_delete_cipher_use_case(&self) -> Arc<SoftDeleteCipherUseCase> {
        Arc::clone(&self.soft_delete_cipher_use_case)
    }

    pub fn restore_cipher_use_case(&self) -> Arc<RestoreCipherUseCase> {
        Arc::clone(&self.restore_cipher_use_case)
    }

    pub fn fetch_cipher_use_case(&self) -> Arc<FetchCipherUseCase> {
        Arc::clone(&self.fetch_cipher_use_case)
    }

    pub fn icon_service(&self) -> Arc<IconService> {
        Arc::clone(&self.icon_service)
    }

    pub fn focus_tracker(&self) -> Arc<Mutex<FocusTracker>> {
        Arc::clone(&self.focus_tracker)
    }

    pub fn text_injection_port(&self) -> Arc<dyn TextInjectionPort> {
        Arc::clone(&self.text_injection_port)
    }

    pub fn unlock_manager(&self) -> Arc<UnifiedUnlockManager> {
        Arc::clone(&self.unlock_manager)
    }

    // Legacy methods - now delegate to UnifiedUnlockManager

    pub async fn set_vault_user_key(
        &self,
        _account_id: String,
        key: VaultUserKey,
    ) -> AppResult<()> {
        // Store key material in unlock manager
        let key_material: VaultKeyMaterial = key.into();
        self.unlock_manager.set_key_material(key_material).await?;

        // Also ensure account context is set if we have it in persisted state
        if self.unlock_manager.account_context().await.is_none() {
            if let Ok(Some(persisted)) = self.persisted_auth_context() {
                let account_ctx = AccountContext {
                    account_id: persisted.account_id,
                    email: persisted.email,
                    base_url: persisted.base_url,
                    kdf: persisted.kdf,
                    kdf_iterations: persisted.kdf_iterations,
                    kdf_memory: persisted.kdf_memory,
                    kdf_parallelism: persisted.kdf_parallelism,
                };
                self.unlock_manager.set_account_context(account_ctx).await?;
            }
        }

        Ok(())
    }

    pub async fn remove_vault_user_key(&self, _account_id: &str) -> AppResult<()> {
        // Remove key material from unlock manager
        self.unlock_manager.remove_key_material().await?;
        Ok(())
    }

    pub async fn get_vault_user_key(&self, _account_id: &str) -> AppResult<Option<VaultUserKey>> {
        // Get key material from unlock manager
        let key_material = self.unlock_manager.key_material().await;
        Ok(key_material.map(|k| k.into()))
    }

    pub async fn set_auth_session(&self, session: AuthSession) -> AppResult<()> {
        // Get previous account ID from unlock manager
        let previous_account_id = self.unlock_manager.active_account_id().await.ok();

        // Convert AuthSession to AccountContext and SessionContext
        let account_ctx: AccountContext = (&session).into();
        let session_ctx: SessionContext = (&session).into();

        // Update unlock manager with new session
        self.unlock_manager
            .set_account_context(account_ctx.clone())
            .await?;
        self.unlock_manager.update_session(session_ctx).await?;

        // If account changed, remove old vault key
        if let Some(prev_id) = previous_account_id {
            if prev_id != session.account_id {
                self.unlock_manager.remove_key_material().await?;
            }
        }

        // Initialize icon downloader with the new base URL
        self.icon_service.set_downloader(&session.base_url).await;

        Ok(())
    }

    pub async fn clear_auth_session(&self) -> AppResult<()> {
        // Get previous account ID before clearing
        let previous_account_id = self.unlock_manager.active_account_id().await.ok();

        // Clear session from unlock manager (keeps account context and vault keys)
        self.unlock_manager.clear_session().await?;

        // Remove vault key for the previous account
        if let Some(_account_id) = previous_account_id {
            self.unlock_manager.remove_key_material().await?;
        }

        self.clear_auth_wrap_runtime()?;

        Ok(())
    }

    pub async fn clear_all_auth_state(&self) -> AppResult<()> {
        self.clear_auth_session().await?;
        self.clear_persisted_auth_state()
    }

    pub async fn auth_session(&self) -> AppResult<Option<AuthSession>> {
        // Reconstruct AuthSession from unlock manager state
        let account_ctx = self.unlock_manager.account_context().await;
        let session_ctx = self.unlock_manager.session_context().await;

        match (account_ctx, session_ctx) {
            (Some(account), Some(session)) => {
                let auth_session = session_context_to_auth_session(&session, &account);
                Ok(Some(auth_session))
            }
            _ => Ok(None),
        }
    }

    pub async fn require_auth_session(&self) -> AppResult<AuthSession> {
        self.auth_session()
            .await?
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "session".to_string(),
                message: "API session expired. Please lock and unlock with master password to restore API access.".to_string(),
            })
    }

    pub fn active_account_id(&self) -> AppResult<String> {
        // Try to get from unlock manager (blocking)
        if let Ok(account_id) = self.unlock_manager.active_account_id_blocking() {
            return Ok(account_id);
        }

        // Fall back to persisted context
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
        self.store_persisted_auth_state(&session.account_id, Some(persisted))
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
        self.store_persisted_auth_state(&session.account_id, Some(persisted))
    }

    pub fn clear_persisted_auth_state(&self) -> AppResult<()> {
        let account_id = self.active_account_id()?;
        self.clear_auth_wrap_runtime()?;
        self.store_persisted_auth_state(&account_id, None)?;
        refresh_active_account_after_logout(self.auth_states_dir.as_ref(), &account_id)?;
        Ok(())
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

    fn store_persisted_auth_state(
        &self,
        account_id: &str,
        value: Option<PersistedAuthState>,
    ) -> AppResult<()> {
        let _persist_guard =
            self.auth_state_persist_lock
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock auth state persistence".to_string(),
                })?;
        persist_persisted_auth_state_to_disk(
            self.auth_states_dir.as_ref(),
            account_id,
            value.as_ref(),
        )?;

        // Update active account index when persisting auth state
        if value.is_some() {
            update_active_account_index(self.auth_states_dir.as_ref(), account_id)?;
        }

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
        // Use block_in_place to allow calling async code from sync context
        // even when already inside a tokio runtime
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            let account_ctx = handle.block_on(self.unlock_manager.account_context());
            Ok(account_ctx.map(|ctx| VaultUnlockContext {
                account_id: ctx.account_id,
                base_url: ctx.base_url,
                email: ctx.email,
                kdf: ctx.kdf,
                kdf_iterations: ctx.kdf_iterations,
                kdf_memory: ctx.kdf_memory,
                kdf_parallelism: ctx.kdf_parallelism,
            }))
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
        _account_id: &str,
    ) -> AppResult<Option<VaultUserKeyMaterial>> {
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            let key_material = handle.block_on(self.unlock_manager.key_material_dto());
            Ok(key_material)
        })
    }

    fn set_vault_user_key_material(
        &self,
        _account_id: String,
        key: VaultUserKeyMaterial,
    ) -> AppResult<()> {
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            let key_material: VaultKeyMaterial = key.into();
            handle.block_on(self.unlock_manager.set_key_material(key_material))
        })
    }

    fn remove_vault_user_key_material(&self, _account_id: &str) -> AppResult<()> {
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(self.unlock_manager.remove_key_material())
        })
    }

    fn get_refresh_token(&self) -> AppResult<Option<String>> {
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            let token = handle.block_on(self.unlock_manager.refresh_token());
            Ok(token)
        })
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

fn load_active_persisted_auth_state(
    auth_states_dir: &Path,
) -> AppResult<Option<PersistedAuthState>> {
    let active_path = auth_states_dir.join("active.json");
    if !active_path.exists() {
        return Ok(None);
    }

    let active_raw =
        std::fs::read_to_string(&active_path).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to read active account index {}: {error}",
                active_path.display()
            ),
        })?;

    let active_data: serde_json::Value =
        serde_json::from_str(&active_raw).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to parse active account index {}: {error}",
                active_path.display()
            ),
        })?;

    let account_id = active_data
        .get("accountId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InternalUnexpected {
            message: format!(
                "active account index missing accountId field: {}",
                active_path.display()
            ),
        })?;

    load_persisted_auth_state_from_disk(auth_states_dir, account_id)
}

fn sanitize_account_id_for_filename(account_id: &str) -> String {
    account_id
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace(':', "%3A")
        .replace('\\', "%5C")
}

fn unsanitize_account_id_from_filename(sanitized: &str) -> String {
    sanitized
        .replace("%5C", "\\")
        .replace("%3A", ":")
        .replace("%2F", "/")
        .replace("%25", "%")
}

fn load_persisted_auth_state_from_disk(
    auth_states_dir: &Path,
    account_id: &str,
) -> AppResult<Option<PersistedAuthState>> {
    let safe_name = sanitize_account_id_for_filename(account_id);
    let account_path = auth_states_dir.join(format!("{}.json", safe_name));
    if !account_path.exists() {
        return Ok(None);
    }
    let raw =
        std::fs::read_to_string(&account_path).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to read persisted auth state {}: {error}",
                account_path.display()
            ),
        })?;
    let parsed = serde_json::from_str::<PersistedAuthState>(&raw).map_err(|error| {
        AppError::InternalUnexpected {
            message: format!(
                "failed to parse persisted auth state {}: {error}",
                account_path.display()
            ),
        }
    })?;
    Ok(Some(parsed))
}

fn persist_persisted_auth_state_to_disk(
    auth_states_dir: &Path,
    account_id: &str,
    value: Option<&PersistedAuthState>,
) -> AppResult<()> {
    let safe_name = sanitize_account_id_for_filename(account_id);
    let account_path = auth_states_dir.join(format!("{}.json", safe_name));
    match value {
        None => {
            if account_path.exists() {
                std::fs::remove_file(&account_path).map_err(|error| {
                    AppError::InternalUnexpected {
                        message: format!(
                            "failed to delete persisted auth state {}: {error}",
                            account_path.display()
                        ),
                    }
                })?;
            }
            Ok(())
        }
        Some(value) => {
            let serialized =
                serde_json::to_vec_pretty(value).map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to serialize persisted auth state {}: {error}",
                        account_path.display()
                    ),
                })?;
            let temp_path = build_temp_auth_state_path(&account_path);
            std::fs::write(&temp_path, serialized).map_err(|error| {
                AppError::InternalUnexpected {
                    message: format!(
                        "failed to write temp auth state {}: {error}",
                        temp_path.display()
                    ),
                }
            })?;
            std::fs::rename(&temp_path, &account_path).map_err(|error| {
                AppError::InternalUnexpected {
                    message: format!(
                        "failed to persist auth state {}: {error}",
                        account_path.display()
                    ),
                }
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

fn update_active_account_index(auth_states_dir: &Path, account_id: &str) -> AppResult<()> {
    let active_path = auth_states_dir.join("active.json");
    let active_data = serde_json::json!({
        "accountId": account_id
    });
    let serialized =
        serde_json::to_vec_pretty(&active_data).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to serialize active account index {}: {error}",
                active_path.display()
            ),
        })?;
    let temp_path = build_temp_auth_state_path(&active_path);
    std::fs::write(&temp_path, serialized).map_err(|error| AppError::InternalUnexpected {
        message: format!(
            "failed to write temp active account index {}: {error}",
            temp_path.display()
        ),
    })?;
    std::fs::rename(&temp_path, &active_path).map_err(|error| AppError::InternalUnexpected {
        message: format!(
            "failed to persist active account index {}: {error}",
            active_path.display()
        ),
    })?;
    Ok(())
}

fn remove_active_account_index(auth_states_dir: &Path) -> AppResult<()> {
    let active_path = auth_states_dir.join("active.json");
    if active_path.exists() {
        std::fs::remove_file(&active_path).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to remove active account index {}: {error}",
                active_path.display()
            ),
        })?;
    }
    Ok(())
}

fn list_remaining_account_ids(auth_states_dir: &Path) -> AppResult<Vec<String>> {
    let mut account_ids = Vec::new();
    let entries =
        std::fs::read_dir(auth_states_dir).map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to read auth states dir {}: {error}",
                auth_states_dir.display()
            ),
        })?;

    for entry in entries {
        let entry = entry.map_err(|error| AppError::InternalUnexpected {
            message: format!(
                "failed to read dir entry in {}: {error}",
                auth_states_dir.display()
            ),
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name != "active.json" && file_name.ends_with(".json") {
                    if let Some(sanitized_id) = file_name.strip_suffix(".json") {
                        account_ids.push(unsanitize_account_id_from_filename(sanitized_id));
                    }
                }
            }
        }
    }

    Ok(account_ids)
}

fn refresh_active_account_after_logout(
    auth_states_dir: &Path,
    logged_out_account_id: &str,
) -> AppResult<()> {
    let remaining_accounts = list_remaining_account_ids(auth_states_dir)?;

    if remaining_accounts.is_empty() {
        remove_active_account_index(auth_states_dir)?;
    } else {
        // Pick the first remaining account as the new active account
        let new_active = &remaining_accounts[0];
        if new_active != logged_out_account_id {
            update_active_account_index(auth_states_dir, new_active)?;
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_persisted_auth_state(account_id: &str) -> PersistedAuthState {
        PersistedAuthState {
            version: 1,
            account_id: account_id.to_string(),
            base_url: "https://vault.example".to_string(),
            email: "user@example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
            encrypted_session: crate::bootstrap::auth_persistence::PersistedEncryptedSession {
                algorithm: "xchacha20poly1305".to_string(),
                kdf: "argon2id".to_string(),
                kdf_memory_kib: 65536,
                kdf_iterations: 3,
                kdf_parallelism: 1,
                salt_b64: "test_salt".to_string(),
                nonce_b64: "test_nonce".to_string(),
                ciphertext_b64: "test_ciphertext".to_string(),
            },
            updated_at_ms: 1234567890,
        }
    }

    #[test]
    fn persist_and_load_per_account_auth_state() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let account_id_1 = "account-1";
        let account_id_2 = "account-2";
        let state_1 = create_test_persisted_auth_state(account_id_1);
        let state_2 = create_test_persisted_auth_state(account_id_2);

        // Persist two accounts
        persist_persisted_auth_state_to_disk(auth_states_dir, account_id_1, Some(&state_1))
            .expect("persist account 1");
        persist_persisted_auth_state_to_disk(auth_states_dir, account_id_2, Some(&state_2))
            .expect("persist account 2");

        // Verify both files exist
        assert!(auth_states_dir.join("account-1.json").exists());
        assert!(auth_states_dir.join("account-2.json").exists());

        // Load account 1
        let loaded_1 = load_persisted_auth_state_from_disk(auth_states_dir, account_id_1)
            .expect("load account 1");
        assert!(loaded_1.is_some());
        assert_eq!(loaded_1.unwrap().account_id, account_id_1);

        // Load account 2
        let loaded_2 = load_persisted_auth_state_from_disk(auth_states_dir, account_id_2)
            .expect("load account 2");
        assert!(loaded_2.is_some());
        assert_eq!(loaded_2.unwrap().account_id, account_id_2);
    }

    #[test]
    fn delete_per_account_auth_state() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let account_id = "account-1";
        let state = create_test_persisted_auth_state(account_id);

        // Persist and verify
        persist_persisted_auth_state_to_disk(auth_states_dir, account_id, Some(&state))
            .expect("persist");
        assert!(auth_states_dir.join("account-1.json").exists());

        // Delete and verify
        persist_persisted_auth_state_to_disk(auth_states_dir, account_id, None).expect("delete");
        assert!(!auth_states_dir.join("account-1.json").exists());
    }

    #[test]
    fn update_active_account_index_creates_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let account_id = "account-1";
        update_active_account_index(auth_states_dir, account_id).expect("update active");

        let active_path = auth_states_dir.join("active.json");
        assert!(active_path.exists());

        let content = fs::read_to_string(&active_path).expect("read active.json");
        let data: serde_json::Value = serde_json::from_str(&content).expect("parse json");
        assert_eq!(data["accountId"].as_str(), Some(account_id));
    }

    #[test]
    fn remove_active_account_index_deletes_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let account_id = "account-1";
        update_active_account_index(auth_states_dir, account_id).expect("update active");
        assert!(auth_states_dir.join("active.json").exists());

        remove_active_account_index(auth_states_dir).expect("remove active");
        assert!(!auth_states_dir.join("active.json").exists());
    }

    #[test]
    fn list_remaining_account_ids_returns_all_accounts() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let state_1 = create_test_persisted_auth_state("account-1");
        let state_2 = create_test_persisted_auth_state("account-2");
        let state_3 = create_test_persisted_auth_state("account-3");

        persist_persisted_auth_state_to_disk(auth_states_dir, "account-1", Some(&state_1))
            .expect("persist 1");
        persist_persisted_auth_state_to_disk(auth_states_dir, "account-2", Some(&state_2))
            .expect("persist 2");
        persist_persisted_auth_state_to_disk(auth_states_dir, "account-3", Some(&state_3))
            .expect("persist 3");
        update_active_account_index(auth_states_dir, "account-1").expect("update active");

        let accounts = list_remaining_account_ids(auth_states_dir).expect("list accounts");
        assert_eq!(accounts.len(), 3);
        assert!(accounts.contains(&"account-1".to_string()));
        assert!(accounts.contains(&"account-2".to_string()));
        assert!(accounts.contains(&"account-3".to_string()));
    }

    #[test]
    fn refresh_active_account_after_logout_removes_active_when_no_accounts_remain() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        update_active_account_index(auth_states_dir, "account-1").expect("update active");
        assert!(auth_states_dir.join("active.json").exists());

        refresh_active_account_after_logout(auth_states_dir, "account-1")
            .expect("refresh after logout");
        assert!(!auth_states_dir.join("active.json").exists());
    }

    #[test]
    fn refresh_active_account_after_logout_switches_to_remaining_account() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let state_1 = create_test_persisted_auth_state("account-1");
        let state_2 = create_test_persisted_auth_state("account-2");

        persist_persisted_auth_state_to_disk(auth_states_dir, "account-1", Some(&state_1))
            .expect("persist 1");
        persist_persisted_auth_state_to_disk(auth_states_dir, "account-2", Some(&state_2))
            .expect("persist 2");
        update_active_account_index(auth_states_dir, "account-1").expect("update active");

        // Delete account-1 and refresh
        persist_persisted_auth_state_to_disk(auth_states_dir, "account-1", None)
            .expect("delete account-1");
        refresh_active_account_after_logout(auth_states_dir, "account-1")
            .expect("refresh after logout");

        // Active should now point to account-2
        let active_path = auth_states_dir.join("active.json");
        assert!(active_path.exists());
        let content = fs::read_to_string(&active_path).expect("read active.json");
        let data: serde_json::Value = serde_json::from_str(&content).expect("parse json");
        assert_eq!(data["accountId"].as_str(), Some("account-2"));
    }

    #[test]
    fn load_active_persisted_auth_state_returns_none_when_no_active_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let result = load_active_persisted_auth_state(auth_states_dir).expect("load active");
        assert!(result.is_none());
    }

    #[test]
    fn load_active_persisted_auth_state_loads_correct_account() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let auth_states_dir = temp_dir.path();

        let state_1 = create_test_persisted_auth_state("account-1");
        let state_2 = create_test_persisted_auth_state("account-2");

        persist_persisted_auth_state_to_disk(auth_states_dir, "account-1", Some(&state_1))
            .expect("persist 1");
        persist_persisted_auth_state_to_disk(auth_states_dir, "account-2", Some(&state_2))
            .expect("persist 2");
        update_active_account_index(auth_states_dir, "account-2").expect("update active");

        let loaded = load_active_persisted_auth_state(auth_states_dir)
            .expect("load active")
            .expect("should have state");
        assert_eq!(loaded.account_id, "account-2");
    }
}
