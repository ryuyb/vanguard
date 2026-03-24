use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
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
use crate::bootstrap::auth_persistence_port::{AuthPersistence, AuthPersistenceService};
use crate::bootstrap::config::AppConfig;
use crate::bootstrap::unlock_state::{UnifiedUnlockManager, VaultKeyMaterial};
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

// Conversion helpers between VaultUserKey and VaultKeyMaterial
// Still actively used by cipher.rs and folder.rs

impl From<&VaultUserKey> for VaultKeyMaterial {
    fn from(key: &VaultUserKey) -> Self {
        Self {
            enc_key: key.enc_key.clone(),
            mac_key: key.mac_key.clone(),
        }
    }
}

impl From<&VaultKeyMaterial> for VaultUserKey {
    fn from(key: &VaultKeyMaterial) -> Self {
        Self {
            enc_key: key.enc_key.clone(),
            mac_key: key.mac_key.clone(),
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
        // Initialize auth persistence service
        let persistence = Arc::new(AuthPersistenceService::new(auth_states_dir));

        // Load persisted auth state to restore account context on app restart
        // Create a temporary runtime since we're in a sync context during app setup
        let initial_account = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::InternalUnexpected {
                message: format!("failed to create temporary runtime: {e}"),
            })
            .ok()
            .and_then(|rt| {
                rt.block_on(async {
                    persistence
                        .load_auth_state()
                        .await
                        .ok()
                        .flatten()
                        .map(|(account_ctx, _)| account_ctx)
                })
            });

        // Initialize unified unlock manager with persisted account context
        let unlock_manager = UnifiedUnlockManager::new(config, initial_account, persistence);

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

    pub async fn active_account_id(&self) -> AppResult<String> {
        self.unlock_manager.active_account_id().await
    }

    pub async fn persisted_auth_context(&self) -> AppResult<Option<PersistedAuthContext>> {
        let account_ctx = self.unlock_manager.account_context().await;
        Ok(account_ctx.map(|ctx| PersistedAuthContext {
            account_id: ctx.account_id,
            base_url: ctx.base_url,
            email: ctx.email,
            kdf: ctx.kdf,
            kdf_iterations: ctx.kdf_iterations,
            kdf_memory: ctx.kdf_memory,
            kdf_parallelism: ctx.kdf_parallelism,
        }))
    }
}

#[async_trait]
impl VaultRuntimePort for AppState {
    async fn active_account_id(&self) -> AppResult<String> {
        self.unlock_manager.active_account_id().await
    }

    async fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>> {
        let account_ctx = self.unlock_manager.account_context().await;
        Ok(account_ctx.map(|ctx| VaultUnlockContext {
            account_id: ctx.account_id,
            base_url: ctx.base_url,
            email: ctx.email,
            kdf: ctx.kdf,
            kdf_iterations: ctx.kdf_iterations,
            kdf_memory: ctx.kdf_memory,
            kdf_parallelism: ctx.kdf_parallelism,
        }))
    }

    async fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>> {
        AppState::persisted_auth_context(self).await.map(|value| {
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

    async fn get_vault_user_key_material(
        &self,
        _account_id: &str,
    ) -> AppResult<Option<VaultUserKeyMaterial>> {
        let key_material = self.unlock_manager.key_material_dto().await;
        Ok(key_material)
    }

    async fn set_vault_user_key_material(
        &self,
        _account_id: String,
        key: VaultUserKeyMaterial,
    ) -> AppResult<()> {
        let key_material: VaultKeyMaterial = key.into();
        self.unlock_manager.set_key_material(key_material).await
    }

    async fn remove_vault_user_key_material(&self, _account_id: &str) -> AppResult<()> {
        self.unlock_manager.remove_key_material().await
    }

    async fn get_refresh_token(&self) -> AppResult<Option<String>> {
        let token = self.unlock_manager.refresh_token().await;
        Ok(token)
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
