use std::fmt::{Display, Formatter};

use crate::support::redaction::redact_sensitive;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    Remote(String),
    RemoteStatus { status: u16, message: String },
    Internal(String),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn remote(message: impl Into<String>) -> Self {
        Self::Remote(message.into())
    }

    pub fn remote_status(status: u16, message: impl Into<String>) -> Self {
        Self::RemoteStatus {
            status,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::Remote(_) => "remote_error",
            Self::RemoteStatus { .. } => "remote_status_error",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Self::RemoteStatus { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Validation(message) => message.clone(),
            Self::Remote(message) => message.clone(),
            Self::RemoteStatus { status, message } => {
                format!("remote status {}: {}", status, message)
            }
            Self::Internal(message) => message.clone(),
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: String::from(self.code()),
            message: self.message(),
        }
    }

    pub fn log_message(&self) -> String {
        redact_sensitive(&self.message())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.log_message())
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}
