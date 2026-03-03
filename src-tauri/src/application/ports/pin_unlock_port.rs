use async_trait::async_trait;

use crate::domain::unlock::{PinLockType, PinProtectedUserKeyEnvelope};
use crate::support::result::AppResult;

#[async_trait]
pub trait PinUnlockPort: Send + Sync {
    fn is_supported(&self) -> bool;

    async fn save_pin_envelope(
        &self,
        account_id: &str,
        lock_type: PinLockType,
        envelope: &PinProtectedUserKeyEnvelope,
    ) -> AppResult<()>;

    async fn load_pin_envelope(
        &self,
        account_id: &str,
        lock_type: PinLockType,
    ) -> AppResult<PinProtectedUserKeyEnvelope>;

    async fn has_pin_envelope(&self, account_id: &str, lock_type: PinLockType) -> AppResult<bool>;

    async fn delete_pin_envelope(
        &self,
        account_id: &str,
        lock_type: PinLockType,
    ) -> AppResult<()>;
}
