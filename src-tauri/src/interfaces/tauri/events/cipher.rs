use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CipherCreated {
    pub account_id: String,
    pub cipher_id: String,
}

impl tauri_specta::Event for CipherCreated {
    const NAME: &'static str = "cipher:created";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CipherUpdated {
    pub account_id: String,
    pub cipher_id: String,
}

impl tauri_specta::Event for CipherUpdated {
    const NAME: &'static str = "cipher:updated";
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CipherDeleted {
    pub account_id: String,
    pub cipher_id: String,
}

impl tauri_specta::Event for CipherDeleted {
    const NAME: &'static str = "cipher:deleted";
}
