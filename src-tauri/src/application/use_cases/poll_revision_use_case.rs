use std::sync::Arc;

use crate::application::dto::sync::RevisionDateQuery;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct PollRevisionUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
}

impl PollRevisionUseCase {
    pub fn new(remote_vault: Arc<dyn RemoteVaultPort>) -> Self {
        Self { remote_vault }
    }

    pub async fn execute(&self, query: RevisionDateQuery) -> AppResult<i64> {
        require_non_empty(&query.base_url, "base_url")?;
        require_non_empty(&query.access_token, "access_token")?;
        self.remote_vault.get_revision_date(query).await
    }
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("{field} cannot be empty"),
        });
    }
    Ok(())
}
