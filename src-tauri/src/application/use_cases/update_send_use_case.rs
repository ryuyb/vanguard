use std::sync::Arc;

use crate::application::dto::sync::{SendMutationResult, SyncSend, UpdateSendCommand};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::send_encryption;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct UpdateSendUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl UpdateSendUseCase {
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
        send: SyncSend,
        user_key: VaultUserKeyMaterial,
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
        if send.name.as_ref().map(|n| n.trim().is_empty()).unwrap_or(true) {
            return Err(AppError::ValidationFieldError {
                field: "send.name".to_string(),
                message: "send name is required".to_string(),
            });
        }

        let encrypted = send_encryption::encrypt_send(&send, &user_key)?;

        let result = self
            .remote_vault
            .update_send(UpdateSendCommand {
                account_id: account_id.clone(),
                base_url,
                access_token,
                send_id: send_id.clone(),
                send: encrypted.clone(),
            })
            .await?;

        let persisted = SyncSend {
            id: result.send_id.clone(),
            revision_date: Some(result.revision_date.clone()),
            ..encrypted
        };
        self.vault_repository.upsert_send(&account_id, &persisted).await?;
        self.event_port.emit_send_updated(&account_id, &result.send_id);

        Ok(result)
    }
}
