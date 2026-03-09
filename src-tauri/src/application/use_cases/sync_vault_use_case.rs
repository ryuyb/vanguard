use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::application::dto::sync::{
    RevisionDateQuery, SyncCipher, SyncCollection, SyncFolder, SyncOutcome,
    SyncPolicy as SyncVaultPolicy, SyncSend, SyncVaultCommand,
};
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::remote_vault_port::RemoteVaultPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::use_cases::poll_revision_use_case::PollRevisionUseCase;
use crate::domain::sync::{SyncItemCounts, SyncResult, SyncTrigger, VaultSnapshotMeta};
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

        Err(AppError::InternalUnexpected {
            message: "sync retry loop terminated unexpectedly without result".into(),
        })
    }

    pub async fn execute_cipher_incremental(
        &self,
        command: SyncVaultCommand,
        cipher_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.access_token, "access_token")?;
        require_non_empty(&cipher_id, "cipher_id")?;

        let started_at = Instant::now();
        let cipher_endpoint = cipher_endpoint(&command.base_url, &cipher_id);
        let revision_endpoint = revision_endpoint(&command.base_url);
        let previous_context = self
            .vault_repository
            .set_sync_running(&command.account_id, &command.base_url)
            .await?;

        log::info!(
            target: "vanguard::sync",
            "incremental cipher sync started account_id={} endpoint={} trigger={:?} event_type={} cipher_id={}",
            command.account_id,
            cipher_endpoint,
            command.trigger,
            event_type,
            cipher_id
        );

        if let Err(error) = self
            .apply_cipher_incremental_update(&command, &cipher_id, event_type)
            .await
        {
            log::error!(
                target: "vanguard::sync",
                "incremental cipher sync failed account_id={} endpoint={} trigger={:?} event_type={} cipher_id={} status={} error_code={} message={}",
                command.account_id,
                cipher_endpoint,
                command.trigger,
                event_type,
                cipher_id,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            let message = error.message();
            self.mark_sync_error_state(&command.account_id, &command.base_url, &error, message)
                .await;
            return Err(error);
        }

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
                        "revision-date failed on incremental sync account_id={} endpoint={} status={} error_code={} message={}",
                        command.account_id,
                        revision_endpoint,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    let message = error.message();
                    self.mark_sync_error_state(
                        &command.account_id,
                        &command.base_url,
                        &error,
                        message,
                    )
                    .await;
                    return Err(error);
                }

                log::warn!(
                    target: "vanguard::sync",
                    "revision-date failed after incremental sync account_id={} endpoint={} status={} error_code={} message={} (fallback_to_previous_revision={})",
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
        let revision_changed = is_revision_changed(previous_context.last_revision_ms, revision_ms);
        let mut counts = previous_context.counts.clone();
        counts.ciphers = self
            .vault_repository
            .count_live_ciphers(&command.account_id)
            .await?;

        let context = self
            .vault_repository
            .set_sync_succeeded(
                &command.account_id,
                &command.base_url,
                revision_ms,
                synced_at_ms,
                counts.clone(),
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

        let result = SyncResult {
            duration_ms: clamp_duration_ms(started_at.elapsed()),
            item_counts: counts,
            revision_changed,
        };

        log::info!(
            target: "vanguard::sync",
            "incremental cipher sync finished account_id={} endpoint={} trigger={:?} event_type={} cipher_id={} duration_ms={} revision_changed={} ciphers={}",
            command.account_id,
            cipher_endpoint,
            command.trigger,
            event_type,
            cipher_id,
            result.duration_ms,
            result.revision_changed,
            result.item_counts.ciphers
        );

        Ok(SyncOutcome { context, result })
    }

    pub async fn execute_folder_incremental(
        &self,
        command: SyncVaultCommand,
        folder_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.access_token, "access_token")?;
        require_non_empty(&folder_id, "folder_id")?;

        let started_at = Instant::now();
        let folder_endpoint = folder_endpoint(&command.base_url, &folder_id);
        let revision_endpoint = revision_endpoint(&command.base_url);
        let previous_context = self
            .vault_repository
            .set_sync_running(&command.account_id, &command.base_url)
            .await?;

        log::info!(
            target: "vanguard::sync",
            "incremental folder sync started account_id={} endpoint={} trigger={:?} event_type={} folder_id={}",
            command.account_id,
            folder_endpoint,
            command.trigger,
            event_type,
            folder_id
        );

        if let Err(error) = self
            .apply_folder_incremental_update(&command, &folder_id, event_type)
            .await
        {
            log::error!(
                target: "vanguard::sync",
                "incremental folder sync failed account_id={} endpoint={} trigger={:?} event_type={} folder_id={} status={} error_code={} message={}",
                command.account_id,
                folder_endpoint,
                command.trigger,
                event_type,
                folder_id,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            let message = error.message();
            self.mark_sync_error_state(&command.account_id, &command.base_url, &error, message)
                .await;
            return Err(error);
        }

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
                        "revision-date failed on incremental sync account_id={} endpoint={} status={} error_code={} message={}",
                        command.account_id,
                        revision_endpoint,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    let message = error.message();
                    self.mark_sync_error_state(
                        &command.account_id,
                        &command.base_url,
                        &error,
                        message,
                    )
                    .await;
                    return Err(error);
                }

                log::warn!(
                    target: "vanguard::sync",
                    "revision-date failed after incremental sync account_id={} endpoint={} status={} error_code={} message={} (fallback_to_previous_revision={})",
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
        let revision_changed = is_revision_changed(previous_context.last_revision_ms, revision_ms);
        let mut counts = previous_context.counts.clone();
        counts.folders = self
            .vault_repository
            .count_live_folders(&command.account_id)
            .await?;

        let context = self
            .vault_repository
            .set_sync_succeeded(
                &command.account_id,
                &command.base_url,
                revision_ms,
                synced_at_ms,
                counts.clone(),
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

        let result = SyncResult {
            duration_ms: clamp_duration_ms(started_at.elapsed()),
            item_counts: counts,
            revision_changed,
        };

        log::info!(
            target: "vanguard::sync",
            "incremental folder sync finished account_id={} endpoint={} trigger={:?} event_type={} folder_id={} duration_ms={} revision_changed={} folders={}",
            command.account_id,
            folder_endpoint,
            command.trigger,
            event_type,
            folder_id,
            result.duration_ms,
            result.revision_changed,
            result.item_counts.folders
        );

        Ok(SyncOutcome { context, result })
    }

    pub async fn execute_send_incremental(
        &self,
        command: SyncVaultCommand,
        send_id: String,
        event_type: i32,
    ) -> AppResult<SyncOutcome> {
        require_non_empty(&command.account_id, "account_id")?;
        require_non_empty(&command.base_url, "base_url")?;
        require_non_empty(&command.access_token, "access_token")?;
        require_non_empty(&send_id, "send_id")?;

        let started_at = Instant::now();
        let send_endpoint = send_endpoint(&command.base_url, &send_id);
        let revision_endpoint = revision_endpoint(&command.base_url);
        let previous_context = self
            .vault_repository
            .set_sync_running(&command.account_id, &command.base_url)
            .await?;

        log::info!(
            target: "vanguard::sync",
            "incremental send sync started account_id={} endpoint={} trigger={:?} event_type={} send_id={}",
            command.account_id,
            send_endpoint,
            command.trigger,
            event_type,
            send_id
        );

        if let Err(error) = self
            .apply_send_incremental_update(&command, &send_id, event_type)
            .await
        {
            log::error!(
                target: "vanguard::sync",
                "incremental send sync failed account_id={} endpoint={} trigger={:?} event_type={} send_id={} status={} error_code={} message={}",
                command.account_id,
                send_endpoint,
                command.trigger,
                event_type,
                send_id,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            let message = error.message();
            self.mark_sync_error_state(&command.account_id, &command.base_url, &error, message)
                .await;
            return Err(error);
        }

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
                        "revision-date failed on incremental sync account_id={} endpoint={} status={} error_code={} message={}",
                        command.account_id,
                        revision_endpoint,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    let message = error.message();
                    self.mark_sync_error_state(
                        &command.account_id,
                        &command.base_url,
                        &error,
                        message,
                    )
                    .await;
                    return Err(error);
                }

                log::warn!(
                    target: "vanguard::sync",
                    "revision-date failed after incremental sync account_id={} endpoint={} status={} error_code={} message={} (fallback_to_previous_revision={})",
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
        let revision_changed = is_revision_changed(previous_context.last_revision_ms, revision_ms);
        let mut counts = previous_context.counts.clone();
        counts.sends = self
            .vault_repository
            .count_live_sends(&command.account_id)
            .await?;

        let context = self
            .vault_repository
            .set_sync_succeeded(
                &command.account_id,
                &command.base_url,
                revision_ms,
                synced_at_ms,
                counts.clone(),
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

        let result = SyncResult {
            duration_ms: clamp_duration_ms(started_at.elapsed()),
            item_counts: counts,
            revision_changed,
        };

        log::info!(
            target: "vanguard::sync",
            "incremental send sync finished account_id={} endpoint={} trigger={:?} event_type={} send_id={} duration_ms={} revision_changed={} sends={}",
            command.account_id,
            send_endpoint,
            command.trigger,
            event_type,
            send_id,
            result.duration_ms,
            result.revision_changed,
            result.item_counts.sends
        );

        Ok(SyncOutcome { context, result })
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
            Err(_) => Err(AppError::NetworkRemoteError {
                status: 0,
                message: format!("sync timed out after {}ms", self.sync_policy.timeout_ms),
            }),
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
                            self.mark_sync_error_state(
                                &command.account_id,
                                &command.base_url,
                                &error,
                                message,
                            )
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
                let skip_payload_persist = should_skip_payload_persist(
                    command.trigger,
                    previous_context.last_sync_at_ms,
                    previous_context.last_revision_ms,
                    revision_ms,
                );
                let counts = if skip_payload_persist {
                    log::info!(
                        target: "vanguard::sync",
                        "sync payload persistence skipped account_id={} endpoint={} reason=revision_unchanged revision_ms={}",
                        command.account_id,
                        sync_endpoint,
                        revision_ms
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| String::from("none"))
                    );
                    previous_context.counts.clone()
                } else {
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
                        self.mark_sync_error_state(
                            &command.account_id,
                            &command.base_url,
                            &error,
                            message,
                        )
                        .await;
                        return Err(error);
                    }
                    summarize_counts(&payload)
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
                    "sync finished account_id={} endpoint={} trigger={:?} duration_ms={} revision_changed={} profile_id={} folders={} collections={} policies={} ciphers={} sends={} domains={} user_decryption={} persisted_payload={}",
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
                    payload.user_decryption.is_some(),
                    !skip_payload_persist
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
                self.mark_sync_error_state(&command.account_id, &command.base_url, &error, message)
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
        self.upsert_folders_chunked(account_id, payload.folders)
            .await?;
        self.upsert_collections_chunked(account_id, payload.collections)
            .await?;
        self.upsert_policies_chunked(account_id, payload.policies)
            .await?;
        self.upsert_ciphers_chunked(account_id, payload.ciphers)
            .await?;
        self.upsert_sends_chunked(account_id, payload.sends).await?;
        self.vault_repository
            .upsert_domains(account_id, payload.domains)
            .await?;
        self.vault_repository
            .upsert_user_decryption(account_id, payload.user_decryption)
            .await?;
        Ok(())
    }

    async fn upsert_folders_chunked(
        &self,
        account_id: &str,
        folders: Vec<SyncFolder>,
    ) -> AppResult<()> {
        let chunk_size = self.sync_policy.db_write_batch_size.max(1);
        if folders.len() <= chunk_size {
            return self
                .vault_repository
                .upsert_folders(account_id, folders)
                .await;
        }

        log::info!(
            target: "vanguard::sync",
            "chunked folder upsert account_id={} total={} chunk_size={}",
            account_id,
            folders.len(),
            chunk_size
        );

        let mut chunk = Vec::with_capacity(chunk_size);
        for folder in folders {
            chunk.push(folder);
            if chunk.len() >= chunk_size {
                self.vault_repository
                    .upsert_folders(account_id, std::mem::take(&mut chunk))
                    .await?;
            }
        }
        if !chunk.is_empty() {
            self.vault_repository
                .upsert_folders(account_id, chunk)
                .await?;
        }

        Ok(())
    }

    async fn upsert_collections_chunked(
        &self,
        account_id: &str,
        collections: Vec<SyncCollection>,
    ) -> AppResult<()> {
        let chunk_size = self.sync_policy.db_write_batch_size.max(1);
        if collections.len() <= chunk_size {
            return self
                .vault_repository
                .upsert_collections(account_id, collections)
                .await;
        }

        log::info!(
            target: "vanguard::sync",
            "chunked collection upsert account_id={} total={} chunk_size={}",
            account_id,
            collections.len(),
            chunk_size
        );

        let mut chunk = Vec::with_capacity(chunk_size);
        for collection in collections {
            chunk.push(collection);
            if chunk.len() >= chunk_size {
                self.vault_repository
                    .upsert_collections(account_id, std::mem::take(&mut chunk))
                    .await?;
            }
        }
        if !chunk.is_empty() {
            self.vault_repository
                .upsert_collections(account_id, chunk)
                .await?;
        }

        Ok(())
    }

    async fn upsert_policies_chunked(
        &self,
        account_id: &str,
        policies: Vec<SyncVaultPolicy>,
    ) -> AppResult<()> {
        let chunk_size = self.sync_policy.db_write_batch_size.max(1);
        if policies.len() <= chunk_size {
            return self
                .vault_repository
                .upsert_policies(account_id, policies)
                .await;
        }

        log::info!(
            target: "vanguard::sync",
            "chunked policy upsert account_id={} total={} chunk_size={}",
            account_id,
            policies.len(),
            chunk_size
        );

        let mut chunk = Vec::with_capacity(chunk_size);
        for policy in policies {
            chunk.push(policy);
            if chunk.len() >= chunk_size {
                self.vault_repository
                    .upsert_policies(account_id, std::mem::take(&mut chunk))
                    .await?;
            }
        }
        if !chunk.is_empty() {
            self.vault_repository
                .upsert_policies(account_id, chunk)
                .await?;
        }

        Ok(())
    }

    async fn upsert_ciphers_chunked(
        &self,
        account_id: &str,
        ciphers: Vec<SyncCipher>,
    ) -> AppResult<()> {
        let chunk_size = self.sync_policy.db_write_batch_size.max(1);
        if ciphers.len() <= chunk_size {
            return self
                .vault_repository
                .upsert_ciphers(account_id, ciphers)
                .await;
        }

        log::info!(
            target: "vanguard::sync",
            "chunked cipher upsert account_id={} total={} chunk_size={}",
            account_id,
            ciphers.len(),
            chunk_size
        );

        let mut chunk = Vec::with_capacity(chunk_size);
        for cipher in ciphers {
            chunk.push(cipher);
            if chunk.len() >= chunk_size {
                self.vault_repository
                    .upsert_ciphers(account_id, std::mem::take(&mut chunk))
                    .await?;
            }
        }
        if !chunk.is_empty() {
            self.vault_repository
                .upsert_ciphers(account_id, chunk)
                .await?;
        }

        Ok(())
    }

    async fn upsert_sends_chunked(&self, account_id: &str, sends: Vec<SyncSend>) -> AppResult<()> {
        let chunk_size = self.sync_policy.db_write_batch_size.max(1);
        if sends.len() <= chunk_size {
            return self.vault_repository.upsert_sends(account_id, sends).await;
        }

        log::info!(
            target: "vanguard::sync",
            "chunked send upsert account_id={} total={} chunk_size={}",
            account_id,
            sends.len(),
            chunk_size
        );

        let mut chunk = Vec::with_capacity(chunk_size);
        for send in sends {
            chunk.push(send);
            if chunk.len() >= chunk_size {
                self.vault_repository
                    .upsert_sends(account_id, std::mem::take(&mut chunk))
                    .await?;
            }
        }
        if !chunk.is_empty() {
            self.vault_repository
                .upsert_sends(account_id, chunk)
                .await?;
        }

        Ok(())
    }

    async fn mark_sync_error_state(
        &self,
        account_id: &str,
        base_url: &str,
        error: &AppError,
        message: String,
    ) {
        let update_result = if is_server_error_status(error) {
            self.vault_repository
                .set_sync_degraded(account_id, base_url, message)
                .await
        } else {
            self.vault_repository
                .set_sync_failed(account_id, base_url, message)
                .await
        };

        if let Err(update_error) = update_result {
            log::warn!(
                target: "vanguard::sync",
                "failed to update sync error state account_id={} endpoint=local-repository status={} error_code={} message={}",
                account_id,
                update_error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                update_error.code(),
                update_error
            );
        }
    }

    async fn apply_cipher_incremental_update(
        &self,
        command: &SyncVaultCommand,
        cipher_id: &str,
        event_type: i32,
    ) -> AppResult<()> {
        if event_type == 2 {
            self.vault_repository
                .delete_cipher_live(&command.account_id, cipher_id)
                .await?;
            return Ok(());
        }

        let cipher = self
            .remote_vault
            .get_cipher(command.clone(), String::from(cipher_id))
            .await?;
        self.persist_incremental_cipher(&command.account_id, cipher)
            .await
    }

    async fn persist_incremental_cipher(
        &self,
        account_id: &str,
        cipher: SyncCipher,
    ) -> AppResult<()> {
        if cipher.deleted_date.is_some() {
            self.vault_repository
                .delete_cipher_live(account_id, &cipher.id)
                .await?;
            return Ok(());
        }

        self.vault_repository
            .upsert_cipher_live(account_id, cipher)
            .await
    }

    async fn apply_folder_incremental_update(
        &self,
        command: &SyncVaultCommand,
        folder_id: &str,
        event_type: i32,
    ) -> AppResult<()> {
        if event_type == 8 {
            self.vault_repository
                .delete_folder_live(&command.account_id, folder_id)
                .await?;
            return Ok(());
        }

        let folder = self
            .remote_vault
            .get_folder(command.clone(), String::from(folder_id))
            .await?;
        self.persist_incremental_folder(&command.account_id, folder)
            .await
    }

    async fn persist_incremental_folder(
        &self,
        account_id: &str,
        folder: SyncFolder,
    ) -> AppResult<()> {
        self.vault_repository
            .upsert_folder_live(account_id, folder)
            .await
    }

    async fn apply_send_incremental_update(
        &self,
        command: &SyncVaultCommand,
        send_id: &str,
        event_type: i32,
    ) -> AppResult<()> {
        if event_type == 14 {
            self.vault_repository
                .delete_send_live(&command.account_id, send_id)
                .await?;
            return Ok(());
        }

        let send = self
            .remote_vault
            .get_send(command.clone(), String::from(send_id))
            .await?;
        self.persist_incremental_send(&command.account_id, send)
            .await
    }

    async fn persist_incremental_send(&self, account_id: &str, send: SyncSend) -> AppResult<()> {
        if send.deletion_date.is_some() {
            self.vault_repository
                .delete_send_live(account_id, &send.id)
                .await?;
            return Ok(());
        }

        self.vault_repository
            .upsert_send_live(account_id, send)
            .await
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
    if matches!(
        error,
        AppError::ValidationFieldError { .. }
            | AppError::ValidationFormatError { .. }
            | AppError::ValidationRequired { .. }
    ) {
        return false;
    }

    !matches!(error.status(), Some(401 | 403))
}

fn is_server_error_status(error: &AppError) -> bool {
    matches!(error.status(), Some(status) if (500..=599).contains(&status))
}

fn is_revision_changed(previous: Option<i64>, current: Option<i64>) -> bool {
    match (previous, current) {
        (_, None) => false,
        (Some(previous), Some(current)) => previous != current,
        (None, Some(_)) => true,
    }
}

fn should_skip_payload_persist(
    trigger: SyncTrigger,
    last_sync_at_ms: Option<i64>,
    previous_revision_ms: Option<i64>,
    current_revision_ms: Option<i64>,
) -> bool {
    if matches!(trigger, SyncTrigger::Manual) {
        return false;
    }

    if last_sync_at_ms.is_none() {
        return false;
    }

    match (previous_revision_ms, current_revision_ms) {
        (Some(previous), Some(current)) => previous == current,
        _ => false,
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
        .map_err(|error| AppError::InternalUnexpected {
            message: format!("system clock before unix epoch: {error}"),
        })?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!("{field} cannot be empty"),
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

#[cfg(test)]
mod tests {
    use super::{is_retryable_error, should_skip_payload_persist};
    use crate::domain::sync::SyncTrigger;
    use crate::support::error::AppError;

    #[test]
    fn payload_persist_is_skipped_when_revision_unchanged_and_synced_before() {
        assert!(should_skip_payload_persist(
            SyncTrigger::Poll,
            Some(1_700_000_000_000),
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn payload_persist_is_not_skipped_for_manual_sync_when_revision_unchanged() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Manual,
            Some(1_700_000_000_000),
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn payload_persist_is_not_skipped_on_first_sync() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Poll,
            None,
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn payload_persist_is_not_skipped_when_revision_unknown() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Poll,
            Some(1_700_000_000_000),
            Some(42),
            None
        ));
        assert!(!should_skip_payload_persist(
            SyncTrigger::Poll,
            Some(1_700_000_000_000),
            None,
            Some(42)
        ));
    }

    #[test]
    fn retryable_error_classifies_network_jitter_as_retryable() {
        assert!(is_retryable_error(&AppError::NetworkRemoteError {
            status: 0,
            message: "temporary network jitter".to_string(),
        }));
        assert!(is_retryable_error(&AppError::NetworkRemoteError {
            status: 503,
            message: "service unavailable".to_string()
        }));
    }

    #[test]
    fn retryable_error_does_not_retry_auth_or_validation() {
        assert!(!is_retryable_error(&AppError::NetworkRemoteError {
            status: 401,
            message: "unauthorized".to_string()
        }));
        assert!(!is_retryable_error(&AppError::NetworkRemoteError {
            status: 403,
            message: "forbidden".to_string()
        }));
        assert!(!is_retryable_error(&AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid request".to_string(),
        }));
    }
}
