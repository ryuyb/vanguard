use std::sync::Arc;

use tauri::{Manager, Runtime};

use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::services::auth_service::AuthService;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::bootstrap::app_state::AppState;
use crate::bootstrap::config::AppConfig;
use crate::infrastructure::persistence::InMemoryVaultRepository;
use crate::interfaces::tauri::events::sync_event_adapter::TauriSyncEventAdapter;
use crate::infrastructure::vaultwarden::{
    VaultwardenClient, VaultwardenConfig, VaultwardenRemotePort,
};
use crate::support::error::AppError;
use crate::support::result::AppResult;

pub fn build_app_state<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<AppState> {
    let config = AppConfig::load(manager)?;

    let mut vaultwarden_config = VaultwardenConfig::new();
    vaultwarden_config.device_identifier = config.device_identifier;
    vaultwarden_config.allow_invalid_certs = config.allow_invalid_certs;

    let client = VaultwardenClient::new(vaultwarden_config).map_err(|error| {
        AppError::internal(format!("failed to create vaultwarden client: {error}"))
    })?;

    let remote_vault: Arc<dyn RemoteVaultPort> = Arc::new(VaultwardenRemotePort::new(client));
    let vault_repository: Arc<dyn VaultRepositoryPort> = Arc::new(InMemoryVaultRepository::new());
    let sync_event_port: Arc<dyn SyncEventPort> =
        Arc::new(TauriSyncEventAdapter::new(manager.app_handle().clone()));
    let auth_service = Arc::new(AuthService::new(Arc::clone(&remote_vault)));
    let sync_policy = SyncPolicy::default();
    let poll_revision_use_case = Arc::new(PollRevisionUseCase::new(Arc::clone(&remote_vault)));
    let sync_vault_use_case = Arc::new(SyncVaultUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        poll_revision_use_case,
        sync_policy.clone(),
    ));
    let sync_service = Arc::new(SyncService::new(
        sync_vault_use_case,
        vault_repository,
        sync_event_port,
        sync_policy,
    ));

    Ok(AppState::new(auth_service, sync_service))
}
