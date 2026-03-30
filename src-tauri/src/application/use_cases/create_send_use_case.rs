use std::sync::Arc;

use crate::application::dto::sync::{CreateSendCommand, SendMutationResult, SyncSend};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::send_encryption;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct CreateSendUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl CreateSendUseCase {
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
        send: SyncSend,
        user_key: VaultUserKeyMaterial,
        file_data: Option<Vec<u8>>,
    ) -> AppResult<SendMutationResult> {
        if account_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "account_id".to_string(),
            });
        }
        if send
            .name
            .as_ref()
            .map(|n| n.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(AppError::ValidationFieldError {
                field: "send.name".to_string(),
                message: "send name is required".to_string(),
            });
        }

        let encrypted = send_encryption::encrypt_send(&send, &user_key)?;

        let is_file = send.r#type == Some(1);

        let result = if is_file {
            let file_bytes = file_data.ok_or_else(|| AppError::ValidationFieldError {
                field: "file_data".to_string(),
                message: "file data is required for file send".to_string(),
            })?;

            let create_result = self
                .remote_vault
                .create_file_send(CreateSendCommand {
                    account_id: account_id.clone(),
                    base_url: base_url.clone(),
                    access_token: access_token.clone(),
                    send: encrypted.clone(),
                })
                .await?;

            self.remote_vault
                .upload_send_file(
                    &base_url,
                    &access_token,
                    &create_result.send_id,
                    &create_result.file_id,
                    file_bytes,
                )
                .await?;

            SendMutationResult {
                send_id: create_result.send_id,
                revision_date: create_result.revision_date,
            }
        } else {
            self.remote_vault
                .create_send(CreateSendCommand {
                    account_id: account_id.clone(),
                    base_url,
                    access_token,
                    send: encrypted.clone(),
                })
                .await?
        };

        let persisted = SyncSend {
            id: result.send_id.clone(),
            revision_date: Some(result.revision_date.clone()),
            ..encrypted
        };
        self.vault_repository
            .upsert_send(&account_id, &persisted)
            .await?;
        self.event_port
            .emit_send_created(&account_id, &result.send_id);

        Ok(result)
    }
}
