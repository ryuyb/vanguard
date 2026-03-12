use std::sync::Arc;

use crate::application::dto::sync::{SyncCipher, SyncVaultCommand};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct FetchCipherUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl FetchCipherUseCase {
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
    ) -> AppResult<SyncCipher> {
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

        let command = SyncVaultCommand {
            account_id: account_id.clone(),
            base_url,
            access_token,
            exclude_domains: true,
            trigger: crate::domain::sync::SyncTrigger::Manual,
        };

        let cipher = self
            .remote_vault
            .get_cipher(command, cipher_id.clone())
            .await?;

        self.vault_repository
            .upsert_cipher(&account_id, &cipher)
            .await?;

        self.event_port.emit_cipher_updated(&account_id, &cipher_id);

        Ok(cipher)
    }
}
