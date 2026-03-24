use std::fmt::{Display, Formatter};

use crate::support::redaction::redact_sensitive;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug)]
pub enum AppError {
    // === 认证错误 ===
    AuthNotAuthenticated,
    AuthInvalidCredentials,
    AuthTokenExpired,
    AuthTokenInvalid,
    AuthPermissionDenied,
    AuthAccountLocked,
    AuthTwoFactorRequired,
    AuthInvalidPin,

    // === 保险库错误 ===
    VaultCipherNotFound { cipher_id: String },
    VaultDecryptionFailed { reason: String },
    VaultSyncConflict { cipher_id: String },
    VaultLocked,
    VaultCorrupted,
    VaultFolderNotFound { folder_id: String },
    VaultFolderNameConflict { name: String },

    // === 验证错误 ===
    ValidationFieldError { field: String, message: String },
    ValidationFormatError { format: String, value: String },
    ValidationRequired { field: String },

    // === 请求错误 ===
    BadRequest { message: String },
    NotFound { resource: String, id: String },

    // === 网络错误 ===
    NetworkConnectionFailed,
    NetworkTimeout,
    NetworkRemoteError { status: u16, message: String },
    NetworkDnsResolutionFailed,

    // === 存储错误 ===
    StorageDatabaseError { operation: String, details: String },
    StorageFileNotFound { path: String },
    StoragePermissionDenied { path: String },

    // === 加密错误 ===
    CryptoKeyDerivationFailed,
    CryptoEncryptionFailed,
    CryptoDecryptionFailed,
    CryptoInvalidKey,

    // === 内部错误 ===
    InternalUnexpected { message: String },
    InternalNotImplemented { feature: String },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            // 认证
            Self::AuthNotAuthenticated => "AUTH_NOT_AUTHENTICATED",
            Self::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            Self::AuthTokenExpired => "AUTH_TOKEN_EXPIRED",
            Self::AuthTokenInvalid => "AUTH_TOKEN_INVALID",
            Self::AuthPermissionDenied => "AUTH_PERMISSION_DENIED",
            Self::AuthAccountLocked => "AUTH_ACCOUNT_LOCKED",
            Self::AuthTwoFactorRequired => "AUTH_TWO_FACTOR_REQUIRED",
            Self::AuthInvalidPin => "AUTH_INVALID_PIN",

            // 保险库
            Self::VaultCipherNotFound { .. } => "VAULT_CIPHER_NOT_FOUND",
            Self::VaultDecryptionFailed { .. } => "VAULT_DECRYPTION_FAILED",
            Self::VaultSyncConflict { .. } => "VAULT_SYNC_CONFLICT",
            Self::VaultLocked => "VAULT_LOCKED",
            Self::VaultCorrupted => "VAULT_CORRUPTED",
            Self::VaultFolderNotFound { .. } => "VAULT_FOLDER_NOT_FOUND",
            Self::VaultFolderNameConflict { .. } => "VAULT_FOLDER_NAME_CONFLICT",

            // 验证
            Self::ValidationFieldError { .. } => "VALIDATION_FIELD_ERROR",
            Self::ValidationFormatError { .. } => "VALIDATION_FORMAT_ERROR",
            Self::ValidationRequired { .. } => "VALIDATION_REQUIRED",

            // 请求
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::NotFound { .. } => "NOT_FOUND",

            // 网络
            Self::NetworkConnectionFailed => "NETWORK_CONNECTION_FAILED",
            Self::NetworkTimeout => "NETWORK_TIMEOUT",
            Self::NetworkRemoteError { .. } => "NETWORK_REMOTE_ERROR",
            Self::NetworkDnsResolutionFailed => "NETWORK_DNS_RESOLUTION_FAILED",

            // 存储
            Self::StorageDatabaseError { .. } => "STORAGE_DATABASE_ERROR",
            Self::StorageFileNotFound { .. } => "STORAGE_FILE_NOT_FOUND",
            Self::StoragePermissionDenied { .. } => "STORAGE_PERMISSION_DENIED",

            // 加密
            Self::CryptoKeyDerivationFailed => "CRYPTO_KEY_DERIVATION_FAILED",
            Self::CryptoEncryptionFailed => "CRYPTO_ENCRYPTION_FAILED",
            Self::CryptoDecryptionFailed => "CRYPTO_DECRYPTION_FAILED",
            Self::CryptoInvalidKey => "CRYPTO_INVALID_KEY",

            // 内部
            Self::InternalUnexpected { .. } => "INTERNAL_UNEXPECTED",
            Self::InternalNotImplemented { .. } => "INTERNAL_NOT_IMPLEMENTED",
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Self::NetworkRemoteError { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            // 认证
            Self::AuthNotAuthenticated => "Not authenticated. Please log in.".to_string(),
            Self::AuthInvalidCredentials => "Invalid credentials".to_string(),
            Self::AuthTokenExpired => "Authentication token expired".to_string(),
            Self::AuthTokenInvalid => "Invalid authentication token".to_string(),
            Self::AuthPermissionDenied => "Permission denied".to_string(),
            Self::AuthAccountLocked => "Account is locked".to_string(),
            Self::AuthTwoFactorRequired => "Two-factor authentication required".to_string(),
            Self::AuthInvalidPin => "Invalid PIN".to_string(),

            // 保险库
            Self::VaultCipherNotFound { cipher_id } => format!("Cipher not found: {}", cipher_id),
            Self::VaultDecryptionFailed { reason } => format!("Decryption failed: {}", reason),
            Self::VaultSyncConflict { cipher_id } => {
                format!("Sync conflict for cipher: {}", cipher_id)
            }
            Self::VaultLocked => "Vault is locked".to_string(),
            Self::VaultCorrupted => "Vault data is corrupted".to_string(),
            Self::VaultFolderNotFound { folder_id } => format!("Folder not found: {}", folder_id),
            Self::VaultFolderNameConflict { name } => {
                format!("Folder name already exists: {}", name)
            }

            // 验证
            Self::ValidationFieldError { field, message } => {
                format!("Validation error in field '{}': {}", field, message)
            }
            Self::ValidationFormatError { format, value } => {
                format!("Invalid format for {}: {}", format, value)
            }
            Self::ValidationRequired { field } => format!("Required field missing: {}", field),

            // 请求
            Self::BadRequest { message } => format!("Bad request: {}", message),
            Self::NotFound { resource, id } => format!("{} not found: {}", resource, id),

            // 网络
            Self::NetworkConnectionFailed => "Network connection failed".to_string(),
            Self::NetworkTimeout => "Network request timed out".to_string(),
            Self::NetworkRemoteError { status, message } => {
                format!("Remote server error ({}): {}", status, message)
            }
            Self::NetworkDnsResolutionFailed => "DNS resolution failed".to_string(),

            // 存储
            Self::StorageDatabaseError { operation, details } => {
                format!("Database error during {}: {}", operation, details)
            }
            Self::StorageFileNotFound { path } => format!("File not found: {}", path),
            Self::StoragePermissionDenied { path } => format!("Permission denied for: {}", path),

            // 加密
            Self::CryptoKeyDerivationFailed => "Key derivation failed".to_string(),
            Self::CryptoEncryptionFailed => "Encryption failed".to_string(),
            Self::CryptoDecryptionFailed => "Decryption failed".to_string(),
            Self::CryptoInvalidKey => "Invalid encryption key".to_string(),

            // 内部
            Self::InternalUnexpected { message } => format!("Unexpected error: {}", message),
            Self::InternalNotImplemented { feature } => {
                format!("Feature not implemented: {}", feature)
            }
        }
    }

    pub fn to_payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: String::from(self.code()),
            message: self.message(),
            details: self.details(),
            timestamp: chrono::Utc::now().timestamp(),
            severity: self.severity(),
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Info
            Self::AuthTwoFactorRequired => ErrorSeverity::Info,

            // Warning
            Self::ValidationFieldError { .. }
            | Self::ValidationFormatError { .. }
            | Self::ValidationRequired { .. } => ErrorSeverity::Warning,

            // Fatal
            Self::InternalUnexpected { .. } | Self::VaultCorrupted | Self::CryptoInvalidKey => {
                ErrorSeverity::Fatal
            }

            // Error (default for most cases)
            _ => ErrorSeverity::Error,
        }
    }

    fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::ValidationFieldError { field, message } => Some(serde_json::json!({
                "field": field,
                "message": message,
            })),
            Self::ValidationFormatError { format, value } => Some(serde_json::json!({
                "format": format,
                "value": value,
            })),
            Self::ValidationRequired { field } => Some(serde_json::json!({
                "field": field,
            })),
            Self::VaultCipherNotFound { cipher_id } => Some(serde_json::json!({
                "cipherId": cipher_id,
            })),
            Self::VaultSyncConflict { cipher_id } => Some(serde_json::json!({
                "cipherId": cipher_id,
            })),
            Self::NetworkRemoteError { status, .. } => Some(serde_json::json!({
                "status": status,
            })),
            Self::StorageDatabaseError { operation, details } => Some(serde_json::json!({
                "operation": operation,
                "details": details,
            })),
            Self::StorageFileNotFound { path } => Some(serde_json::json!({
                "path": path,
            })),
            Self::StoragePermissionDenied { path } => Some(serde_json::json!({
                "path": path,
            })),
            Self::VaultFolderNotFound { folder_id } => Some(serde_json::json!({
                "folderId": folder_id,
            })),
            Self::VaultFolderNameConflict { name } => Some(serde_json::json!({
                "name": name,
            })),
            Self::BadRequest { message } => Some(serde_json::json!({
                "message": message,
            })),
            Self::NotFound { resource, id } => Some(serde_json::json!({
                "resource": resource,
                "id": id,
            })),
            _ => None,
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

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_payload().serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: i64,
    pub severity: ErrorSeverity,
}
