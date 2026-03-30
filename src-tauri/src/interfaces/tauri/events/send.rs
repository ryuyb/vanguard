use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendCreated {
    pub account_id: String,
    pub send_id: String,
}

impl tauri_specta::Event for SendCreated {
    const NAME: &'static str = "send:created";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendUpdated {
    pub account_id: String,
    pub send_id: String,
}

impl tauri_specta::Event for SendUpdated {
    const NAME: &'static str = "send:updated";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendDeleted {
    pub account_id: String,
    pub send_id: String,
}

impl tauri_specta::Event for SendDeleted {
    const NAME: &'static str = "send:deleted";
}
