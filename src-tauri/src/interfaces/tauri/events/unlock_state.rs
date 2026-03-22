use serde::{Deserialize, Serialize};
use specta::Type;

use crate::interfaces::tauri::dto::unlock_state::UnlockStatusDto;

/// Event emitted when unlock state changes
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UnlockStateEvent {
    pub old_status: UnlockStatusDto,
    pub new_status: UnlockStatusDto,
    pub has_key_material: bool,
    pub account_id: Option<String>,
}

/// Tauri event for unlock state changes
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UnlockStateChanged {
    pub event: UnlockStateEvent,
}

impl tauri_specta::Event for UnlockStateChanged {
    const NAME: &'static str = "unlock-state:changed";
}
