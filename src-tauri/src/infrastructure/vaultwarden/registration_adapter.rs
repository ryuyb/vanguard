use crate::application::dto::auth::{RegistrationOutcome, SendVerificationEmailCommand};
use crate::support::error::AppError;
use crate::support::result::AppResult;

use super::error::VaultwardenError;
use super::models::RegisterRequest;
use super::VaultwardenClient;

#[derive(Clone)]
pub struct VaultwardenRegistrationAdapter {
    client: VaultwardenClient,
}

impl VaultwardenRegistrationAdapter {
    pub fn new(client: VaultwardenClient) -> Self {
        Self { client }
    }

    pub async fn send_verification_email(
        &self,
        command: SendVerificationEmailCommand,
    ) -> AppResult<RegistrationOutcome> {
        log::debug!(
            target: "vanguard::vaultwarden",
            "send_verification_email request base_url={} has_name={}",
            command.base_url,
            command.name.is_some()
        );

        match self
            .client
            .register(
                &command.base_url,
                RegisterRequest {
                    email: command.email,
                    name: command.name,
                    receive_marketing_emails: false,
                },
            )
            .await
        {
            Ok((200, Some(response))) => {
                let token = response.token.unwrap_or_default();
                if token.trim().is_empty() {
                    log::error!(
                        target: "vanguard::vaultwarden",
                        "send_verification_email got 200 without token"
                    );
                    return Err(AppError::NetworkRemoteError {
                        status: 200,
                        message: "register response missing token".to_string(),
                    });
                }

                log::info!(
                    target: "vanguard::vaultwarden",
                    "send_verification_email result=direct_registration"
                );
                Ok(RegistrationOutcome::DirectRegistration { token })
            }
            Ok((204, _)) => {
                log::info!(
                    target: "vanguard::vaultwarden",
                    "send_verification_email result=email_verification_required"
                );
                Ok(RegistrationOutcome::EmailVerificationRequired)
            }
            Ok((status, _)) => {
                log::warn!(
                    target: "vanguard::vaultwarden",
                    "send_verification_email unexpected_status={}",
                    status
                );
                Err(AppError::NetworkRemoteError {
                    status,
                    message: format!("unexpected register response status {status}"),
                })
            }
            Err(VaultwardenError::ApiError {
                status: 400,
                message,
                ..
            }) => {
                log::warn!(
                    target: "vanguard::vaultwarden",
                    "send_verification_email result=registration_disabled"
                );
                Ok(RegistrationOutcome::Disabled { message })
            }
            Err(error) => {
                let mapped = map_vaultwarden_error(error);
                log::error!(
                    target: "vanguard::vaultwarden",
                    "send_verification_email failed message={}",
                    mapped.log_message()
                );
                Err(mapped)
            }
        }
    }
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
