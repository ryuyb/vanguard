use async_trait::async_trait;

use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::support::result::AppResult;

#[async_trait]
pub trait VaultRuntimePort: Send + Sync {
    async fn active_account_id(&self) -> AppResult<String>;
    async fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>>;
    async fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>>;
    async fn get_vault_user_key_material(
        &self,
        account_id: &str,
    ) -> AppResult<Option<VaultUserKeyMaterial>>;
    async fn set_vault_user_key_material(
        &self,
        account_id: String,
        key: VaultUserKeyMaterial,
    ) -> AppResult<()>;
    async fn remove_vault_user_key_material(&self, account_id: &str) -> AppResult<()>;
    /// Get the refresh token from the current auth session, if available.
    /// Returns None if no session exists or session has no refresh token.
    async fn get_refresh_token(&self) -> AppResult<Option<String>>;
}
