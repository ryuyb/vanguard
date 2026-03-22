use std::sync::Arc;

use tauri::{Emitter, Manager, Runtime};

use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::biometric_unlock_port::BiometricUnlockPort;
use crate::application::ports::clipboard_port::ClipboardPort;
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::notification_port::NotificationPort;
use crate::application::ports::pin_unlock_port::PinUnlockPort;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::text_injection_port::TextInjectionPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::create_cipher_use_case::CreateCipherUseCase;
use crate::application::use_cases::delete_cipher_use_case::DeleteCipherUseCase;
use crate::application::use_cases::fetch_cipher_use_case::FetchCipherUseCase;
use crate::application::use_cases::get_cipher_detail_use_case::GetCipherDetailUseCase;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::application::use_cases::restore_cipher_use_case::RestoreCipherUseCase;
use crate::application::use_cases::soft_delete_cipher_use_case::SoftDeleteCipherUseCase;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::application::use_cases::update_cipher_use_case::UpdateCipherUseCase;
use crate::bootstrap::app_state::AppState;
use crate::bootstrap::config::AppConfig;
use crate::bootstrap::unlock_state::{UnlockState, UnlockStatus};
use crate::infrastructure::desktop::EnigoTextInjectionAdapter;
use crate::infrastructure::persistence::{
    SqliteMasterPasswordUnlockDataPort, SqliteVaultRepository,
};
use crate::infrastructure::security::biometric_unlock_port_adapter::KeychainBiometricUnlockPort;
use crate::infrastructure::security::clipboard_port_adapter::TauriClipboardPortAdapter;
use crate::infrastructure::security::pin_unlock_port_adapter::KeychainPinUnlockPort;
use crate::infrastructure::vaultwarden::{
    VaultwardenClient, VaultwardenConfig, VaultwardenNotificationPort, VaultwardenRemotePort,
};
use crate::interfaces::tauri::dto::unlock_state::UnlockStatusDto;
use crate::interfaces::tauri::events::sync_event_adapter::TauriSyncEventAdapter;
use crate::interfaces::tauri::events::unlock_state::{UnlockStateChanged, UnlockStateEvent};
use crate::support::error::AppError;
use crate::support::result::AppResult;

pub fn build_app_state<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<AppState> {
    let mut config = AppConfig::load(manager)?;

    // Check text injection permission and disable spotlight_autofill if not available
    config.check_and_fix_text_injection_permission(manager)?;

    let mut vaultwarden_config = VaultwardenConfig::new();
    vaultwarden_config.device_identifier = config.device_identifier.clone();
    vaultwarden_config.allow_invalid_certs = config.allow_invalid_certs;

    let client = VaultwardenClient::new(vaultwarden_config).map_err(|error| {
        AppError::InternalUnexpected {
            message: format!("failed to create vaultwarden client: {error}"),
        }
    })?;

    let remote_vault: Arc<dyn RemoteVaultPort> =
        Arc::new(VaultwardenRemotePort::new(client.clone()));
    let notification_port: Arc<dyn NotificationPort> = Arc::new(VaultwardenNotificationPort::new());
    let sqlite_dir = resolve_sqlite_dir(manager)?;
    let auth_state_path = resolve_auth_state_path(manager)?;
    let vault_repository: Arc<dyn VaultRepositoryPort> =
        Arc::new(SqliteVaultRepository::new(sqlite_dir)?);
    let master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort> = Arc::new(
        SqliteMasterPasswordUnlockDataPort::new(Arc::clone(&vault_repository)),
    );
    let pin_unlock_port: Arc<dyn PinUnlockPort> = Arc::new(KeychainPinUnlockPort::new());
    let biometric_unlock_port: Arc<dyn BiometricUnlockPort> = Arc::new(KeychainBiometricUnlockPort);
    let clipboard_port: Arc<dyn ClipboardPort> =
        Arc::new(TauriClipboardPortAdapter::new(manager.app_handle().clone()));
    let text_injection_port: Arc<dyn TextInjectionPort> =
        Arc::new(EnigoTextInjectionAdapter::new()?);
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
    let fetch_cipher_use_case = Arc::new(FetchCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let realtime_sync_service = Arc::new(RealtimeSyncService::new(
        notification_port,
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
        Arc::clone(&sync_service),
        Arc::clone(&fetch_cipher_use_case),
        sync_policy,
        config.device_identifier.clone(),
    ));
    let get_cipher_detail_use_case =
        Arc::new(GetCipherDetailUseCase::new(Arc::clone(&sync_service)));

    let create_cipher_use_case = Arc::new(CreateCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let update_cipher_use_case = Arc::new(UpdateCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let delete_cipher_use_case = Arc::new(DeleteCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let soft_delete_cipher_use_case = Arc::new(SoftDeleteCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let restore_cipher_use_case = Arc::new(RestoreCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let fetch_cipher_use_case = Arc::new(FetchCipherUseCase::new(
        Arc::clone(&remote_vault),
        Arc::clone(&vault_repository),
        Arc::clone(&sync_event_port),
    ));

    let app_state = AppState::new(
        auth_service,
        sync_service,
        realtime_sync_service,
        client,
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
        auth_state_path,
        config,
    );

    // Setup state change event emitter
    setup_state_event_emitter(manager.app_handle().clone(), &app_state);

    Ok(app_state)
}

fn resolve_sqlite_dir<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<std::path::PathBuf> {
    let app_data_dir =
        manager
            .path()
            .app_data_dir()
            .map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to resolve app data dir: {error}"),
            })?;
    let sqlite_dir = app_data_dir.join("vault-repositories");
    std::fs::create_dir_all(&sqlite_dir).map_err(|error| AppError::InternalUnexpected {
        message: format!(
            "failed to create app data dir {}: {error}",
            sqlite_dir.display()
        ),
    })?;
    Ok(sqlite_dir)
}

fn resolve_auth_state_path<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> AppResult<std::path::PathBuf> {
    let app_data_dir =
        manager
            .path()
            .app_data_dir()
            .map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to resolve app data dir: {error}"),
            })?;
    let auth_states_dir = app_data_dir.join("auth-states");
    std::fs::create_dir_all(&auth_states_dir).map_err(|error| AppError::InternalUnexpected {
        message: format!(
            "failed to create auth states dir {}: {error}",
            auth_states_dir.display()
        ),
    })?;
    Ok(auth_states_dir)
}

/// Setup state change event emitter for Tauri events
fn setup_state_event_emitter<R: Runtime>(app_handle: tauri::AppHandle<R>, app_state: &AppState) {
    let manager = app_state.unlock_manager();

    manager.on_state_change(move |old_state: &UnlockState, new_state: &UnlockState| {
        let old_status = unlock_status_to_dto(old_state.status);
        let new_status = unlock_status_to_dto(new_state.status);

        let event_dto = UnlockStateEvent {
            old_status,
            new_status,
            has_key_material: new_state.key_material.is_some(),
            account_id: new_state
                .account_context
                .as_ref()
                .map(|ctx| ctx.account_id.clone()),
        };

        if let Err(e) = app_handle.emit(
            "unlock-state:changed",
            UnlockStateChanged { event: event_dto },
        ) {
            log::warn!(
                target: "vanguard::wiring",
                "Failed to emit unlock state change event: {}",
                e
            );
        }
    });
}

fn unlock_status_to_dto(status: UnlockStatus) -> UnlockStatusDto {
    match status {
        UnlockStatus::Locked => UnlockStatusDto::Locked,
        UnlockStatus::VaultUnlockedSessionExpired => UnlockStatusDto::VaultUnlockedSessionExpired,
        UnlockStatus::FullyUnlocked => UnlockStatusDto::FullyUnlocked,
        UnlockStatus::Unlocking => UnlockStatusDto::Unlocking,
    }
}
