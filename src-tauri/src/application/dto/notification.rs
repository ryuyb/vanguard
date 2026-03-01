use serde_json::Value;

#[derive(Debug, Clone)]
pub struct NotificationConnectCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub queue_limit: usize,
}

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub event_type: i32,
    pub context_id: Option<String>,
    pub payload: Option<Value>,
    pub received_at_ms: i64,
}
