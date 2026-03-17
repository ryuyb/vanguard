use async_trait::async_trait;

use crate::application::dto::auth::{
    MasterPasswordPolicy as AppMasterPasswordPolicy, PasswordLoginCommand, PasswordLoginOutcome,
    PreloginInfo, PreloginQuery, RefreshTokenCommand, SendEmailLoginCommand, SessionInfo,
    TwoFactorChallenge, TwoFactorProviderHint as AppTwoFactorProviderHint, VerifyEmailTokenCommand,
    WebauthnAllowCredential as AppWebauthnAllowCredential,
    WebauthnRequestExtensions as AppWebauthnRequestExtensions,
};
use crate::application::dto::sync::{
    CipherMutationResult, CreateCipherCommand, DeleteCipherCommand, RestoreCipherCommand,
    RevisionDateQuery, SoftDeleteCipherCommand, SyncCipher, SyncFolder, SyncSend, SyncVaultCommand,
    SyncVaultPayload, UpdateCipherCommand,
};
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

use super::error::VaultwardenError;
use super::mapper::{
    map_cipher_to_remote, map_sync_cipher, map_sync_folder, map_sync_response, map_sync_send,
};
use super::models::{
    PasswordLoginRequest, PreloginRequest, RefreshTokenRequest, SendEmailLoginRequest,
    TokenErrorResponse, TokenResponse, TwoFactorProviderHint, VerifyEmailTokenRequest,
    WebauthnAllowCredential, WebauthnRequestExtensions,
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

        let master_password_hash = derive_master_password_hash(
            &command.username,
            &command.password,
            &prelogin,
        )
        .map_err(|error| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("unable to derive master password hash from prelogin params: {error}"),
        })?;

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
                let error = *error;
                Ok(PasswordLoginOutcome::TwoFactorRequired(
                    TwoFactorChallenge {
                        error: error.error,
                        error_description: error.error_description,
                        providers: error.two_factor_providers.unwrap_or_default(),
                        providers2: error.two_factor_providers2.map(map_two_factor_provider_map),
                        master_password_policy: error
                            .master_password_policy
                            .map(map_master_password_policy),
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
                    |error| AppError::ValidationFieldError {
                        field: "unknown".to_string(),
                        message: format!(
                            "unable to derive master password hash for send-email-login: {error}"
                        ),
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

    async fn sync_vault(&self, command: SyncVaultCommand) -> AppResult<SyncVaultPayload> {
        let response = self
            .client
            .sync(
                &command.base_url,
                &command.access_token,
                command.exclude_domains,
            )
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(map_sync_response(response))
    }

    async fn get_cipher(
        &self,
        command: SyncVaultCommand,
        cipher_id: String,
    ) -> AppResult<SyncCipher> {
        let response = self
            .client
            .get_cipher(&command.base_url, &command.access_token, &cipher_id)
            .await
            .map_err(map_vaultwarden_error)?;
        Ok(map_sync_cipher(response))
    }

    async fn get_folder(
        &self,
        command: SyncVaultCommand,
        folder_id: String,
    ) -> AppResult<SyncFolder> {
        let response = self
            .client
            .get_folder(&command.base_url, &command.access_token, &folder_id)
            .await
            .map_err(map_vaultwarden_error)?;
        Ok(map_sync_folder(response))
    }

    async fn get_folders(&self, command: SyncVaultCommand) -> AppResult<Vec<SyncFolder>> {
        let response = self
            .client
            .get_folders(&command.base_url, &command.access_token)
            .await
            .map_err(map_vaultwarden_error)?;
        Ok(response.into_iter().map(map_sync_folder).collect())
    }

    async fn get_send(&self, command: SyncVaultCommand, send_id: String) -> AppResult<SyncSend> {
        let response = self
            .client
            .get_send(&command.base_url, &command.access_token, &send_id)
            .await
            .map_err(map_vaultwarden_error)?;
        Ok(map_sync_send(response))
    }

    async fn get_revision_date(&self, query: RevisionDateQuery) -> AppResult<i64> {
        self.client
            .revision_date(&query.base_url, &query.access_token)
            .await
            .map_err(map_vaultwarden_error)
    }

    async fn create_cipher(&self, command: CreateCipherCommand) -> AppResult<CipherMutationResult> {
        let remote_cipher = map_cipher_to_remote(&command.cipher);
        let response = self
            .client
            .create_cipher(&command.base_url, &command.access_token, &remote_cipher)
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(CipherMutationResult {
            cipher_id: response.id,
            revision_date: response.revision_date,
        })
    }

    async fn update_cipher(&self, command: UpdateCipherCommand) -> AppResult<CipherMutationResult> {
        let remote_cipher = map_cipher_to_remote(&command.cipher);
        let response = self
            .client
            .update_cipher(
                &command.base_url,
                &command.access_token,
                &command.cipher_id,
                &remote_cipher,
            )
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(CipherMutationResult {
            cipher_id: response.id,
            revision_date: response.revision_date,
        })
    }

    async fn delete_cipher(&self, command: DeleteCipherCommand) -> AppResult<()> {
        self.client
            .delete_cipher(&command.base_url, &command.access_token, &command.cipher_id)
            .await
            .map_err(map_vaultwarden_error)
    }

    async fn soft_delete_cipher(
        &self,
        command: SoftDeleteCipherCommand,
    ) -> AppResult<CipherMutationResult> {
        self.client
            .soft_delete_cipher(&command.base_url, &command.access_token, &command.cipher_id)
            .await
            .map_err(map_vaultwarden_error)?;

        Ok(CipherMutationResult {
            cipher_id: command.cipher_id,
            revision_date: String::new(),
        })
    }

    async fn restore_cipher(&self, command: RestoreCipherCommand) -> AppResult<()> {
        self.client
            .restore_cipher(&command.base_url, &command.access_token, &command.cipher_id)
            .await
            .map_err(map_vaultwarden_error)
    }
}

fn map_master_password_policy(
    policy: super::models::MasterPasswordPolicy,
) -> AppMasterPasswordPolicy {
    AppMasterPasswordPolicy {
        min_complexity: policy.min_complexity,
        min_length: policy.min_length,
        require_lower: policy.require_lower,
        require_upper: policy.require_upper,
        require_numbers: policy.require_numbers,
        require_special: policy.require_special,
        enforce_on_login: policy.enforce_on_login,
        object: policy.object,
    }
}

fn map_two_factor_provider_map(
    providers: std::collections::HashMap<String, Option<TwoFactorProviderHint>>,
) -> std::collections::HashMap<String, Option<AppTwoFactorProviderHint>> {
    providers
        .into_iter()
        .map(|(key, value)| (key, value.map(map_two_factor_provider_hint)))
        .collect()
}

fn map_two_factor_provider_hint(provider: TwoFactorProviderHint) -> AppTwoFactorProviderHint {
    AppTwoFactorProviderHint {
        host: provider.host,
        signature: provider.signature,
        auth_url: provider.auth_url,
        nfc: provider.nfc,
        email: provider.email,
        challenge: provider.challenge,
        timeout: provider.timeout,
        rp_id: provider.rp_id,
        allow_credentials: provider
            .allow_credentials
            .into_iter()
            .map(map_webauthn_allow_credential)
            .collect(),
        user_verification: provider.user_verification,
        extensions: provider.extensions.map(map_webauthn_request_extensions),
    }
}

fn map_webauthn_allow_credential(
    credential: WebauthnAllowCredential,
) -> AppWebauthnAllowCredential {
    AppWebauthnAllowCredential {
        r#type: credential.r#type,
        id: credential.id,
        transports: credential.transports,
    }
}

fn map_webauthn_request_extensions(
    extensions: WebauthnRequestExtensions,
) -> AppWebauthnRequestExtensions {
    AppWebauthnRequestExtensions {
        appid: extensions.appid,
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
        VaultwardenError::MissingBaseUrl => AppError::ValidationRequired {
            field: "base_url".to_string(),
        },
        VaultwardenError::InvalidEndpoint(message) => AppError::ValidationFieldError {
            field: "endpoint".to_string(),
            message: message.to_string(),
        },
        VaultwardenError::Transport(message) => AppError::NetworkRemoteError { status: 0, message },
        VaultwardenError::Decode(message) => AppError::NetworkRemoteError { status: 0, message },
        VaultwardenError::ApiError {
            status, message, ..
        } => AppError::NetworkRemoteError { status, message },
        VaultwardenError::TokenRejected { status, error } => {
            let error = *error;
            let message =
                first_non_empty(vec![error.error_description, error.error, error.message])
                    .unwrap_or_else(|| String::from("token rejected"));
            AppError::NetworkRemoteError { status, message }
        }
    }
}

fn first_non_empty(values: Vec<Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
}
