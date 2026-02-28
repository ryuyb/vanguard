use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::application::dto::sync::{
    SyncCipher, SyncCollection, SyncDomains, SyncFolder, SyncPolicy, SyncProfile, SyncSend,
    SyncUserDecryption,
};
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::domain::sync::{SyncContext, SyncItemCounts, SyncState};
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Default)]
pub struct InMemoryVaultRepository {
    contexts: RwLock<HashMap<String, SyncContext>>,
    vaults: RwLock<HashMap<String, AccountVaultSnapshot>>,
    staging: RwLock<HashMap<String, AccountVaultSnapshot>>,
}

impl InMemoryVaultRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl VaultRepositoryPort for InMemoryVaultRepository {
    async fn set_sync_running(&self, account_id: &str, base_url: &str) -> AppResult<SyncContext> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| AppError::internal("failed to lock sync contexts for write"))?;

        let context = contexts
            .entry(account_id.to_string())
            .or_insert_with(|| SyncContext::new(account_id.to_string()));
        context.base_url = Some(base_url.to_string());
        context.state = SyncState::Running;
        context.last_error = None;

        Ok(context.clone())
    }

    async fn set_sync_succeeded(
        &self,
        account_id: &str,
        base_url: &str,
        revision_ms: Option<i64>,
        synced_at_ms: i64,
        counts: SyncItemCounts,
    ) -> AppResult<SyncContext> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| AppError::internal("failed to lock sync contexts for write"))?;

        let context = contexts
            .entry(account_id.to_string())
            .or_insert_with(|| SyncContext::new(account_id.to_string()));
        context.base_url = Some(base_url.to_string());
        context.state = SyncState::Succeeded;
        context.last_revision_ms = revision_ms;
        context.last_sync_at_ms = Some(synced_at_ms);
        context.last_error = None;
        context.counts = counts;

        Ok(context.clone())
    }

    async fn set_sync_failed(
        &self,
        account_id: &str,
        base_url: &str,
        error_message: String,
    ) -> AppResult<SyncContext> {
        let mut contexts = self
            .contexts
            .write()
            .map_err(|_| AppError::internal("failed to lock sync contexts for write"))?;

        let context = contexts
            .entry(account_id.to_string())
            .or_insert_with(|| SyncContext::new(account_id.to_string()));
        context.base_url = Some(base_url.to_string());
        context.state = SyncState::Failed;
        context.last_error = Some(error_message);

        Ok(context.clone())
    }

    async fn get_sync_context(&self, account_id: &str) -> AppResult<Option<SyncContext>> {
        let contexts = self
            .contexts
            .read()
            .map_err(|_| AppError::internal("failed to lock sync contexts for read"))?;

        Ok(contexts.get(account_id).cloned())
    }

    async fn begin_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        let base_snapshot = {
            let vaults = self
                .vaults
                .read()
                .map_err(|_| AppError::internal("failed to lock vault snapshots for read"))?;
            vaults.get(account_id).cloned().unwrap_or_default()
        };

        let mut staging = self
            .staging
            .write()
            .map_err(|_| AppError::internal("failed to lock staging snapshots for write"))?;
        if staging.contains_key(account_id) {
            return Err(AppError::validation(format!(
                "sync transaction already started for account_id={account_id}"
            )));
        }

        staging.insert(account_id.to_string(), base_snapshot);
        Ok(())
    }

    async fn commit_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        let staged_snapshot = {
            let mut staging = self
                .staging
                .write()
                .map_err(|_| AppError::internal("failed to lock staging snapshots for write"))?;
            staging.remove(account_id).ok_or_else(|| {
                AppError::validation(format!(
                    "no active sync transaction for account_id={account_id}"
                ))
            })?
        };

        let mut vaults = self
            .vaults
            .write()
            .map_err(|_| AppError::internal("failed to lock vault snapshots for write"))?;
        vaults.insert(account_id.to_string(), staged_snapshot);
        Ok(())
    }

    async fn rollback_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        let mut staging = self
            .staging
            .write()
            .map_err(|_| AppError::internal("failed to lock staging snapshots for write"))?;
        if staging.remove(account_id).is_none() {
            return Err(AppError::validation(format!(
                "no active sync transaction for account_id={account_id}"
            )));
        }
        Ok(())
    }

    async fn upsert_profile(&self, account_id: &str, profile: SyncProfile) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            snapshot.profile = Some(profile);
        })
    }

    async fn upsert_folders(&self, account_id: &str, folders: Vec<SyncFolder>) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            for folder in folders {
                snapshot.folders.insert(folder.id.clone(), folder);
            }
        })
    }

    async fn upsert_collections(
        &self,
        account_id: &str,
        collections: Vec<SyncCollection>,
    ) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            for collection in collections {
                snapshot
                    .collections
                    .insert(collection.id.clone(), collection);
            }
        })
    }

    async fn upsert_policies(&self, account_id: &str, policies: Vec<SyncPolicy>) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            for policy in policies {
                snapshot.policies.insert(policy.id.clone(), policy);
            }
        })
    }

    async fn upsert_ciphers(&self, account_id: &str, ciphers: Vec<SyncCipher>) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            for cipher in ciphers {
                snapshot.ciphers.insert(cipher.id.clone(), cipher);
            }
        })
    }

    async fn upsert_sends(&self, account_id: &str, sends: Vec<SyncSend>) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            for send in sends {
                snapshot.sends.insert(send.id.clone(), send);
            }
        })
    }

    async fn upsert_domains(
        &self,
        account_id: &str,
        domains: Option<SyncDomains>,
    ) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            snapshot.domains = domains;
        })
    }

    async fn upsert_user_decryption(
        &self,
        account_id: &str,
        user_decryption: Option<SyncUserDecryption>,
    ) -> AppResult<()> {
        self.with_staging_mut(account_id, |snapshot| {
            snapshot.user_decryption = user_decryption;
        })
    }
}

impl InMemoryVaultRepository {
    fn with_staging_mut<F>(&self, account_id: &str, f: F) -> AppResult<()>
    where
        F: FnOnce(&mut AccountVaultSnapshot),
    {
        let mut staging = self
            .staging
            .write()
            .map_err(|_| AppError::internal("failed to lock staging snapshots for write"))?;
        let snapshot = staging.get_mut(account_id).ok_or_else(|| {
            AppError::validation(format!(
                "no active sync transaction for account_id={account_id}"
            ))
        })?;
        f(snapshot);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct AccountVaultSnapshot {
    profile: Option<SyncProfile>,
    folders: HashMap<String, SyncFolder>,
    collections: HashMap<String, SyncCollection>,
    policies: HashMap<String, SyncPolicy>,
    ciphers: HashMap<String, SyncCipher>,
    sends: HashMap<String, SyncSend>,
    domains: Option<SyncDomains>,
    user_decryption: Option<SyncUserDecryption>,
}
