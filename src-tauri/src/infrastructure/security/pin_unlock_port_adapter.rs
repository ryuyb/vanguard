use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::application::ports::pin_unlock_port::PinUnlockPort;
use crate::domain::unlock::{PinLockType, PinProtectedUserKeyEnvelope};
use crate::infrastructure::security::pin_store;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone, Default)]
pub struct KeychainPinUnlockPort {
    ephemeral_envelopes: Arc<Mutex<HashMap<String, PinProtectedUserKeyEnvelope>>>,
}

impl KeychainPinUnlockPort {
    pub fn new() -> Self {
        Self::default()
    }

    fn normalize_account_id(account_id: &str) -> AppResult<String> {
        let trimmed = account_id.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(
                "account_id is empty, cannot use pin unlock",
            ));
        }
        Ok(String::from(trimmed))
    }

    fn save_ephemeral(
        &self,
        account_id: &str,
        envelope: &PinProtectedUserKeyEnvelope,
    ) -> AppResult<()> {
        let mut store = self
            .ephemeral_envelopes
            .lock()
            .map_err(|_| AppError::internal("failed to lock pin ephemeral envelope store"))?;
        store.insert(String::from(account_id), envelope.clone());
        Ok(())
    }

    fn load_ephemeral(&self, account_id: &str) -> AppResult<PinProtectedUserKeyEnvelope> {
        let store = self
            .ephemeral_envelopes
            .lock()
            .map_err(|_| AppError::internal("failed to lock pin ephemeral envelope store"))?;
        store.get(account_id).cloned().ok_or_else(|| {
            AppError::validation("ephemeral pin unlock is not configured for this account")
        })
    }

    fn has_ephemeral(&self, account_id: &str) -> AppResult<bool> {
        let store = self
            .ephemeral_envelopes
            .lock()
            .map_err(|_| AppError::internal("failed to lock pin ephemeral envelope store"))?;
        Ok(store.contains_key(account_id))
    }

    fn delete_ephemeral(&self, account_id: &str) -> AppResult<()> {
        let mut store = self
            .ephemeral_envelopes
            .lock()
            .map_err(|_| AppError::internal("failed to lock pin ephemeral envelope store"))?;
        store.remove(account_id);
        Ok(())
    }
}

#[async_trait]
impl PinUnlockPort for KeychainPinUnlockPort {
    fn is_supported(&self) -> bool {
        pin_store::is_supported()
    }

    async fn save_pin_envelope(
        &self,
        account_id: &str,
        lock_type: PinLockType,
        envelope: &PinProtectedUserKeyEnvelope,
    ) -> AppResult<()> {
        let account_id = Self::normalize_account_id(account_id)?;

        match lock_type {
            PinLockType::Persistent => pin_store::save_persistent_pin_envelope(
                &account_id,
                &pin_store::PersistentPinEnvelope::new(account_id.clone(), envelope.clone()),
            ),
            PinLockType::Ephemeral => self.save_ephemeral(&account_id, envelope),
            PinLockType::Disabled => Err(AppError::validation(
                "cannot save pin envelope when pin lock type is disabled",
            )),
        }
    }

    async fn load_pin_envelope(
        &self,
        account_id: &str,
        lock_type: PinLockType,
    ) -> AppResult<PinProtectedUserKeyEnvelope> {
        let account_id = Self::normalize_account_id(account_id)?;

        match lock_type {
            PinLockType::Persistent => {
                pin_store::load_persistent_pin_envelope(&account_id).map(|payload| payload.envelope)
            }
            PinLockType::Ephemeral => self.load_ephemeral(&account_id),
            PinLockType::Disabled => Err(AppError::validation(
                "cannot load pin envelope when pin lock type is disabled",
            )),
        }
    }

    async fn has_pin_envelope(&self, account_id: &str, lock_type: PinLockType) -> AppResult<bool> {
        let account_id = Self::normalize_account_id(account_id)?;

        match lock_type {
            PinLockType::Persistent => pin_store::has_persistent_pin_envelope(&account_id),
            PinLockType::Ephemeral => self.has_ephemeral(&account_id),
            PinLockType::Disabled => Ok(false),
        }
    }

    async fn delete_pin_envelope(&self, account_id: &str, lock_type: PinLockType) -> AppResult<()> {
        let account_id = Self::normalize_account_id(account_id)?;

        match lock_type {
            PinLockType::Persistent => pin_store::delete_persistent_pin_envelope(&account_id),
            PinLockType::Ephemeral => self.delete_ephemeral(&account_id),
            PinLockType::Disabled => Ok(()),
        }
    }
}
