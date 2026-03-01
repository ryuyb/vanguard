#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub max_retries: u8,
    pub backoff_ms: u64,
    pub debounce_ms: u64,
    pub timeout_ms: u64,
    pub poll_interval_seconds: u64,
    pub ws_reconnect_base_ms: u64,
    pub ws_reconnect_max_ms: u64,
    pub ws_online_poll_interval_seconds: u64,
    pub ws_offline_poll_interval_seconds: u64,
    pub ws_message_debounce_ms: u64,
    pub ws_event_queue_limit: usize,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            backoff_ms: 500,
            debounce_ms: 300,
            timeout_ms: 15_000,
            poll_interval_seconds: 60,
            ws_reconnect_base_ms: 1_000,
            ws_reconnect_max_ms: 30_000,
            ws_online_poll_interval_seconds: 120,
            ws_offline_poll_interval_seconds: 60,
            ws_message_debounce_ms: 300,
            ws_event_queue_limit: 256,
        }
    }
}
