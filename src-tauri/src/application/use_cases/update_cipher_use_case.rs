use std::sync::Arc;

use crate::application::dto::sync::{CipherMutationResult, SyncCipher, UpdateCipherCommand};
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

        let command = UpdateCipherCommand {
            account_id: account_id.clone(),
            base_url,
            access_token,
            cipher_id: cipher_id.clone(),
            cipher,
        };

        let result = self.remote_vault.update_cipher(command).await?;

        let cipher_with_id = SyncCipher {
            id: result.cipher_id.clone(),
            revision_date: Some(result.revision_date.clone()),
            ..self.build_cipher_from_result(&result)
        };

        self.vault_repository
            .upsert_cipher(&account_id, &cipher_with_id)
            .await?;

        self.event_port
            .emit_cipher_updated(&account_id, &result.cipher_id);

        Ok(result)
    }

    fn build_cipher_from_result(&self, _result: &CipherMutationResult) -> SyncCipher {
        SyncCipher {
            id: String::new(),
            organization_id: None,
            folder_id: None,
            r#type: None,
            name: None,
            notes: None,
            key: None,
            favorite: None,
            edit: None,
            view_password: None,
            organization_use_totp: None,
            creation_date: None,
            revision_date: None,
            deleted_date: None,
            archived_date: None,
            reprompt: None,
            permissions: None,
            object: None,
            fields: Vec::new(),
            password_history: Vec::new(),
            collection_ids: Vec::new(),
            data: None,
            login: None,
            secure_note: None,
            card: None,
            identity: None,
            ssh_key: None,
            attachments: Vec::new(),
        }
    }
}
