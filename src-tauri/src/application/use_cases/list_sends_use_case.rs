use std::sync::Arc;

use crate::application::dto::sync::SyncSend;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::send_encryption;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct ListSendsUseCase {
    vault_repository: Arc<dyn VaultRepositoryPort>,
}

impl ListSendsUseCase {
    pub fn new(vault_repository: Arc<dyn VaultRepositoryPort>) -> Self {
        Self { vault_repository }
    }

    pub async fn execute(
        &self,
        account_id: String,
        user_key: VaultUserKeyMaterial,
    ) -> AppResult<Vec<SyncSend>> {
        if account_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "account_id".to_string(),
            });
        }

        let sends = self.vault_repository.list_live_sends(&account_id).await?;

        sends
            .into_iter()
            .map(|send| send_encryption::decrypt_send(&send, &user_key))
            .collect()
    }
}
