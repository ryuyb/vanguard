use async_trait::async_trait;

use crate::application::dto::auth::{
    PasswordLoginCommand, PasswordLoginOutcome, PreloginInfo, PreloginQuery, RefreshTokenCommand,
    SendEmailLoginCommand, SessionInfo, TwoFactorChallenge, VerifyEmailTokenCommand,
};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

use super::error::VaultwardenError;
use super::models::{
    PasswordLoginRequest, PreloginRequest, RefreshTokenRequest, SendEmailLoginRequest,
    TokenErrorResponse, TokenResponse, VerifyEmailTokenRequest,
};
use super::password_hash::derive_master_password_hash;
use super::VaultwardenClient;

#[derive(Clone)]
pub struct VaultwardenRemotePort {
    client: VaultwardenClient,
}

impl VaultwardenRemotePort {
    pub fn new(client: VaultwardenClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RemoteVaultPort for VaultwardenRemotePort {
    async fn prelogin(&self, query: PreloginQuery) -> AppResult<PreloginInfo> {
        let response = self
            .client
            .prelogin(&query.base_url, PreloginRequest { email: query.email })
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(PreloginInfo {
            kdf: response.kdf,
            kdf_iterations: response.kdf_iterations,
            kdf_memory: response.kdf_memory,
            kdf_parallelism: response.kdf_parallelism,
        })
    }

    async fn login_with_password(
        &self,
        command: PasswordLoginCommand,
    ) -> AppResult<PasswordLoginOutcome> {
        let prelogin = self
            .client
            .prelogin(
                &command.base_url,
                PreloginRequest {
                    email: command.username.clone(),
                },
            )
            .await
            .map_err(map_vaultwarden_error)?;

        let master_password_hash =
            derive_master_password_hash(&command.username, &command.password, &prelogin).map_err(
                |error| {
                    AppError::validation(format!(
                        "unable to derive master password hash from prelogin params: {error}"
                    ))
                },
            )?;

        let request = PasswordLoginRequest {
            client_id: self.client.client_id().to_string(),
            username: command.username,
            password: master_password_hash,
            scope: self.client.scope().to_string(),
            device_identifier: self.client.device_identifier().to_string(),
            device_name: self.client.device_name().to_string(),
            device_type: self.client.device_type().to_string(),
            two_factor_provider: command.two_factor_provider,
            two_factor_token: command.two_factor_token,
            two_factor_remember: command
                .two_factor_remember
                .map(|value| if value { 1 } else { 0 }),
            authrequest: command.authrequest,
        };

        let result = self
            .client
            .login_with_password(&command.base_url, request)
            .await;

        match result {
            Ok(token) => Ok(PasswordLoginOutcome::Authenticated(map_session(token))),
            Err(VaultwardenError::TokenRejected { error, .. })
                if is_two_factor_required(&error) =>
            {
                Ok(PasswordLoginOutcome::TwoFactorRequired(
                    TwoFactorChallenge {
                        error: error.error,
                        error_description: error.error_description,
                        providers: error.two_factor_providers.unwrap_or_default(),
                        providers2: error.two_factor_providers2,
                        master_password_policy: error.master_password_policy,
                    },
                ))
            }
            Err(error) => Err(map_vaultwarden_error(error)),
        }
    }

    async fn refresh_token(&self, command: RefreshTokenCommand) -> AppResult<SessionInfo> {
        let response = self
            .client
            .refresh_token(
                &command.base_url,
                RefreshTokenRequest {
                    refresh_token: command.refresh_token,
                    client_id: Some(self.client.client_id().to_string()),
                },
            )
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(map_session(response))
    }

    async fn send_email_login(&self, command: SendEmailLoginCommand) -> AppResult<()> {
        let derived_hash = if let (Some(email), Some(plaintext_password)) =
            (&command.email, &command.plaintext_password)
        {
            let prelogin = self
                .client
                .prelogin(
                    &command.base_url,
                    PreloginRequest {
                        email: email.clone(),
                    },
                )
                .await
                .map_err(map_vaultwarden_error)?;

            Some(
                derive_master_password_hash(email, plaintext_password, &prelogin).map_err(
                    |error| {
                        AppError::validation(format!(
                            "unable to derive master password hash for send-email-login: {error}"
                        ))
                    },
                )?,
            )
        } else {
            None
        };

        self.client
            .send_email_login(
                &command.base_url,
                SendEmailLoginRequest {
                    device_identifier: self.client.device_identifier().to_string(),
                    email: command.email,
                    master_password_hash: derived_hash,
                    auth_request_id: command.auth_request_id,
                    auth_request_access_code: command.auth_request_access_code,
                },
            )
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(())
    }

    async fn verify_email_token(&self, command: VerifyEmailTokenCommand) -> AppResult<()> {
        self.client
            .verify_email_token(
                &command.base_url,
                VerifyEmailTokenRequest {
                    user_id: command.user_id,
                    token: command.token,
                },
            )
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(())
    }
}

fn map_session(response: TokenResponse) -> SessionInfo {
    SessionInfo {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_in: response.expires_in,
        token_type: response.token_type,
        scope: response.scope,
        key: response.key,
        private_key: response.private_key,
        kdf: response.kdf,
        kdf_iterations: response.kdf_iterations,
        kdf_memory: response.kdf_memory,
        kdf_parallelism: response.kdf_parallelism,
        two_factor_token: response.two_factor_token,
    }
}

fn is_two_factor_required(error: &TokenErrorResponse) -> bool {
    if error.two_factor_providers.is_some() {
        return true;
    }

    error
        .error_description
        .as_ref()
        .map(|value| value.to_lowercase().contains("two factor required"))
        .unwrap_or(false)
}

fn map_vaultwarden_error(error: VaultwardenError) -> AppError {
    match error {
        VaultwardenError::MissingBaseUrl => AppError::validation("base_url cannot be empty"),
        VaultwardenError::InvalidEndpoint(message) => AppError::validation(message),
        VaultwardenError::Transport(message) => AppError::remote(message),
        VaultwardenError::Decode(message) => AppError::remote(message),
        VaultwardenError::ApiError {
            status, message, ..
        } => AppError::remote_status(status, message),
        VaultwardenError::TokenRejected { status, error } => {
            let message = error
                .error_description
                .or(error.error)
                .unwrap_or_else(|| String::from("token rejected"));
            AppError::remote_status(status, message)
        }
    }
}
