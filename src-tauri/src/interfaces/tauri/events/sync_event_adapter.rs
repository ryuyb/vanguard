use tauri::{AppHandle, Runtime};
use tauri_specta::Event;

use crate::application::ports::sync_event_port::SyncEventPort;
use crate::domain::sync::SyncContext;
use crate::interfaces::tauri::events::cipher::{CipherCreated, CipherDeleted, CipherUpdated};
use crate::interfaces::tauri::events::send::{SendCreated, SendDeleted, SendUpdated};
use crate::interfaces::tauri::events::sync::{
    VaultFoldersSynced, VaultSyncAuthRequired, VaultSyncFailed, VaultSyncLoggedOut,
    VaultSyncStarted, VaultSyncSucceeded,
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
        let status = mapping::to_sync_status_response_dto(context.clone(), None);
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

    fn emit_folders_synced(&self, account_id: &str, folder_count: u32) {
        if let Err(error) = (VaultFoldersSynced {
            account_id: String::from(account_id),
            folder_count,
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-folders-synced for account_id={account_id}: {error}"
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

    fn emit_logged_out(&self, account_id: &str, reason: &str) {
        if let Err(error) = (VaultSyncLoggedOut {
            account_id: String::from(account_id),
            reason: String::from(reason),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::sync",
                "failed to emit vault-sync-logged-out for account_id={account_id}: {error}"
            );
        }
    }

    fn emit_cipher_created(&self, account_id: &str, cipher_id: &str) {
        if let Err(error) = (CipherCreated {
            account_id: String::from(account_id),
            cipher_id: String::from(cipher_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::cipher",
                "failed to emit cipher:created for account_id={account_id} cipher_id={cipher_id}: {error}"
            );
        }
    }

    fn emit_cipher_updated(&self, account_id: &str, cipher_id: &str) {
        if let Err(error) = (CipherUpdated {
            account_id: String::from(account_id),
            cipher_id: String::from(cipher_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::cipher",
                "failed to emit cipher:updated for account_id={account_id} cipher_id={cipher_id}: {error}"
            );
        }
    }

    fn emit_cipher_deleted(&self, account_id: &str, cipher_id: &str) {
        if let Err(error) = (CipherDeleted {
            account_id: String::from(account_id),
            cipher_id: String::from(cipher_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::cipher",
                "failed to emit cipher:deleted for account_id={account_id} cipher_id={cipher_id}: {error}"
            );
        }
    }

    fn emit_send_created(&self, account_id: &str, send_id: &str) {
        if let Err(error) = (SendCreated {
            account_id: String::from(account_id),
            send_id: String::from(send_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::send",
                "failed to emit send:created for account_id={account_id} send_id={send_id}: {error}"
            );
        }
    }

    fn emit_send_updated(&self, account_id: &str, send_id: &str) {
        if let Err(error) = (SendUpdated {
            account_id: String::from(account_id),
            send_id: String::from(send_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::send",
                "failed to emit send:updated for account_id={account_id} send_id={send_id}: {error}"
            );
        }
    }

    fn emit_send_deleted(&self, account_id: &str, send_id: &str) {
        if let Err(error) = (SendDeleted {
            account_id: String::from(account_id),
            send_id: String::from(send_id),
        })
        .emit(&self.app)
        {
            log::warn!(
                target: "vanguard::tauri::send",
                "failed to emit send:deleted for account_id={account_id} send_id={send_id}: {error}"
            );
        }
    }
}
