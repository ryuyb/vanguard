//! UnlockContextProvider Implementation for UnifiedUnlockManager
//!
//! This module implements the UnlockContextProvider trait for UnifiedUnlockManager,
//! providing a clean application-layer interface for accessing unlock state.

use async_trait::async_trait;

use crate::application::ports::unlock_context_port::{
    FullUnlockContext, UnlockContext, UnlockContextError, UnlockContextProvider,
};
use crate::bootstrap::unlock_state::{AccountContext, UnifiedUnlockManager};
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[async_trait]
impl UnlockContextProvider for UnifiedUnlockManager {
    async fn require_unlocked(&self) -> AppResult<UnlockContext> {
        let state = self.current_state().await;

        // Check if vault is unlocked (has key material)
        let key = state
            .key_material
            .ok_or_else(|| AppError::from(UnlockContextError::VaultLocked))?;

        // Check if account context exists
        let account = state
            .account_context
            .ok_or_else(|| AppError::from(UnlockContextError::NotAuthenticated))?;

        Ok(UnlockContext { account, key })
    }

    async fn require_fully_unlocked(&self) -> AppResult<FullUnlockContext> {
        let state = self.current_state().await;

        // Check if vault is unlocked
        let key = state
            .key_material
            .ok_or_else(|| AppError::from(UnlockContextError::VaultLocked))?;

        // Check if account context exists
        let account = state
            .account_context
            .ok_or_else(|| AppError::from(UnlockContextError::NotAuthenticated))?;

        // Check if session is valid
        let session = state
            .session_context
            .ok_or_else(|| AppError::from(UnlockContextError::SessionExpired))?;

        // Verify session is not expired
        if session.is_expired() {
            return Err(AppError::from(UnlockContextError::SessionExpired));
        }

        Ok(FullUnlockContext {
            account,
            session,
            key,
        })
    }

    async fn try_get_unlocked(&self) -> Option<UnlockContext> {
        let state = self.current_state().await;

        match (state.key_material, state.account_context) {
            (Some(key), Some(account)) => Some(UnlockContext { account, key }),
            _ => None,
        }
    }

    async fn try_get_fully_unlocked(&self) -> Option<FullUnlockContext> {
        let state = self.current_state().await;

        match (
            state.key_material,
            state.account_context,
            state.session_context,
        ) {
            (Some(key), Some(account), Some(session)) if !session.is_expired() => {
                Some(FullUnlockContext {
                    account,
                    session,
                    key,
                })
            }
            _ => None,
        }
    }

    async fn get_account_context(&self) -> Option<AccountContext> {
        self.account_context().await
    }
}

/// Extension trait for UnifiedUnlockManager to provide additional convenience methods
///
/// These methods are specifically designed to replace legacy AuthSession usage patterns.
#[async_trait]
pub trait UnifiedUnlockManagerExt: Send + Sync {
    /// Get account ID if vault is unlocked
    ///
    /// Replacement for: `auth_session.account_id`
    async fn unlocked_account_id(&self) -> AppResult<String>;

    /// Get base URL if vault is unlocked
    ///
    /// Replacement for: `auth_session.base_url`
    async fn unlocked_base_url(&self) -> AppResult<String>;

    /// Get access token if session is valid
    ///
    /// Replacement for: `auth_session.access_token`
    async fn access_token(&self) -> AppResult<String>;

    /// Check if we can make API calls (fully unlocked)
    ///
    /// Replacement for: `auth_session.is_some()`
    async fn can_make_api_calls(&self) -> bool;
}

#[async_trait]
impl UnifiedUnlockManagerExt for UnifiedUnlockManager {
    async fn unlocked_account_id(&self) -> AppResult<String> {
        self.require_unlocked()
            .await
            .map(|ctx| ctx.account.account_id)
    }

    async fn unlocked_base_url(&self) -> AppResult<String> {
        self.require_unlocked()
            .await
            .map(|ctx| ctx.account.base_url)
    }

    async fn access_token(&self) -> AppResult<String> {
        self.require_fully_unlocked()
            .await
            .map(|ctx| ctx.session.access_token)
    }

    async fn can_make_api_calls(&self) -> bool {
        self.try_get_fully_unlocked().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require mocking the persistence layer
    // For now, they serve as documentation of expected behavior

    #[test]
    fn test_unlock_context_error_mapping() {
        let err = UnlockContextError::VaultLocked;
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::VaultLocked));

        let err = UnlockContextError::SessionExpired;
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::AuthTokenExpired));

        let err = UnlockContextError::NotAuthenticated;
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::AuthNotAuthenticated));
    }
}
