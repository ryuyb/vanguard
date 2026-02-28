use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncNowRequestDto {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub exclude_domains: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusRequestDto {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncCountsDto {
    pub folders: u32,
    pub collections: u32,
    pub policies: u32,
    pub ciphers: u32,
    pub sends: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SyncStateDto {
    Idle,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WsStatusDto {
    Unknown,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusResponseDto {
    pub account_id: String,
    pub base_url: Option<String>,
    pub state: SyncStateDto,
    pub ws_status: WsStatusDto,
    pub last_revision_ms: Option<String>,
    pub last_sync_at_ms: Option<String>,
    pub last_error: Option<String>,
    pub counts: SyncCountsDto,
}
