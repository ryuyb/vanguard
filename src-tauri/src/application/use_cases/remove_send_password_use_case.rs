use std::sync::Arc;

use crate::application::dto::sync::{RemoveSendPasswordCommand, SendMutationResult};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct RemoveSendPasswordUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl RemoveSendPasswordUseCase {
    pub fn new(
        remote_vault: Arc<dyn RemoteVaultPort>,
        vault_repository: Arc<dyn VaultRepositoryPort>,
        event_port: Arc<dyn SyncEventPort>,
    ) -> Self {
        Self {
            remote_vault,
            vault_repository,
            event_port,
        }
    }

    pub async fn execute(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
        send_id: String,
    ) -> AppResult<SendMutationResult> {
        if account_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "account_id".to_string(),
            });
        }
        if send_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "send_id".to_string(),
            });
        }

        let updated = self
            .remote_vault
            .remove_send_password(RemoveSendPasswordCommand {
                account_id: account_id.clone(),
                base_url,
                access_token,
                send_id: send_id.clone(),
            })
            .await?;

        let result = SendMutationResult {
            send_id: updated.id.clone(),
            revision_date: updated.revision_date.clone().unwrap_or_default(),
        };

        self.vault_repository
            .upsert_send(&account_id, &updated)
            .await?;
        self.event_port
            .emit_send_updated(&account_id, &result.send_id);

        Ok(result)
    }
}
