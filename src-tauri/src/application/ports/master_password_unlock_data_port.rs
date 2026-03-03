use async_trait::async_trait;

use crate::domain::unlock::MasterPasswordUnlockData;
use crate::support::result::AppResult;

#[async_trait]
pub trait MasterPasswordUnlockDataPort: Send + Sync {
    async fn load_master_password_unlock_data(
        &self,
        account_id: &str,
    ) -> AppResult<Option<MasterPasswordUnlockData>>;

    async fn save_master_password_unlock_data(
        &self,
        account_id: &str,
        data: &MasterPasswordUnlockData,
    ) -> AppResult<()>;

    async fn delete_master_password_unlock_data(&self, account_id: &str) -> AppResult<()>;
}
