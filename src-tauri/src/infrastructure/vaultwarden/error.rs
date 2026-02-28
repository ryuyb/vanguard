use std::error::Error;
use std::fmt::{Display, Formatter};

use super::models::TokenErrorResponse;

#[derive(Debug)]
pub enum VaultwardenError {
    MissingBaseUrl,
    InvalidEndpoint(&'static str),
    Transport(String),
    Decode(String),
    ApiError {
        status: u16,
        message: String,
        body: Option<String>,
    },
    TokenRejected {
        status: u16,
        error: TokenErrorResponse,
    },
}

impl Display for VaultwardenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBaseUrl => write!(f, "vaultwarden base url is missing"),
            Self::InvalidEndpoint(endpoint) => {
                write!(f, "vaultwarden endpoint is invalid: {endpoint}")
            }
            Self::Transport(message) => write!(f, "vaultwarden transport error: {message}"),
            Self::Decode(message) => write!(f, "vaultwarden decode error: {message}"),
            Self::ApiError {
                status, message, ..
            } => {
                write!(f, "vaultwarden api error ({status}): {message}")
            }
            Self::TokenRejected { status, error } => {
                let description = error
                    .error_description
                    .clone()
                    .or(error.error.clone())
                    .unwrap_or_else(|| String::from("token rejected"));
                write!(f, "vaultwarden token rejected ({status}): {description}")
            }
        }
    }
}

impl Error for VaultwardenError {}

pub type VaultwardenResult<T> = Result<T, VaultwardenError>;
