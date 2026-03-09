use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::application::dto::sync::{
    RevisionDateQuery, SyncCipher, SyncFolder, SyncMetricsSummary, SyncOutcome, SyncUserDecryption,
    SyncVaultCommand,
};
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::application::use_cases::sync_vault_use_case::SyncVaultUseCase;
use crate::domain::sync::{SyncContext, SyncItemCounts, SyncResult, SyncTrigger};
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
    sync_metrics_cache: Arc<Mutex<HashMap<String, SyncMetricsAccumulator>>>,
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
            sync_metrics_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn sync_now(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        let account_id = command.account_id.clone();
        let endpoint = sync_endpoint(&command.base_url);
        let trigger = command.trigger;

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

        self.sync_event_port.emit_sync_started(&account_id);
        let started_at = Instant::now();

        match self.sync_vault_use_case.execute(command).await {
            Ok(outcome) => {
                self.record_sync_success(&account_id, &outcome.result);
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                self.record_sync_failure(&account_id, clamp_duration_ms(started_at.elapsed()));
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

    pub async fn sync_cipher_by_id(
        &self,
        command: SyncVaultCommand,
        cipher_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&cipher_id, "cipher_id")?;

        let account_id = command.account_id.clone();
        let endpoint = cipher_endpoint(&command.base_url, &cipher_id);
        let trigger = command.trigger;

        if let Err(error) = self.acquire_running_slot(&account_id) {
            log::warn!(
                target: "vanguard::sync",
                "incremental sync rejected account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                account_id,
                endpoint,
                event_type,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            return Err(error);
        }
        let _guard = RunningSlotGuard::new(Arc::clone(&self.running_accounts), account_id.clone());
        self.sync_event_port.emit_sync_started(&account_id);
        let started_at = Instant::now();

        match self
            .sync_vault_use_case
            .execute_cipher_incremental(command, cipher_id, event_type)
            .await
        {
            Ok(outcome) => {
                self.record_sync_success(&account_id, &outcome.result);
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                self.record_sync_failure(&account_id, clamp_duration_ms(started_at.elapsed()));
                self.process_sync_auth_error(&account_id, trigger, &error);
                log::error!(
                    target: "vanguard::sync",
                    "incremental sync failed account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                    account_id,
                    endpoint,
                    event_type,
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

    pub async fn sync_folder_by_id(
        &self,
        command: SyncVaultCommand,
        folder_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&folder_id, "folder_id")?;

        let account_id = command.account_id.clone();
        let endpoint = folder_endpoint(&command.base_url, &folder_id);
        let trigger = command.trigger;

        if let Err(error) = self.acquire_running_slot(&account_id) {
            log::warn!(
                target: "vanguard::sync",
                "incremental sync rejected account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                account_id,
                endpoint,
                event_type,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            return Err(error);
        }
        let _guard = RunningSlotGuard::new(Arc::clone(&self.running_accounts), account_id.clone());
        self.sync_event_port.emit_sync_started(&account_id);
        let started_at = Instant::now();

        match self
            .sync_vault_use_case
            .execute_folder_incremental(command, folder_id, event_type)
            .await
        {
            Ok(outcome) => {
                self.record_sync_success(&account_id, &outcome.result);
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                self.record_sync_failure(&account_id, clamp_duration_ms(started_at.elapsed()));
                self.process_sync_auth_error(&account_id, trigger, &error);
                log::error!(
                    target: "vanguard::sync",
                    "incremental sync failed account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                    account_id,
                    endpoint,
                    event_type,
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

    pub async fn sync_send_by_id(
        &self,
        command: SyncVaultCommand,
        send_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&send_id, "send_id")?;

        let account_id = command.account_id.clone();
        let endpoint = send_endpoint(&command.base_url, &send_id);
        let trigger = command.trigger;

        if let Err(error) = self.acquire_running_slot(&account_id) {
            log::warn!(
                target: "vanguard::sync",
                "incremental sync rejected account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                account_id,
                endpoint,
                event_type,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            return Err(error);
        }
        let _guard = RunningSlotGuard::new(Arc::clone(&self.running_accounts), account_id.clone());
        self.sync_event_port.emit_sync_started(&account_id);
        let started_at = Instant::now();

        match self
            .sync_vault_use_case
            .execute_send_incremental(command, send_id, event_type)
            .await
        {
            Ok(outcome) => {
                self.record_sync_success(&account_id, &outcome.result);
                self.sync_event_port.emit_sync_succeeded(&outcome.context);
                Ok(outcome)
            }
            Err(error) => {
                self.record_sync_failure(&account_id, clamp_duration_ms(started_at.elapsed()));
                self.process_sync_auth_error(&account_id, trigger, &error);
                log::error!(
                    target: "vanguard::sync",
                    "incremental sync failed account_id={} endpoint={} event_type={} status={} error_code={} message={}",
                    account_id,
                    endpoint,
                    event_type,
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

    pub fn sync_metrics(&self, account_id: String) -> AppResult<SyncMetricsSummary> {
        require_non_empty(&account_id, "account_id")?;
        let cache = self
            .sync_metrics_cache
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock sync metrics cache".to_string(),
            })?;

        let summary = cache
            .get(&account_id)
            .map(|metrics| metrics.to_summary())
            .unwrap_or_else(|| SyncMetricsSummary {
                window_size: self.sync_policy.sync_metrics_window_size.max(1) as u32,
                ..SyncMetricsSummary::default()
            });
        Ok(summary)
    }

    pub async fn sync_status(&self, account_id: String) -> AppResult<SyncContext> {
        require_non_empty(&account_id, "account_id")?;
        Ok(self
            .vault_repository
            .get_sync_context(&account_id)
            .await?
            .unwrap_or_else(|| SyncContext::new(account_id)))
    }

    pub async fn list_live_folders(&self, account_id: String) -> AppResult<Vec<SyncFolder>> {
        require_non_empty(&account_id, "account_id")?;
        self.vault_repository.list_live_folders(&account_id).await
    }

    pub async fn list_live_ciphers(
        &self,
        account_id: String,
        offset: u32,
        limit: u32,
    ) -> AppResult<Vec<SyncCipher>> {
        require_non_empty(&account_id, "account_id")?;
        if limit == 0 {
            return Err(AppError::ValidationFieldError {
                field: "limit".to_string(),
                message: "must be greater than 0".to_string(),
            });
        }
        self.vault_repository
            .list_live_ciphers(&account_id, offset, limit)
            .await
    }

    pub async fn count_live_ciphers(&self, account_id: String) -> AppResult<u32> {
        require_non_empty(&account_id, "account_id")?;
        self.vault_repository.count_live_ciphers(&account_id).await
    }

    pub async fn get_live_cipher(
        &self,
        account_id: String,
        cipher_id: String,
    ) -> AppResult<Option<SyncCipher>> {
        require_non_empty(&account_id, "account_id")?;
        require_non_empty(&cipher_id, "cipher_id")?;
        self.vault_repository
            .get_live_cipher(&account_id, &cipher_id)
            .await
    }

    pub async fn load_live_user_decryption(
        &self,
        account_id: String,
    ) -> AppResult<Option<SyncUserDecryption>> {
        require_non_empty(&account_id, "account_id")?;
        self.vault_repository
            .load_live_user_decryption(&account_id)
            .await
    }

    pub fn start_revision_polling(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
    ) -> AppResult<()> {
        self.start_revision_polling_with_interval(
            account_id,
            base_url,
            access_token,
            self.sync_policy.poll_interval_seconds,
        )
    }

    pub fn start_revision_polling_with_interval(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
        interval_seconds: u64,
    ) -> AppResult<()> {
        require_non_empty(&account_id, "account_id")?;
        require_non_empty(&base_url, "base_url")?;
        require_non_empty(&access_token, "access_token")?;

        let mut poll_workers =
            self.poll_workers
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock sync poll workers".to_string(),
                })?;

        if let Some(handle) = poll_workers.remove(&account_id) {
            handle.abort();
            log::info!(
                target: "vanguard::sync",
                "revision polling restarted account_id={}",
                account_id
            );
        }

        let polling_interval_seconds = interval_seconds.max(1);
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

    pub fn stop_polling_for_account(&self, account_id: &str) -> AppResult<()> {
        require_non_empty(account_id, "account_id")?;
        self.stop_revision_polling(account_id)
    }

    fn acquire_running_slot(&self, account_id: &str) -> AppResult<()> {
        let mut running_accounts =
            self.running_accounts
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock running sync accounts".to_string(),
                })?;

        if running_accounts.contains(account_id) {
            return Err(AppError::ValidationFieldError {
                field: "account_id".to_string(),
                message: format!("sync is already running for account_id={account_id}"),
            });
        }

        running_accounts.insert(account_id.to_string());
        Ok(())
    }

    fn enforce_debounce(&self, account_id: &str) -> AppResult<()> {
        if self.sync_policy.debounce_ms == 0 {
            return Ok(());
        }

        let mut recent_triggers =
            self.recent_triggers
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock recent sync triggers".to_string(),
                })?;
        let now = Instant::now();

        if let Some(last_trigger) = recent_triggers.get(account_id) {
            let elapsed_ms = now
                .checked_duration_since(*last_trigger)
                .map(|duration| duration.as_millis() as u64)
                .unwrap_or(0);
            if elapsed_ms < self.sync_policy.debounce_ms {
                return Err(AppError::ValidationFieldError {
                    field: "account_id".to_string(),
                    message: format!(
                        "sync trigger is debounced for account_id={account_id} (debounce_ms={})",
                        self.sync_policy.debounce_ms
                    ),
                });
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
        let mut poll_workers =
            self.poll_workers
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock sync poll workers".to_string(),
                })?;
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

    fn record_sync_success(&self, account_id: &str, result: &SyncResult) {
        if let Ok(mut cache) = self.sync_metrics_cache.lock() {
            let window_size = self.sync_policy.sync_metrics_window_size.max(1);
            let metrics = cache
                .entry(String::from(account_id))
                .or_insert_with(|| SyncMetricsAccumulator::new(window_size));
            metrics.push(SyncMetricSample {
                success: true,
                duration_ms: result.duration_ms.max(0),
                item_counts: Some(result.item_counts.clone()),
            });
        }
    }

    fn record_sync_failure(&self, account_id: &str, duration_ms: i64) {
        if let Ok(mut cache) = self.sync_metrics_cache.lock() {
            let window_size = self.sync_policy.sync_metrics_window_size.max(1);
            let metrics = cache
                .entry(String::from(account_id))
                .or_insert_with(|| SyncMetricsAccumulator::new(window_size));
            metrics.push(SyncMetricSample {
                success: false,
                duration_ms: duration_ms.max(0),
                item_counts: None,
            });
        }
    }
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationRequired {
            field: field.to_string(),
        });
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

fn cipher_endpoint(base_url: &str, cipher_id: &str) -> String {
    format!(
        "{}/api/ciphers/{}",
        base_url.trim_end_matches('/'),
        cipher_id
    )
}

fn folder_endpoint(base_url: &str, folder_id: &str) -> String {
    format!(
        "{}/api/folders/{}",
        base_url.trim_end_matches('/'),
        folder_id
    )
}

fn send_endpoint(base_url: &str, send_id: &str) -> String {
    format!("{}/api/sends/{}", base_url.trim_end_matches('/'), send_id)
}

fn clamp_duration_ms(duration: std::time::Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
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

#[derive(Debug, Clone)]
struct SyncMetricSample {
    success: bool,
    duration_ms: i64,
    item_counts: Option<SyncItemCounts>,
}

#[derive(Debug, Clone)]
struct SyncMetricsAccumulator {
    window_size: usize,
    samples: VecDeque<SyncMetricSample>,
}

impl SyncMetricsAccumulator {
    fn new(window_size: usize) -> Self {
        Self {
            window_size: window_size.max(1),
            samples: VecDeque::new(),
        }
    }

    fn push(&mut self, sample: SyncMetricSample) {
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    fn to_summary(&self) -> SyncMetricsSummary {
        let sample_count = self.samples.len() as u32;
        let success_count = self.samples.iter().filter(|sample| sample.success).count() as u32;
        let failure_count = sample_count.saturating_sub(success_count);
        let failure_rate = if sample_count == 0 {
            0.0
        } else {
            failure_count as f64 / sample_count as f64
        };

        let last_duration_ms = self.samples.back().map(|sample| sample.duration_ms);
        let average_duration_ms = if self.samples.is_empty() {
            None
        } else {
            let total_duration: i128 = self
                .samples
                .iter()
                .map(|sample| i128::from(sample.duration_ms.max(0)))
                .sum();
            Some((total_duration / i128::from(self.samples.len() as u64)) as i64)
        };

        let last_item_counts = self
            .samples
            .iter()
            .rev()
            .find_map(|sample| sample.item_counts.clone());
        let average_item_counts = average_item_counts(
            self.samples
                .iter()
                .filter_map(|sample| sample.item_counts.as_ref()),
        );

        SyncMetricsSummary {
            window_size: self.window_size as u32,
            sample_count,
            success_count,
            failure_count,
            failure_rate,
            last_duration_ms,
            average_duration_ms,
            last_item_counts,
            average_item_counts,
        }
    }
}

fn average_item_counts<'a>(
    counts: impl Iterator<Item = &'a SyncItemCounts>,
) -> Option<SyncItemCounts> {
    let mut count: u64 = 0;
    let mut folders: u64 = 0;
    let mut collections: u64 = 0;
    let mut policies: u64 = 0;
    let mut ciphers: u64 = 0;
    let mut sends: u64 = 0;

    for entry in counts {
        count = count.saturating_add(1);
        folders = folders.saturating_add(u64::from(entry.folders));
        collections = collections.saturating_add(u64::from(entry.collections));
        policies = policies.saturating_add(u64::from(entry.policies));
        ciphers = ciphers.saturating_add(u64::from(entry.ciphers));
        sends = sends.saturating_add(u64::from(entry.sends));
    }

    if count == 0 {
        return None;
    }

    Some(SyncItemCounts {
        folders: (folders / count).min(u64::from(u32::MAX)) as u32,
        collections: (collections / count).min(u64::from(u32::MAX)) as u32,
        policies: (policies / count).min(u64::from(u32::MAX)) as u32,
        ciphers: (ciphers / count).min(u64::from(u32::MAX)) as u32,
        sends: (sends / count).min(u64::from(u32::MAX)) as u32,
    })
}
