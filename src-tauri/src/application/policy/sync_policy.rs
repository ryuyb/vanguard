#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub max_retries: u8,
    pub backoff_ms: u64,
    pub debounce_ms: u64,
    pub timeout_ms: u64,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            backoff_ms: 500,
            debounce_ms: 300,
            timeout_ms: 15_000,
        }
    }
}
