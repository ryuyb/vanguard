use serde::Serialize;
use specta::Type;

use crate::interfaces::tauri::dto::sync::SyncStatusResponseDto;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultSyncStarted {
    pub account_id: String,
}

impl tauri_specta::Event for VaultSyncStarted {
    const NAME: &'static str = "vault-sync:started";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultSyncSucceeded {
    pub account_id: String,
    pub status: SyncStatusResponseDto,
}

impl tauri_specta::Event for VaultSyncSucceeded {
    const NAME: &'static str = "vault-sync:succeeded";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultSyncFailed {
    pub account_id: String,
    pub code: String,
    pub message: String,
}

impl tauri_specta::Event for VaultSyncFailed {
    const NAME: &'static str = "vault-sync:failed";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultSyncAuthRequired {
    pub account_id: String,
    pub status: u16,
    pub message: String,
}

impl tauri_specta::Event for VaultSyncAuthRequired {
    const NAME: &'static str = "vault-sync:auth-required";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultSyncLoggedOut {
    pub account_id: String,
    pub reason: String,
}

impl tauri_specta::Event for VaultSyncLoggedOut {
    const NAME: &'static str = "vault-sync:logged-out";
}
