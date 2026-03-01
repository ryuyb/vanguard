use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::application::dto::sync::{RevisionDateQuery, SyncOutcome, SyncVaultCommand};
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::domain::sync::{SyncContext, SyncTrigger};
use crate::support::error::AppError;
use crate::support::result::AppResult;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct SyncService {
    sync_vault_use_case: Arc<SyncVaultUseCase>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    sync_event_port: Arc<dyn SyncEventPort>,
    poll_revision_use_case: Arc<PollRevisionUseCase>,
    sync_policy: SyncPolicy,
    running_accounts: Arc<Mutex<HashSet<String>>>,
    recent_triggers: Arc<Mutex<HashMap<String, Instant>>>,
    poll_workers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl SyncService {
    pub fn new(
        sync_vault_use_case: Arc<SyncVaultUseCase>,
        vault_repository: Arc<dyn VaultRepositoryPort>,
        sync_event_port: Arc<dyn SyncEventPort>,
        poll_revision_use_case: Arc<PollRevisionUseCase>,
        sync_policy: SyncPolicy,
    ) -> Self {
        Self {
            sync_vault_use_case,
            vault_repository,
            sync_event_port,
            poll_revision_use_case,
            sync_policy,
            running_accounts: Arc::new(Mutex::new(HashSet::new())),
            recent_triggers: Arc::new(Mutex::new(HashMap::new())),
            poll_workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn sync_now(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        let account_id = command.account_id.clone();
        let endpoint = sync_endpoint(&command.base_url);
        let trigger = command.trigger;

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
        let _guard = RunningSlotGuard::new(Arc::clone(&self.running_accounts), account_id.clone());
        self.sync_event_port.emit_sync_started(&account_id);

        match self.sync_vault_use_case.execute(command).await {
            Ok(outcome) => {
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                self.process_sync_auth_error(&account_id, trigger, &error);
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

    pub fn start_revision_polling(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
    ) -> AppResult<()> {
        require_non_empty(&account_id, "account_id")?;
        require_non_empty(&base_url, "base_url")?;
        require_non_empty(&access_token, "access_token")?;

        let mut poll_workers = self
            .poll_workers
            .lock()
            .map_err(|_| AppError::internal("failed to lock sync poll workers"))?;

        if let Some(handle) = poll_workers.remove(&account_id) {
            handle.abort();
            log::info!(
                target: "vanguard::sync",
                "revision polling restarted account_id={}",
                account_id
            );
        }

        let polling_interval_seconds = self.sync_policy.poll_interval_seconds.max(1);
        let service = self.clone();
        let worker_account_id = account_id.clone();
        let worker_base_url = base_url.clone();
        let worker_access_token = access_token.clone();
        let handle = tokio::spawn(async move {
            log::info!(
                target: "vanguard::sync",
                "revision polling started account_id={} endpoint={} interval_seconds={}",
                worker_account_id,
                revision_endpoint(&worker_base_url),
                polling_interval_seconds
            );
            loop {
                sleep(Duration::from_secs(polling_interval_seconds)).await;
                if let Err(error) = service
                    .poll_revision_once(
                        worker_account_id.clone(),
                        worker_base_url.clone(),
                        worker_access_token.clone(),
                        SyncTrigger::Poll,
                    )
                    .await
                {
                    log::warn!(
                        target: "vanguard::sync",
                        "revision polling failed account_id={} endpoint={} status={} error_code={} message={}",
                        worker_account_id,
                        revision_endpoint(&worker_base_url),
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    if is_auth_status_error(&error) {
                        break;
                    }
                }
            }

            service.detach_poll_worker(&worker_account_id);
        });

        poll_workers.insert(account_id, handle);
        Ok(())
    }

    pub async fn check_revision_now(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
        trigger: SyncTrigger,
    ) -> AppResult<()> {
        require_non_empty(&account_id, "account_id")?;
        require_non_empty(&base_url, "base_url")?;
        require_non_empty(&access_token, "access_token")?;
        self.poll_revision_once(account_id, base_url, access_token, trigger)
            .await
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

    async fn poll_revision_once(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
        trigger: SyncTrigger,
    ) -> AppResult<()> {
        let endpoint = revision_endpoint(&base_url);
        let remote_revision_ms = self
            .poll_revision_use_case
            .execute(RevisionDateQuery {
                base_url: base_url.clone(),
                access_token: access_token.clone(),
            })
            .await
            .inspect_err(|error| {
                if let Some(status) = auth_status(error) {
                    if trigger != SyncTrigger::Poll {
                        let _ = self.stop_revision_polling(&account_id);
                    }
                    self.sync_event_port
                        .emit_auth_required(&account_id, status, &error.message());
                }
            })?;

        let local_context = self
            .vault_repository
            .get_sync_context(&account_id)
            .await?
            .unwrap_or_else(|| SyncContext::new(account_id.clone()));

        if local_context.last_revision_ms == Some(remote_revision_ms) {
            return Ok(());
        }

        log::info!(
            target: "vanguard::sync",
            "revision changed, scheduling sync account_id={} endpoint={} trigger={:?} local_revision_ms={} remote_revision_ms={}",
            account_id,
            endpoint,
            trigger,
            local_context
                .last_revision_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| String::from("none")),
            remote_revision_ms
        );

        let command = SyncVaultCommand {
            account_id: account_id.clone(),
            base_url: base_url.clone(),
            access_token,
            exclude_domains: false,
            trigger,
        };

        self.sync_now(command).await?;
        Ok(())
    }

    fn process_sync_auth_error(&self, account_id: &str, trigger: SyncTrigger, error: &AppError) {
        if let Some(status) = auth_status(error) {
            if trigger != SyncTrigger::Poll {
                let _ = self.stop_revision_polling(account_id);
            }
            self.sync_event_port
                .emit_auth_required(account_id, status, &error.message());
        }
    }

    fn stop_revision_polling(&self, account_id: &str) -> AppResult<()> {
        let mut poll_workers = self
            .poll_workers
            .lock()
            .map_err(|_| AppError::internal("failed to lock sync poll workers"))?;
        if let Some(handle) = poll_workers.remove(account_id) {
            handle.abort();
            log::info!(
                target: "vanguard::sync",
                "revision polling stopped account_id={}",
                account_id
            );
        }
        Ok(())
    }

    fn detach_poll_worker(&self, account_id: &str) {
        if let Ok(mut poll_workers) = self.poll_workers.lock() {
            poll_workers.remove(account_id);
        }
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

fn revision_endpoint(base_url: &str) -> String {
    format!(
        "{}/api/accounts/revision-date",
        base_url.trim_end_matches('/')
    )
}

fn auth_status(error: &AppError) -> Option<u16> {
    match error.status() {
        Some(401) => Some(401),
        Some(403) => Some(403),
        _ => None,
    }
}

fn is_auth_status_error(error: &AppError) -> bool {
    auth_status(error).is_some()
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
