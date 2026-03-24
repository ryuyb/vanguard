//! Unlock Context Provider Port
//!
//! This module defines the port for accessing unlock context from the application layer.
//! It provides a unified interface for retrieving authentication and vault state,
//! replacing the legacy AuthSession-based approach.
//!
//! # Migration Guide
//!
//! ## Before (Legacy)
//! ```rust,ignore
//! let session = app_state.auth_session().await?;
//! let account_id = session.account_id;
//! let access_token = session.access_token;
//! ```
//!
//! ## After (New API)
//! ```rust,ignore
//! // For vault operations (need keys)
//! let ctx = unlock_manager.require_unlocked().await?;
//! let account_id = ctx.account.account_id;
//!
//! // For API operations (need session)
//! let ctx = unlock_manager.require_fully_unlocked().await?;
//! let access_token = ctx.session.access_token;
//! ```

use async_trait::async_trait;

use crate::bootstrap::unlock_state::{AccountContext, SessionContext, VaultKeyMaterial};
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Context for vault operations (keys available, vault unlocked)
///
/// This provides access to account information and encryption keys.
/// Use this when you need to perform cryptographic operations but don't
/// necessarily need API access.
#[derive(Debug, Clone)]
pub struct UnlockContext {
    /// Account information (non-sensitive)
    pub account: AccountContext,
    /// Vault encryption keys (sensitive, zeroized on drop)
    pub key: VaultKeyMaterial,
}

impl UnlockContext {
    /// Get account ID
    pub fn account_id(&self) -> &str {
        &self.account.account_id
    }

    /// Get base URL for API calls
    pub fn base_url(&self) -> &str {
        &self.account.base_url
    }

    /// Get email
    pub fn email(&self) -> &str {
        &self.account.email
    }

    /// Convert to vault unlock context DTO
    pub fn to_vault_context(&self) -> crate::application::dto::vault::VaultUnlockContext {
        self.account.to_vault_context()
    }

    /// Convert to user key material DTO
    pub fn to_key_material(&self) -> crate::application::dto::vault::VaultUserKeyMaterial {
        self.key.to_dto()
    }
}

/// Full context for operations requiring both vault access and API session
///
/// This provides complete access to account, session, and keys.
/// Use this when you need to make authenticated API calls.
#[derive(Debug, Clone)]
pub struct FullUnlockContext {
    /// Account information (non-sensitive)
    pub account: AccountContext,
    /// API session (contains access_token - sensitive)
    pub session: SessionContext,
    /// Vault encryption keys (sensitive, zeroized on drop)
    pub key: VaultKeyMaterial,
}

impl FullUnlockContext {
    /// Get account ID
    pub fn account_id(&self) -> &str {
        &self.account.account_id
    }

    /// Get base URL for API calls
    pub fn base_url(&self) -> &str {
        &self.account.base_url
    }

    /// Get email
    pub fn email(&self) -> &str {
        &self.account.email
    }

    /// Get access token for API authentication
    pub fn access_token(&self) -> &str {
        &self.session.access_token
    }

    /// Get refresh token if available
    pub fn refresh_token(&self) -> Option<&str> {
        self.session.refresh_token.as_deref()
    }

    /// Check if session is expiring soon
    pub fn is_session_expiring_soon(&self, threshold_secs: u64) -> bool {
        use std::time::Duration;
        self.session
            .is_expiring_within(Duration::from_secs(threshold_secs))
    }

    /// Convert to vault unlock context DTO
    pub fn to_vault_context(&self) -> crate::application::dto::vault::VaultUnlockContext {
        self.account.to_vault_context()
    }

    /// Convert to user key material DTO
    pub fn to_key_material(&self) -> crate::application::dto::vault::VaultUserKeyMaterial {
        self.key.to_dto()
    }

    /// Convert to unlock context (without session)
    pub fn to_unlock_context(&self) -> UnlockContext {
        UnlockContext {
            account: self.account.clone(),
            key: self.key.clone(),
        }
    }
}

/// Port for accessing unlock context
///
/// This trait abstracts the unlock state management and provides
/// a clean interface for the application layer.
#[async_trait]
pub trait UnlockContextProvider: Send + Sync {
    /// Require vault to be unlocked (keys available)
    ///
    /// Returns `UnlockContext` containing account info and keys.
    /// Fails if vault is locked.
    ///
    /// # Use Cases
    /// - Local cipher encryption/decryption
    /// - Vault operations that don't require API calls
    async fn require_unlocked(&self) -> AppResult<UnlockContext>;

    /// Require full unlock (vault unlocked + valid API session)
    ///
    /// Returns `FullUnlockContext` containing account, session, and keys.
    /// Fails if vault is locked or session has expired.
    ///
    /// # Use Cases
    /// - Creating/updating ciphers via API
    /// - Sync operations
    /// - Any operation requiring authenticated API calls
    async fn require_fully_unlocked(&self) -> AppResult<FullUnlockContext>;

    /// Check if vault is unlocked (no error if locked)
    ///
    /// Returns `Some(UnlockContext)` if unlocked, `None` if locked.
    /// Use this for optional operations where you don't want to fail.
    async fn try_get_unlocked(&self) -> Option<UnlockContext>;

    /// Check if fully unlocked (no error if not)
    ///
    /// Returns `Some(FullUnlockContext)` if fully unlocked, `None` otherwise.
    /// Use this for optional API operations.
    async fn try_get_fully_unlocked(&self) -> Option<FullUnlockContext>;

    /// Get account context if available (even if locked)
    ///
    /// Returns account info from persistence if vault was previously unlocked.
    /// This is useful for UI state restoration.
    async fn get_account_context(&self) -> Option<AccountContext>;
}

/// Errors that can occur when accessing unlock context
#[derive(Debug, Clone)]
pub enum UnlockContextError {
    VaultLocked,
    SessionExpired,
    NotAuthenticated,
}

impl UnlockContextError {
    pub fn to_message(&self) -> &'static str {
        match self {
            UnlockContextError::VaultLocked => {
                "Vault is locked. Please unlock with master password or PIN."
            }
            UnlockContextError::SessionExpired => {
                "API session has expired. Please lock and unlock to restore session."
            }
            UnlockContextError::NotAuthenticated => "Not authenticated. Please log in.",
        }
    }
}

impl From<UnlockContextError> for AppError {
    fn from(err: UnlockContextError) -> Self {
        match err {
            UnlockContextError::VaultLocked => AppError::VaultLocked,
            UnlockContextError::SessionExpired => AppError::AuthTokenExpired,
            UnlockContextError::NotAuthenticated => AppError::AuthNotAuthenticated,
        }
    }
}
