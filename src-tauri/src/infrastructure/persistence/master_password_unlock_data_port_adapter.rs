use std::sync::Arc;

use async_trait::async_trait;

use crate::application::dto::sync::{
    SyncKdfParams, SyncMasterPasswordUnlock, SyncUserDecryption,
};
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::domain::unlock::{MasterPasswordUnlockData, MasterPasswordUnlockKdf};
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct SqliteMasterPasswordUnlockDataPort {
    vault_repository: Arc<dyn VaultRepositoryPort>,
}

impl SqliteMasterPasswordUnlockDataPort {
    pub fn new(vault_repository: Arc<dyn VaultRepositoryPort>) -> Self {
        Self { vault_repository }
    }
}

#[async_trait]
impl MasterPasswordUnlockDataPort for SqliteMasterPasswordUnlockDataPort {
    async fn load_master_password_unlock_data(
        &self,
        account_id: &str,
    ) -> AppResult<Option<MasterPasswordUnlockData>> {
        let user_decryption = self
            .vault_repository
            .load_live_user_decryption(account_id)
            .await?;
        Ok(extract_master_password_unlock_data(user_decryption))
    }

    async fn save_master_password_unlock_data(
        &self,
        account_id: &str,
        data: &MasterPasswordUnlockData,
    ) -> AppResult<()> {
        let payload = SyncUserDecryption {
            master_password_unlock: Some(SyncMasterPasswordUnlock {
                kdf: Some(SyncKdfParams {
                    kdf_type: Some(data.kdf.kdf_type),
                    iterations: Some(data.kdf.iterations),
                    memory: data.kdf.memory,
                    parallelism: data.kdf.parallelism,
                }),
                master_key_encrypted_user_key: None,
                master_key_wrapped_user_key: Some(data.master_key_wrapped_user_key.clone()),
                salt: Some(data.salt.clone()),
            }),
        };

        self.vault_repository.begin_sync_transaction(account_id).await?;
        let update_result = self
            .vault_repository
            .upsert_user_decryption(account_id, Some(payload))
            .await;
        if let Err(error) = update_result {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        if let Err(error) = self.vault_repository.commit_sync_transaction(account_id).await {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        Ok(())
    }

    async fn delete_master_password_unlock_data(&self, account_id: &str) -> AppResult<()> {
        self.vault_repository.begin_sync_transaction(account_id).await?;
        let update_result = self
            .vault_repository
            .upsert_user_decryption(account_id, None)
            .await;
        if let Err(error) = update_result {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        if let Err(error) = self.vault_repository.commit_sync_transaction(account_id).await {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        Ok(())
    }
}

fn extract_master_password_unlock_data(
    value: Option<SyncUserDecryption>,
) -> Option<MasterPasswordUnlockData> {
    let value = value?;
    let unlock = value.master_password_unlock?;
    let kdf = unlock.kdf?;
    let kdf_type = kdf.kdf_type?;
    let iterations = kdf.iterations?;
    let salt = unlock
        .salt
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())?;
    let master_key_wrapped_user_key = unlock
        .master_key_wrapped_user_key
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())?;

    Some(MasterPasswordUnlockData {
        kdf: MasterPasswordUnlockKdf {
            kdf_type,
            iterations,
            memory: kdf.memory,
            parallelism: kdf.parallelism,
        },
        salt,
        master_key_wrapped_user_key,
    })
}
