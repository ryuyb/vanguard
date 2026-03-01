#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Idle,
    Running,
    Succeeded,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTrigger {
    Startup,
    Manual,
    Poll,
    WebSocket,
    Foreground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsStatus {
    Unknown,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Default)]
pub struct SyncItemCounts {
    pub folders: u32,
    pub collections: u32,
    pub policies: u32,
    pub ciphers: u32,
    pub sends: u32,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub duration_ms: i64,
    pub item_counts: SyncItemCounts,
    pub revision_changed: bool,
}

#[derive(Debug, Clone)]
pub struct SyncContext {
    pub account_id: String,
    pub base_url: Option<String>,
    pub state: SyncState,
    pub ws_status: WsStatus,
    pub last_revision_ms: Option<i64>,
    pub last_sync_at_ms: Option<i64>,
    pub last_error: Option<String>,
    pub counts: SyncItemCounts,
}

impl SyncContext {
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            base_url: None,
            state: SyncState::Idle,
            ws_status: WsStatus::Unknown,
            last_revision_ms: None,
            last_sync_at_ms: None,
            last_error: None,
            counts: SyncItemCounts::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VaultSnapshotMeta {
    pub snapshot_revision_ms: Option<i64>,
    pub snapshot_synced_at_ms: i64,
    pub source: SyncTrigger,
}
