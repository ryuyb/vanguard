use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

use crate::application::ports::sync_event_port::SyncEventPort;
use crate::domain::sync::SyncContext;
use crate::interfaces::tauri::events::sync::{
    VaultSyncAuthRequired, VaultSyncFailed, VaultSyncStarted, VaultSyncSucceeded,
};
use crate::interfaces::tauri::mapping;

#[derive(Clone)]
pub struct TauriSyncEventAdapter<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriSyncEventAdapter<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> SyncEventPort for TauriSyncEventAdapter<R> {
    fn emit_sync_started(&self, account_id: &str) {
        if let Err(error) = (VaultSyncStarted {
            account_id: String::from(account_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-sync-started for account_id={account_id}: {error}"
            );
        }
    }

    fn emit_sync_succeeded(&self, context: &SyncContext) {
        let status = mapping::to_sync_status_response_dto(context.clone());
        if let Err(error) = (VaultSyncSucceeded {
            account_id: context.account_id.clone(),
            status,
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-sync-succeeded for account_id={}: {error}",
                context.account_id
            );
        }
    }

    fn emit_sync_failed(&self, account_id: &str, code: &str, message: &str) {
        if let Err(error) = (VaultSyncFailed {
            account_id: String::from(account_id),
            code: String::from(code),
            message: String::from(message),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-sync-failed for account_id={account_id}: {error}"
            );
        }
    }

    fn emit_auth_required(&self, account_id: &str, status: u16, message: &str) {
        if let Err(error) = (VaultSyncAuthRequired {
            account_id: String::from(account_id),
            status,
            message: String::from(message),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-sync-auth-required for account_id={account_id}: {error}"
            );
        }
    }
}
