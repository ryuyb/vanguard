use std::sync::Arc;

use tauri::{Manager, Runtime};

use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::services::auth_service::AuthService;
use crate::bootstrap::app_state::AppState;
use crate::bootstrap::config::AppConfig;
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
    let auth_service = Arc::new(AuthService::new(remote_vault));

    Ok(AppState::new(auth_service))
}
