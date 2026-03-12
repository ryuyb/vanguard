use std::sync::Arc;

use crate::application::dto::sync::DeleteCipherCommand;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct DeleteCipherUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl DeleteCipherUseCase {
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
        cipher_id: String,
    ) -> AppResult<()> {
        if account_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "account_id".to_string(),
            });
        }

        if cipher_id.trim().is_empty() {
            return Err(AppError::ValidationRequired {
                field: "cipher_id".to_string(),
            });
        }

        let command = DeleteCipherCommand {
            account_id: account_id.clone(),
            base_url,
            access_token,
            cipher_id: cipher_id.clone(),
        };

        self.remote_vault.delete_cipher(command).await?;

        self.vault_repository
            .delete_cipher(&account_id, &cipher_id)
            .await?;

        self.event_port.emit_cipher_deleted(&account_id, &cipher_id);

        Ok(())
    }
}
