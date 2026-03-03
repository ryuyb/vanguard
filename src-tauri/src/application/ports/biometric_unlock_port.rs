use crate::application::dto::vault::VaultBiometricBundle;
use crate::support::result::AppResult;

pub trait BiometricUnlockPort: Send + Sync {
    fn is_supported(&self) -> bool;
    fn save_unlock_bundle(&self, account_id: &str, bundle: &VaultBiometricBundle) -> AppResult<()>;
    fn load_unlock_bundle(&self, account_id: &str) -> AppResult<VaultBiometricBundle>;
    fn has_unlock_bundle(&self, account_id: &str) -> AppResult<bool>;
    fn delete_unlock_bundle(&self, account_id: &str) -> AppResult<()>;
}
