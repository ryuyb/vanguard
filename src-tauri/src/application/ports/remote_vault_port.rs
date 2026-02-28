use async_trait::async_trait;

use crate::application::dto::auth::{
    PasswordLoginCommand, PasswordLoginOutcome, PreloginInfo, PreloginQuery, RefreshTokenCommand,
    SendEmailLoginCommand, SessionInfo, VerifyEmailTokenCommand,
};
use crate::support::result::AppResult;

#[async_trait]
pub trait RemoteVaultPort: Send + Sync {
    async fn prelogin(&self, query: PreloginQuery) -> AppResult<PreloginInfo>;

    async fn login_with_password(
        &self,
        command: PasswordLoginCommand,
    ) -> AppResult<PasswordLoginOutcome>;

    async fn refresh_token(&self, command: RefreshTokenCommand) -> AppResult<SessionInfo>;

    async fn send_email_login(&self, command: SendEmailLoginCommand) -> AppResult<()>;

    async fn verify_email_token(&self, command: VerifyEmailTokenCommand) -> AppResult<()>;
}
