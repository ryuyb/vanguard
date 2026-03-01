use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::application::dto::sync::{RevisionDateQuery, SyncOutcome, SyncVaultCommand};
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::domain::sync::{SyncItemCounts, SyncResult, VaultSnapshotMeta};
use crate::support::error::AppError;
use crate::support::result::AppResult;
use tokio::time::{sleep, timeout, Duration};

#[derive(Clone)]
pub struct SyncVaultUseCase {
    remote_vault: Arc<dyn RemoteVaultPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    poll_revision_use_case: Arc<PollRevisionUseCase>,
    sync_policy: SyncPolicy,
}

impl SyncVaultUseCase {
    pub fn new(
        remote_vault: Arc<dyn RemoteVaultPort>,
        vault_repository: Arc<dyn VaultRepositoryPort>,
        poll_revision_use_case: Arc<PollRevisionUseCase>,
        sync_policy: SyncPolicy,
    ) -> Self {
        Self {
            remote_vault,
            vault_repository,
            poll_revision_use_case,
            sync_policy,
        }
    }

    pub async fn execute(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        let sync_endpoint = sync_endpoint(&command.base_url);
        let total_attempts = usize::from(self.sync_policy.max_retries) + 1;
        for attempt in 0..total_attempts {
            match self.execute_with_timeout(command.clone()).await {
                Ok(outcome) => return Ok(outcome),
                Err(error) => {
                    let is_last_attempt = attempt + 1 >= total_attempts;
                    let retryable = is_retryable_error(&error);
                    if is_last_attempt || !retryable {
                        return Err(error);
                    }

                    let backoff_ms = backoff_for_attempt(self.sync_policy.backoff_ms, attempt);
                    log::warn!(
                        target: "vanguard::sync",
                        "sync retry scheduled account_id={} endpoint={} attempt={}/{} backoff_ms={} status={} error_code={} reason={}",
                        command.account_id,
                        sync_endpoint,
                        attempt + 1,
                        total_attempts,
                        backoff_ms,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }

        Err(AppError::internal(
            "sync retry loop terminated unexpectedly without result",
        ))
    }

    async fn execute_with_timeout(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        if self.sync_policy.timeout_ms == 0 {
            return self.execute_once(command).await;
        }

        match timeout(
            Duration::from_millis(self.sync_policy.timeout_ms),
            self.execute_once(command),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(AppError::remote(format!(
                "sync timed out after {}ms",
                self.sync_policy.timeout_ms
            ))),
        }
    }

    async fn execute_once(&self, command: SyncVaultCommand) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.access_token, "access_token")?;

        let previous_context = self
            .vault_repository
            .set_sync_running(&command.account_id, &command.base_url)
            .await?;
        let started_at = Instant::now();
        let sync_endpoint = sync_endpoint(&command.base_url);
        let revision_endpoint = revision_endpoint(&command.base_url);
        log::info!(
            target: "vanguard::sync",
            "sync started account_id={} endpoint={} trigger={:?}",
            command.account_id,
            sync_endpoint,
            command.trigger
        );

        let sync_result = self.remote_vault.sync_vault(command.clone()).await;
        match sync_result {
            Ok(payload) => {
                if let Err(error) = self
                    .persist_sync_payload(&command.account_id, payload.clone())
                    .await
                {
                    log::error!(
                        target: "vanguard::sync",
                        "sync persist failed account_id={} endpoint=local-repository status={} error_code={} message={}",
                        command.account_id,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    let message = error.message();
                    let _ = self
                        .vault_repository
                        .set_sync_failed(&command.account_id, &command.base_url, message)
                        .await;
                    return Err(error);
                }

                let counts = summarize_counts(&payload);
                let revision_ms = match self
                    .poll_revision_use_case
                    .execute(RevisionDateQuery {
                        base_url: command.base_url.clone(),
                        access_token: command.access_token.clone(),
                    })
                    .await
                {
                    Ok(value) => Some(value),
                    Err(error) => {
                        if previous_context.last_sync_at_ms.is_none() {
                            log::error!(
                                target: "vanguard::sync",
                                "revision-date failed on initial sync account_id={} endpoint={} status={} error_code={} message={}",
                                command.account_id,
                                revision_endpoint,
                                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                                error.code(),
                                error
                            );
                            let message = error.message();
                            let _ = self
                                .vault_repository
                                .set_sync_failed(&command.account_id, &command.base_url, message)
                                .await;
                            return Err(error);
                        }

                        log::warn!(
                            target: "vanguard::sync",
                            "revision-date failed account_id={} endpoint={} status={} error_code={} message={} (fallback_to_previous_revision={})",
                            command.account_id,
                            revision_endpoint,
                            error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                            error.code(),
                            error,
                            previous_context.last_revision_ms.is_some()
                        );
                        previous_context.last_revision_ms
                    }
                };
                let synced_at_ms = now_unix_ms()?;
                let revision_changed =
                    is_revision_changed(previous_context.last_revision_ms, revision_ms);
                let duration_ms = clamp_duration_ms(started_at.elapsed());
                let result = SyncResult {
                    duration_ms,
                    item_counts: counts.clone(),
                    revision_changed,
                };

                let context = self
                    .vault_repository
                    .set_sync_succeeded(
                        &command.account_id,
                        &command.base_url,
                        revision_ms,
                        synced_at_ms,
                        counts,
                    )
                    .await?;
                self.vault_repository
                    .save_snapshot_meta(
                        &command.account_id,
                        VaultSnapshotMeta {
                            snapshot_revision_ms: revision_ms,
                            snapshot_synced_at_ms: synced_at_ms,
                            source: command.trigger,
                        },
                    )
                    .await?;
                log::info!(
                    target: "vanguard::sync",
                    "sync finished account_id={} endpoint={} trigger={:?} duration_ms={} revision_changed={} profile_id={} folders={} collections={} policies={} ciphers={} sends={} domains={} user_decryption={}",
                    command.account_id,
                    sync_endpoint,
                    command.trigger,
                    result.duration_ms,
                    result.revision_changed,
                    payload.profile.id,
                    payload.folders.len(),
                    payload.collections.len(),
                    payload.policies.len(),
                    payload.ciphers.len(),
                    payload.sends.len(),
                    payload.domains.is_some(),
                    payload.user_decryption.is_some()
                );
                Ok(SyncOutcome { context, result })
            }
            Err(error) => {
                log::error!(
                    target: "vanguard::sync",
                    "sync request failed account_id={} endpoint={} status={} error_code={} message={}",
                    command.account_id,
                    sync_endpoint,
                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                    error.code(),
                    error
                );
                let message = error.message();
                let _ = self
                    .vault_repository
                    .set_sync_failed(&command.account_id, &command.base_url, message)
                    .await;
                Err(error)
            }
        }
    }

    async fn persist_sync_payload(
        &self,
        account_id: &str,
        payload: crate::application::dto::sync::SyncVaultPayload,
    ) -> AppResult<()> {
        self.vault_repository
            .begin_sync_transaction(account_id)
            .await?;

        if let Err(error) = self.persist_sync_payload_inner(account_id, payload).await {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        if let Err(error) = self
            .vault_repository
            .commit_sync_transaction(account_id)
            .await
        {
            let _ = self
                .vault_repository
                .rollback_sync_transaction(account_id)
                .await;
            return Err(error);
        }

        Ok(())
    }

    async fn persist_sync_payload_inner(
        &self,
        account_id: &str,
        payload: crate::application::dto::sync::SyncVaultPayload,
    ) -> AppResult<()> {
        self.vault_repository
            .upsert_profile(account_id, payload.profile)
            .await?;
        self.vault_repository
            .upsert_folders(account_id, payload.folders)
            .await?;
        self.vault_repository
            .upsert_collections(account_id, payload.collections)
            .await?;
        self.vault_repository
            .upsert_policies(account_id, payload.policies)
            .await?;
        self.vault_repository
            .upsert_ciphers(account_id, payload.ciphers)
            .await?;
        self.vault_repository
            .upsert_sends(account_id, payload.sends)
            .await?;
        self.vault_repository
            .upsert_domains(account_id, payload.domains)
            .await?;
        self.vault_repository
            .upsert_user_decryption(account_id, payload.user_decryption)
            .await?;
        Ok(())
    }
}

fn clamp_count(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}

fn clamp_duration_ms(duration: std::time::Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

fn backoff_for_attempt(base_backoff_ms: u64, attempt: usize) -> u64 {
    let shift = attempt.min(16) as u32;
    let multiplier = 1_u64 << shift;
    base_backoff_ms.saturating_mul(multiplier)
}

fn is_retryable_error(error: &AppError) -> bool {
    !matches!(error, AppError::Validation(_))
}

fn is_revision_changed(previous: Option<i64>, current: Option<i64>) -> bool {
    match (previous, current) {
        (_, None) => false,
        (Some(previous), Some(current)) => previous != current,
        (None, Some(_)) => true,
    }
}

fn summarize_counts(payload: &crate::application::dto::sync::SyncVaultPayload) -> SyncItemCounts {
    SyncItemCounts {
        folders: clamp_count(payload.folders.len()),
        collections: clamp_count(payload.collections.len()),
        policies: clamp_count(payload.policies.len()),
        ciphers: clamp_count(payload.ciphers.len()),
        sends: clamp_count(payload.sends.len()),
    }
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(format!("system clock before unix epoch: {error}")))?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
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
