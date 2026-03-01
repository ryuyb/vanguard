use async_trait::async_trait;

use crate::application::dto::notification::{NotificationConnectCommand, NotificationEvent};
use crate::support::result::AppResult;

#[async_trait]
pub trait NotificationPort: Send + Sync {
    async fn connect(&self, command: NotificationConnectCommand) -> AppResult<()>;

    async fn next_event(&self, account_id: &str) -> AppResult<Option<NotificationEvent>>;

    async fn disconnect(&self, account_id: &str) -> AppResult<()>;
}
