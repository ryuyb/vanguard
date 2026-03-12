use std::sync::Arc;

use crate::application::cipher_encryption;
use crate::application::dto::sync::{CipherMutationResult, SyncCipher, UpdateCipherCommand};
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct UpdateCipherUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    event_port: Arc<dyn SyncEventPort>,
}

impl UpdateCipherUseCase {
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
        cipher: SyncCipher,
        user_key: VaultUserKeyMaterial,
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

        // Encrypt cipher before sending to server
        let encrypted_cipher = cipher_encryption::encrypt_cipher(&cipher, &user_key)?;

        let command = UpdateCipherCommand {
            account_id: account_id.clone(),
            base_url,
            access_token,
            cipher_id: cipher_id.clone(),
            cipher: encrypted_cipher,
        };

        let result = self.remote_vault.update_cipher(command).await?;

        // Store the encrypted cipher locally
        let cipher_with_id = SyncCipher {
            id: result.cipher_id.clone(),
            revision_date: Some(result.revision_date.clone()),
            ..cipher
        };

        self.vault_repository
            .upsert_cipher(&account_id, &cipher_with_id)
            .await?;

        self.event_port
            .emit_cipher_updated(&account_id, &result.cipher_id);

        Ok(result)
    }
}
