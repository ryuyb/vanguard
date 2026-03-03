use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::support::result::AppResult;

pub trait VaultRuntimePort: Send + Sync {
    fn active_account_id(&self) -> AppResult<String>;
    fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>>;
    fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>>;
    fn get_vault_user_key_material(
        &self,
        account_id: &str,
    ) -> AppResult<Option<VaultUserKeyMaterial>>;
    fn set_vault_user_key_material(
        &self,
        account_id: String,
        key: VaultUserKeyMaterial,
    ) -> AppResult<()>;
    fn remove_vault_user_key_material(&self, account_id: &str) -> AppResult<()>;
}
