use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::application::dto::sync::{SyncOutcome, SyncVaultCommand};
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::domain::sync::SyncContext;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct SyncService {
    sync_vault_use_case: Arc<SyncVaultUseCase>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    sync_event_port: Arc<dyn SyncEventPort>,
    sync_policy: SyncPolicy,
    running_accounts: Arc<Mutex<HashSet<String>>>,
    recent_triggers: Arc<Mutex<HashMap<String, Instant>>>,
}

impl SyncService {
    pub fn new(
        sync_vault_use_case: Arc<SyncVaultUseCase>,
        vault_repository: Arc<dyn VaultRepositoryPort>,
        sync_event_port: Arc<dyn SyncEventPort>,
        sync_policy: SyncPolicy,
    ) -> Self {
        Self {
            sync_vault_use_case,
            vault_repository,
            sync_event_port,
            sync_policy,
            running_accounts: Arc::new(Mutex::new(HashSet::new())),
            recent_triggers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn sync_now(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        let account_id = command.account_id.clone();
        let endpoint = sync_endpoint(&command.base_url);

        if let Err(error) = self.enforce_debounce(&account_id) {
            log::warn!(
                target: "vanguard::sync",
                "sync rejected account_id={} endpoint={} status={} error_code={} message={}",
                account_id,
                endpoint,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            return Err(error);
        }

        if let Err(error) = self.acquire_running_slot(&account_id) {
            log::warn!(
                target: "vanguard::sync",
                "sync rejected account_id={} endpoint={} status={} error_code={} message={}",
                account_id,
                endpoint,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            return Err(error);
        }
        let _guard = RunningSlotGuard::new(
            Arc::clone(&self.running_accounts),
            account_id.clone(),
        );
        self.sync_event_port.emit_sync_started(&account_id);

        match self.sync_vault_use_case.execute(command).await {
            Ok(outcome) => {
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                log::error!(
                    target: "vanguard::sync",
                    "sync failed account_id={} endpoint={} status={} error_code={} message={}",
                    account_id,
                    endpoint,
                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                    error.code(),
                    error
                );
                let payload = error.to_payload();
                self.sync_event_port
                    .emit_sync_failed(&account_id, &payload.code, &payload.message);
                Err(error)
            }
        }
    }

    pub async fn sync_status(&self, account_id: String) -> AppResult<SyncContext> {
        require_non_empty(&account_id, "account_id")?;
        Ok(self
            .vault_repository
            .get_sync_context(&account_id)
            .await?
            .unwrap_or_else(|| SyncContext::new(account_id)))
    }

    fn acquire_running_slot(&self, account_id: &str) -> AppResult<()> {
        let mut running_accounts = self
            .running_accounts
            .lock()
            .map_err(|_| AppError::internal("failed to lock running sync accounts"))?;

        if running_accounts.contains(account_id) {
            return Err(AppError::validation(format!(
                "sync is already running for account_id={account_id}"
            )));
        }

        running_accounts.insert(account_id.to_string());
        Ok(())
    }

    fn enforce_debounce(&self, account_id: &str) -> AppResult<()> {
        if self.sync_policy.debounce_ms == 0 {
            return Ok(());
        }

        let mut recent_triggers = self
            .recent_triggers
            .lock()
            .map_err(|_| AppError::internal("failed to lock recent sync triggers"))?;
        let now = Instant::now();

        if let Some(last_trigger) = recent_triggers.get(account_id) {
            let elapsed_ms = now
                .checked_duration_since(*last_trigger)
                .map(|duration| duration.as_millis() as u64)
                .unwrap_or(0);
            if elapsed_ms < self.sync_policy.debounce_ms {
                return Err(AppError::validation(format!(
                    "sync trigger is debounced for account_id={account_id} (debounce_ms={})",
                    self.sync_policy.debounce_ms
                )));
            }
        }

        recent_triggers.insert(account_id.to_string(), now);
        Ok(())
    }
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::validation(format!("{field} cannot be empty")));
    }
    Ok(())
}

fn sync_endpoint(base_url: &str) -> String {
    format!("{}/api/sync", base_url.trim_end_matches('/'))
}

struct RunningSlotGuard {
    running_accounts: Arc<Mutex<HashSet<String>>>,
    account_id: String,
}

impl RunningSlotGuard {
    fn new(running_accounts: Arc<Mutex<HashSet<String>>>, account_id: String) -> Self {
        Self {
            running_accounts,
            account_id,
        }
    }
}

impl Drop for RunningSlotGuard {
    fn drop(&mut self) {
        if let Ok(mut running_accounts) = self.running_accounts.lock() {
            running_accounts.remove(&self.account_id);
        }
    }
}
