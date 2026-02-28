use async_trait::async_trait;

use crate::application::dto::sync::{
    SyncCipher, SyncCollection, SyncDomains, SyncFolder, SyncPolicy, SyncProfile, SyncSend,
    SyncUserDecryption,
};
use crate::domain::sync::{SyncContext, SyncItemCounts};
use crate::support::result::AppResult;

#[async_trait]
pub trait VaultRepositoryPort: Send + Sync {
    async fn set_sync_running(&self, account_id: &str, base_url: &str) -> AppResult<SyncContext>;

    async fn set_sync_succeeded(
        &self,
        account_id: &str,
        base_url: &str,
        revision_ms: Option<i64>,
        synced_at_ms: i64,
        counts: SyncItemCounts,
    ) -> AppResult<SyncContext>;

    async fn set_sync_failed(
        &self,
        account_id: &str,
        base_url: &str,
        error_message: String,
    ) -> AppResult<SyncContext>;

    async fn get_sync_context(&self, account_id: &str) -> AppResult<Option<SyncContext>>;

    async fn begin_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn commit_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn rollback_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn upsert_profile(&self, account_id: &str, profile: SyncProfile) -> AppResult<()>;

    async fn upsert_folders(&self, account_id: &str, folders: Vec<SyncFolder>) -> AppResult<()>;

    async fn upsert_collections(
        &self,
        account_id: &str,
        collections: Vec<SyncCollection>,
    ) -> AppResult<()>;

    async fn upsert_policies(
        &self,
        account_id: &str,
        policies: Vec<SyncPolicy>,
    ) -> AppResult<()>;

    async fn upsert_ciphers(&self, account_id: &str, ciphers: Vec<SyncCipher>) -> AppResult<()>;

    async fn upsert_sends(&self, account_id: &str, sends: Vec<SyncSend>) -> AppResult<()>;

    async fn upsert_domains(&self, account_id: &str, domains: Option<SyncDomains>)
        -> AppResult<()>;

    async fn upsert_user_decryption(
        &self,
        account_id: &str,
        user_decryption: Option<SyncUserDecryption>,
    ) -> AppResult<()>;
}
