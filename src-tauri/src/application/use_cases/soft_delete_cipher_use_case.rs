use std::sync::Arc;

use crate::application::dto::sync::{CipherMutationResult, SoftDeleteCipherCommand, SyncVaultCommand};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::domain::sync::SyncTrigger;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct SoftDeleteCipherUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl SoftDeleteCipherUseCase {
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
    ) -> AppResult<CipherMutationResult> {
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

        let command = SoftDeleteCipherCommand {
            account_id: account_id.clone(),
            base_url: base_url.clone(),
            access_token: access_token.clone(),
            cipher_id: cipher_id.clone(),
        };

        let result = self.remote_vault.soft_delete_cipher(command).await?;

        // Fetch the updated cipher from remote to get the deleted_date
        let sync_command = SyncVaultCommand {
            account_id: account_id.clone(),
            base_url,
            access_token,
            exclude_domains: false,
            trigger: SyncTrigger::Manual,
        };

        if let Ok(updated_cipher) = self
            .remote_vault
            .get_cipher(sync_command, cipher_id.clone())
            .await
        {
            self.vault_repository
                .upsert_cipher(&account_id, &updated_cipher)
                .await?;
        }

        self.event_port
            .emit_cipher_updated(&account_id, &result.cipher_id);

        Ok(result)
    }
}
