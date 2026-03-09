use std::sync::Arc;

use crate::application::dto::auth::{
    PasswordLoginCommand, PasswordLoginOutcome, PreloginInfo, PreloginQuery, RefreshTokenCommand,
    SendEmailLoginCommand, SessionInfo, VerifyEmailTokenCommand,
};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct AuthService {
    remote_vault: Arc<dyn RemoteVaultPort>,
}

impl AuthService {
    pub fn new(remote_vault: Arc<dyn RemoteVaultPort>) -> Self {
        Self { remote_vault }
    }

    pub async fn prelogin(&self, query: PreloginQuery) -> AppResult<PreloginInfo> {
        require_non_empty(&query.base_url, "base_url")?;
        require_non_empty(&query.email, "email")?;
        self.remote_vault.prelogin(query).await
    }

    pub async fn login_with_password(
        &self,
        command: PasswordLoginCommand,
    ) -> AppResult<PasswordLoginOutcome> {
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.username, "username")?;
        require_non_empty(&command.password, "password")?;
        self.remote_vault.login_with_password(command).await
    }

    pub async fn refresh_token(&self, command: RefreshTokenCommand) -> AppResult<SessionInfo> {
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.refresh_token, "refresh_token")?;
        self.remote_vault.refresh_token(command).await
    }

    pub async fn send_email_login(&self, command: SendEmailLoginCommand) -> AppResult<()> {
        require_non_empty(&command.base_url, "base_url")?;
        self.remote_vault.send_email_login(command).await
    }

    pub async fn verify_email_token(&self, command: VerifyEmailTokenCommand) -> AppResult<()> {
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.user_id, "user_id")?;
        require_non_empty(&command.token, "token")?;
        self.remote_vault.verify_email_token(command).await
    }
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationRequired {
            field: field.to_string(),
        });
    }

    Ok(())
}
