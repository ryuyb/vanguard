use async_trait::async_trait;

use crate::application::dto::sync::{
    SyncCipher, SyncCollection, SyncDomains, SyncFolder, SyncPolicy, SyncProfile, SyncSend,
    SyncUserDecryption,
};
use crate::domain::sync::{SyncContext, SyncItemCounts, VaultSnapshotMeta, WsStatus};
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

    async fn set_sync_degraded(
        &self,
        account_id: &str,
        base_url: &str,
        error_message: String,
    ) -> AppResult<SyncContext>;

    async fn get_sync_context(&self, account_id: &str) -> AppResult<Option<SyncContext>>;

    async fn set_ws_status(&self, account_id: &str, ws_status: WsStatus) -> AppResult<SyncContext>;

    async fn begin_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn commit_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn rollback_sync_transaction(&self, account_id: &str) -> AppResult<()>;

    async fn upsert_profile(&self, account_id: &str, profile: SyncProfile) -> AppResult<()>;

    async fn upsert_folders(&self, account_id: &str, folders: Vec<SyncFolder>) -> AppResult<()>;

    async fn upsert_folder_live(&self, account_id: &str, folder: SyncFolder) -> AppResult<()>;

    async fn delete_folder_live(&self, account_id: &str, folder_id: &str) -> AppResult<()>;

    async fn count_live_folders(&self, account_id: &str) -> AppResult<u32>;

    async fn list_live_folders(&self, account_id: &str) -> AppResult<Vec<SyncFolder>>;

    async fn upsert_collections(
        &self,
        account_id: &str,
        collections: Vec<SyncCollection>,
    ) -> AppResult<()>;

    async fn upsert_policies(&self, account_id: &str, policies: Vec<SyncPolicy>) -> AppResult<()>;

    async fn upsert_ciphers(&self, account_id: &str, ciphers: Vec<SyncCipher>) -> AppResult<()>;

    async fn upsert_cipher_live(&self, account_id: &str, cipher: SyncCipher) -> AppResult<()>;

    async fn upsert_cipher(&self, account_id: &str, cipher: &SyncCipher) -> AppResult<()>;

    async fn delete_cipher(&self, account_id: &str, cipher_id: &str) -> AppResult<()>;

    async fn delete_cipher_live(&self, account_id: &str, cipher_id: &str) -> AppResult<()>;

    async fn count_live_ciphers(&self, account_id: &str) -> AppResult<u32>;

    async fn get_live_cipher(
        &self,
        account_id: &str,
        cipher_id: &str,
    ) -> AppResult<Option<SyncCipher>>;

    async fn list_live_ciphers(
        &self,
        account_id: &str,
        offset: u32,
        limit: u32,
    ) -> AppResult<Vec<SyncCipher>>;

    async fn upsert_sends(&self, account_id: &str, sends: Vec<SyncSend>) -> AppResult<()>;

    async fn upsert_send_live(&self, account_id: &str, send: SyncSend) -> AppResult<()>;

    async fn delete_send_live(&self, account_id: &str, send_id: &str) -> AppResult<()>;

    async fn count_live_sends(&self, account_id: &str) -> AppResult<u32>;

    async fn upsert_domains(&self, account_id: &str, domains: Option<SyncDomains>)
        -> AppResult<()>;

    async fn upsert_user_decryption(
        &self,
        account_id: &str,
        user_decryption: Option<SyncUserDecryption>,
    ) -> AppResult<()>;

    async fn load_live_user_decryption(
        &self,
        account_id: &str,
    ) -> AppResult<Option<SyncUserDecryption>>;

    async fn save_snapshot_meta(
        &self,
        account_id: &str,
        snapshot_meta: VaultSnapshotMeta,
    ) -> AppResult<()>;

    async fn load_snapshot_meta(&self, account_id: &str) -> AppResult<Option<VaultSnapshotMeta>>;

    async fn delete_account_database(&self, account_id: &str) -> AppResult<()>;
}
