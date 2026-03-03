use std::sync::Arc;

use tauri::{Manager, Runtime};

use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::notification_port::NotificationPort;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::bootstrap::app_state::AppState;
use crate::bootstrap::config::AppConfig;
use crate::infrastructure::persistence::{
    SqliteMasterPasswordUnlockDataPort, SqliteVaultRepository,
};
use crate::infrastructure::security::biometric_unlock_port_adapter::KeychainBiometricUnlockPort;
use crate::infrastructure::vaultwarden::{
    VaultwardenClient, VaultwardenConfig, VaultwardenNotificationPort, VaultwardenRemotePort,
};
use crate::interfaces::tauri::events::sync_event_adapter::TauriSyncEventAdapter;
use crate::support::error::AppError;
use crate::support::result::AppResult;

pub fn build_app_state<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<AppState> {
    let config = AppConfig::load(manager)?;

    let mut vaultwarden_config = VaultwardenConfig::new();
    vaultwarden_config.device_identifier = config.device_identifier.clone();
    vaultwarden_config.allow_invalid_certs = config.allow_invalid_certs;

    let client = VaultwardenClient::new(vaultwarden_config).map_err(|error| {
        AppError::internal(format!("failed to create vaultwarden client: {error}"))
    })?;

    let remote_vault: Arc<dyn RemoteVaultPort> = Arc::new(VaultwardenRemotePort::new(client));
    let notification_port: Arc<dyn NotificationPort> = Arc::new(VaultwardenNotificationPort::new());
    let sqlite_dir = resolve_sqlite_dir(manager)?;
    let auth_state_path = resolve_auth_state_path(manager)?;
    let vault_repository: Arc<dyn VaultRepositoryPort> =
        Arc::new(SqliteVaultRepository::new(sqlite_dir)?);
    let master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort> =
        Arc::new(SqliteMasterPasswordUnlockDataPort::new(Arc::clone(
            &vault_repository,
        )));
    let biometric_unlock_port: Arc<dyn BiometricUnlockPort> = Arc::new(KeychainBiometricUnlockPort);
    let sync_event_port: Arc<dyn SyncEventPort> =
        Arc::new(TauriSyncEventAdapter::new(manager.app_handle().clone()));
    let auth_service = Arc::new(AuthService::new(Arc::clone(&remote_vault)));
    let sync_policy = SyncPolicy {
        poll_interval_seconds: config.sync_poll_interval_seconds,
        ..SyncPolicy::default()
    };
    let poll_revision_use_case = Arc::new(PollRevisionUseCase::new(Arc::clone(&remote_vault)));
    let sync_vault_use_case = Arc::new(SyncVaultUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&poll_revision_use_case),
        sync_policy.clone(),
    ));
    let sync_service = Arc::new(SyncService::new(
        sync_vault_use_case,
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
        poll_revision_use_case,
        sync_policy.clone(),
    ));
    let realtime_sync_service = Arc::new(RealtimeSyncService::new(
        notification_port,
        vault_repository,
        sync_event_port,
        Arc::clone(&sync_service),
        sync_policy,
        config.device_identifier,
    ));
    let get_cipher_detail_use_case =
        Arc::new(GetCipherDetailUseCase::new(Arc::clone(&sync_service)));

    Ok(AppState::new(
        auth_service,
        sync_service,
        realtime_sync_service,
        master_password_unlock_data_port,
        biometric_unlock_port,
        get_cipher_detail_use_case,
        auth_state_path,
    ))
}

fn resolve_sqlite_dir<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<std::path::PathBuf> {
    let app_data_dir = manager
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(format!("failed to resolve app data dir: {error}")))?;
    let sqlite_dir = app_data_dir.join("vault-repositories");
    std::fs::create_dir_all(&sqlite_dir).map_err(|error| {
        AppError::internal(format!(
            "failed to create app data dir {}: {error}",
            sqlite_dir.display()
        ))
    })?;
    Ok(sqlite_dir)
}

fn resolve_auth_state_path<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> AppResult<std::path::PathBuf> {
    let app_data_dir = manager
        .path()
        .app_data_dir()
        .map_err(|error| AppError::internal(format!("failed to resolve app data dir: {error}")))?;
    std::fs::create_dir_all(&app_data_dir).map_err(|error| {
        AppError::internal(format!(
            "failed to create app data dir {}: {error}",
            app_data_dir.display()
        ))
    })?;
    Ok(app_data_dir.join("auth-state.json"))
}
