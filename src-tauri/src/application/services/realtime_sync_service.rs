use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};

use crate::application::dto::notification::{NotificationConnectCommand, NotificationEvent};
use crate::application::dto::sync::SyncVaultCommand;
use crate::application::policy::sync_policy::SyncPolicy;
use crate::application::ports::notification_port::NotificationPort;
use crate::application::ports::sync_event_port::SyncEventPort;
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::application::services::sync_service::SyncService;
use crate::application::use_cases::fetch_cipher_use_case::FetchCipherUseCase;
use crate::domain::sync::{PushType, SyncTrigger, WsStatus};
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct RealtimeSyncService {
    notification_port: Arc<dyn NotificationPort>,
    vault_repository: Arc<dyn VaultRepositoryPort>,
    sync_event_port: Arc<dyn SyncEventPort>,
    sync_service: Arc<SyncService>,
    fetch_cipher_use_case: Arc<FetchCipherUseCase>,
    sync_policy: SyncPolicy,
    device_identifier: String,
    workers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl RealtimeSyncService {
    pub fn new(
        notification_port: Arc<dyn NotificationPort>,
        vault_repository: Arc<dyn VaultRepositoryPort>,
        sync_event_port: Arc<dyn SyncEventPort>,
        sync_service: Arc<SyncService>,
        fetch_cipher_use_case: Arc<FetchCipherUseCase>,
        sync_policy: SyncPolicy,
        device_identifier: String,
    ) -> Self {
        Self {
            notification_port,
            vault_repository,
            sync_event_port,
            sync_service,
            fetch_cipher_use_case,
            sync_policy,
            device_identifier,
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_for_account(
        &self,
        account_id: String,
        base_url: String,
        access_token: String,
    ) -> AppResult<()> {
        require_non_empty(&account_id, "account_id")?;
        require_non_empty(&base_url, "base_url")?;
        require_non_empty(&access_token, "access_token")?;

        self.stop_existing_worker(&account_id)?;
        if let Err(error) = self.notification_port.disconnect(&account_id).await {
            log::debug!(
                target: "vanguard::sync::ws",
                "failed to disconnect stale websocket stream account_id={} message={}",
                account_id,
                error
            );
        }

        self.vault_repository
            .set_ws_status(&account_id, WsStatus::Unknown)
            .await?;

        let service = self.clone();
        let worker_account_id = account_id.clone();
        let worker_base_url = base_url.clone();
        let worker_access_token = access_token.clone();
        let handle = tokio::spawn(async move {
            service
                .run_worker(worker_account_id, worker_base_url, worker_access_token)
                .await;
        });

        let mut workers = self
            .workers
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock realtime sync workers".to_string(),
            })?;
        workers.insert(account_id, handle);
        Ok(())
    }

    pub async fn stop_for_account(&self, account_id: &str) -> AppResult<()> {
        require_non_empty(account_id, "account_id")?;
        self.stop_existing_worker(account_id)?;

        if let Err(error) = self.notification_port.disconnect(account_id).await {
            log::debug!(
                target: "vanguard::sync::ws",
                "failed to close websocket stream account_id={} message={}",
                account_id,
                error
            );
        }

        self.vault_repository
            .set_ws_status(account_id, WsStatus::Disconnected)
            .await?;
        Ok(())
    }

    fn stop_existing_worker(&self, account_id: &str) -> AppResult<()> {
        let mut workers = self
            .workers
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock realtime sync workers".to_string(),
            })?;
        if let Some(handle) = workers.remove(account_id) {
            handle.abort();
            log::info!(
                target: "vanguard::sync::ws",
                "realtime sync worker stopped account_id={}",
                account_id
            );
        }
        Ok(())
    }

    fn detach_worker(&self, account_id: &str) {
        if let Ok(mut workers) = self.workers.lock() {
            workers.remove(account_id);
        }
    }

    async fn run_worker(&self, account_id: String, base_url: String, access_token: String) {
        let ws_endpoint = websocket_endpoint(&base_url);
        let mut reconnect_attempt = 0usize;

        log::info!(
            target: "vanguard::sync::ws",
            "realtime sync worker started account_id={} endpoint={}",
            account_id,
            ws_endpoint
        );

        loop {
            log::info!(
                target: "vanguard::sync::ws",
                "connecting websocket account_id={} endpoint={} attempt={}",
                account_id,
                ws_endpoint,
                reconnect_attempt + 1
            );

            match self
                .notification_port
                .connect(NotificationConnectCommand {
                    account_id: account_id.clone(),
                    base_url: base_url.clone(),
                    access_token: access_token.clone(),
                    queue_limit: self.sync_policy.ws_event_queue_limit,
                    connect_timeout_ms: self.sync_policy.ws_connect_timeout_ms,
                    handshake_timeout_ms: self.sync_policy.ws_handshake_timeout_ms,
                    shutdown_timeout_ms: self.sync_policy.ws_shutdown_timeout_ms,
                })
                .await
            {
                Ok(()) => {
                    reconnect_attempt = 0;
                    self.update_ws_status(&account_id, WsStatus::Connected)
                        .await;
                    log::info!(
                        target: "vanguard::sync::ws",
                        "websocket connected account_id={} endpoint={}",
                        account_id,
                        ws_endpoint
                    );
                    self.set_polling_interval(
                        &account_id,
                        &base_url,
                        &access_token,
                        self.sync_policy.ws_online_poll_interval_seconds,
                    );

                    if let Err(error) = self
                        .sync_service
                        .check_revision_now(
                            account_id.clone(),
                            base_url.clone(),
                            access_token.clone(),
                            SyncTrigger::WebSocket,
                        )
                        .await
                    {
                        log::warn!(
                            target: "vanguard::sync::ws",
                            "post-connect revision check failed account_id={} status={} error_code={} message={}",
                            account_id,
                            error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                            error.code(),
                            error
                        );
                        if self.handle_auth_error(&account_id, &error) {
                            break;
                        }
                    }

                    let mut should_reconnect = true;
                    loop {
                        match self.notification_port.next_event(&account_id).await {
                            Ok(Some(event)) => {
                                let outcome = self
                                    .process_notification_event(
                                        &account_id,
                                        &base_url,
                                        &access_token,
                                        event,
                                    )
                                    .await;
                                if matches!(outcome, EventOutcome::Stop) {
                                    should_reconnect = false;
                                    break;
                                }
                            }
                            Ok(None) => {
                                log::warn!(
                                    target: "vanguard::sync::ws",
                                    "websocket stream closed account_id={} endpoint={}",
                                    account_id,
                                    ws_endpoint
                                );
                                break;
                            }
                            Err(error) => {
                                log::warn!(
                                    target: "vanguard::sync::ws",
                                    "websocket receive failed account_id={} endpoint={} status={} error_code={} message={}",
                                    account_id,
                                    ws_endpoint,
                                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                                    error.code(),
                                    error
                                );
                                if self.handle_auth_error(&account_id, &error) {
                                    should_reconnect = false;
                                }
                                break;
                            }
                        }
                    }

                    if let Err(error) = self.notification_port.disconnect(&account_id).await {
                        log::debug!(
                            target: "vanguard::sync::ws",
                            "websocket disconnect cleanup failed account_id={} message={}",
                            account_id,
                            error
                        );
                    }
                    self.update_ws_status(&account_id, WsStatus::Disconnected)
                        .await;

                    if !should_reconnect {
                        break;
                    }
                    self.set_polling_interval(
                        &account_id,
                        &base_url,
                        &access_token,
                        self.sync_policy.ws_offline_poll_interval_seconds,
                    );
                }
                Err(error) => {
                    log::warn!(
                        target: "vanguard::sync::ws",
                        "websocket connect failed account_id={} endpoint={} status={} error_code={} message={}",
                        account_id,
                        ws_endpoint,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    self.update_ws_status(&account_id, WsStatus::Disconnected)
                        .await;

                    if self.handle_auth_error(&account_id, &error) || !is_retryable_ws_error(&error)
                    {
                        break;
                    }
                    self.set_polling_interval(
                        &account_id,
                        &base_url,
                        &access_token,
                        self.sync_policy.ws_offline_poll_interval_seconds,
                    );
                }
            }

            reconnect_attempt = reconnect_attempt.saturating_add(1);
            let delay_ms = reconnect_delay_ms(
                self.sync_policy.ws_reconnect_base_ms,
                self.sync_policy.ws_reconnect_max_ms,
                reconnect_attempt,
            );
            sleep(Duration::from_millis(delay_ms)).await;
        }

        self.update_ws_status(&account_id, WsStatus::Disconnected)
            .await;
        self.detach_worker(&account_id);
        log::info!(
            target: "vanguard::sync::ws",
            "realtime sync worker exited account_id={}",
            account_id
        );
    }

    async fn process_notification_event(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        event: NotificationEvent,
    ) -> EventOutcome {
        if event
            .context_id
            .as_deref()
            .map(|value| value == self.device_identifier)
            .unwrap_or(false)
        {
            log::debug!(
                target: "vanguard::sync::ws",
                "ignored websocket loopback event account_id={} type={} context_id={}",
                account_id,
                event.event_type,
                event.context_id.as_deref().unwrap_or("none")
            );
            return EventOutcome::Continue;
        }

        log::debug!(
            target: "vanguard::sync::ws",
            "received websocket event account_id={} type={} has_payload={} context_id={} received_at_ms={}",
            account_id,
            event.event_type,
            event.payload.is_some(),
            event.context_id.as_deref().unwrap_or("none"),
            event.received_at_ms
        );

        let Some(push_type) = PushType::from_i32(event.event_type) else {
            log::debug!(
                target: "vanguard::sync::ws",
                "ignored unknown websocket event type account_id={} type={}",
                account_id,
                event.event_type
            );
            return EventOutcome::Continue;
        };

        if push_type.is_logout() {
            return self.handle_logout_event(account_id);
        }

        if push_type.is_incremental_cipher_event() {
            let cipher_id = extract_payload_id(event.payload.as_ref());
            return self
                .trigger_websocket_incremental_cipher_sync(
                    account_id,
                    base_url,
                    access_token,
                    push_type,
                    cipher_id.as_deref(),
                )
                .await;
        }

        if push_type.is_incremental_folder_event() {
            let folder_id = extract_payload_id(event.payload.as_ref());
            return self
                .trigger_websocket_incremental_folder_sync(
                    account_id,
                    base_url,
                    access_token,
                    push_type,
                    folder_id.as_deref(),
                )
                .await;
        }

        if push_type.is_incremental_send_event() {
            let send_id = extract_payload_id(event.payload.as_ref());
            return self
                .trigger_websocket_incremental_send_sync(
                    account_id,
                    base_url,
                    access_token,
                    push_type,
                    send_id.as_deref(),
                )
                .await;
        }

        if push_type.is_sync_event() {
            let burst_outcome = self.collect_sync_burst(account_id).await;
            if matches!(burst_outcome, BurstOutcome::Stop) {
                return EventOutcome::Stop;
            }

            let merged_count = match burst_outcome {
                BurstOutcome::Continue { merged_count } => merged_count,
                BurstOutcome::Stop => 0,
            };
            if merged_count > 0 {
                log::debug!(
                    target: "vanguard::sync::ws",
                    "merged websocket sync events account_id={} merged_count={}",
                    account_id,
                    merged_count
                );
            }

            return self
                .trigger_websocket_sync(account_id, base_url, access_token, push_type)
                .await;
        }

        log::debug!(
            target: "vanguard::sync::ws",
            "ignored websocket event account_id={} type={:?}",
            account_id,
            push_type
        );
        EventOutcome::Continue
    }

    async fn collect_sync_burst(&self, account_id: &str) -> BurstOutcome {
        if self.sync_policy.ws_message_debounce_ms == 0 {
            return BurstOutcome::Continue { merged_count: 0 };
        }

        let wait_duration = Duration::from_millis(self.sync_policy.ws_message_debounce_ms);
        let mut merged_count: usize = 0;

        loop {
            let next = timeout(wait_duration, self.notification_port.next_event(account_id)).await;
            match next {
                Err(_) => break,
                Ok(Ok(Some(event))) => {
                    if event
                        .context_id
                        .as_deref()
                        .map(|value| value == self.device_identifier)
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    if let Some(push_type) = PushType::from_i32(event.event_type) {
                        if push_type.is_logout() {
                            return self.handle_logout_event(account_id).into();
                        }
                        if push_type.is_sync_event() {
                            merged_count = merged_count.saturating_add(1);
                            continue;
                        }
                    }

                    log::debug!(
                        target: "vanguard::sync::ws",
                        "ignored websocket event during debounce account_id={} type={}",
                        account_id,
                        event.event_type
                    );
                }
                Ok(Ok(None)) => break,
                Ok(Err(error)) => {
                    log::warn!(
                        target: "vanguard::sync::ws",
                        "websocket receive failed during debounce account_id={} status={} error_code={} message={}",
                        account_id,
                        error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                        error.code(),
                        error
                    );
                    if self.handle_auth_error(account_id, &error) {
                        return BurstOutcome::Stop;
                    }
                    break;
                }
            }
        }

        BurstOutcome::Continue { merged_count }
    }

    async fn trigger_websocket_sync(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        push_type: PushType,
    ) -> EventOutcome {
        let command = SyncVaultCommand {
            account_id: String::from(account_id),
            base_url: String::from(base_url),
            access_token: String::from(access_token),
            exclude_domains: false,
            trigger: SyncTrigger::WebSocket,
        };
        if let Err(error) = self.sync_service.sync_now(command).await {
            log::warn!(
                target: "vanguard::sync::ws",
                "websocket-triggered sync failed account_id={} type={:?} status={} error_code={} message={}",
                account_id,
                push_type,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
            if is_auth_status_error(&error) {
                return EventOutcome::Stop;
            }
        }
        EventOutcome::Continue
    }

    async fn trigger_websocket_incremental_cipher_sync(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        push_type: PushType,
        cipher_id: Option<&str>,
    ) -> EventOutcome {
        let Some(cipher_id) = cipher_id else {
            log::debug!(
                target: "vanguard::sync::ws",
                "incremental websocket sync fallback to full sync due to missing cipher id account_id={} type={:?}",
                account_id,
                push_type
            );
            return self
                .trigger_websocket_sync(account_id, base_url, access_token, push_type)
                .await;
        };

        if push_type == PushType::SyncCipherDelete {
            match self
                .vault_repository
                .delete_cipher(account_id, cipher_id)
                .await
            {
                Ok(_) => {
                    self.sync_event_port
                        .emit_cipher_deleted(account_id, cipher_id);
                    log::debug!(
                        target: "vanguard::sync::ws",
                        "cipher deleted via websocket account_id={} cipher_id={}",
                        account_id,
                        cipher_id
                    );
                    return EventOutcome::Continue;
                }
                Err(error) => {
                    log::warn!(
                        target: "vanguard::sync::ws",
                        "cipher delete failed account_id={} cipher_id={} error={}",
                        account_id,
                        cipher_id,
                        error
                    );
                    return EventOutcome::Continue;
                }
            }
        }

        match self
            .fetch_cipher_use_case
            .execute(
                String::from(account_id),
                String::from(base_url),
                String::from(access_token),
                String::from(cipher_id),
            )
            .await
        {
            Ok(_) => {
                log::debug!(
                    target: "vanguard::sync::ws",
                    "cipher synced via websocket account_id={} cipher_id={} type={:?}",
                    account_id,
                    cipher_id,
                    push_type
                );
                EventOutcome::Continue
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::sync::ws",
                    "incremental websocket sync failed and will fallback to full sync account_id={} type={:?} cipher_id={} status={} error_code={} message={}",
                    account_id,
                    push_type,
                    cipher_id,
                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                    error.code(),
                    error
                );
                if is_auth_status_error(&error) {
                    return EventOutcome::Stop;
                }
                self.trigger_websocket_sync(account_id, base_url, access_token, push_type)
                    .await
            }
        }
    }

    async fn trigger_websocket_incremental_folder_sync(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        push_type: PushType,
        folder_id: Option<&str>,
    ) -> EventOutcome {
        let Some(folder_id) = folder_id else {
            log::debug!(
                target: "vanguard::sync::ws",
                "incremental websocket sync fallback to full sync due to missing folder id account_id={} type={:?}",
                account_id,
                push_type
            );
            return self
                .trigger_websocket_sync(account_id, base_url, access_token, push_type)
                .await;
        };

        let command = SyncVaultCommand {
            account_id: String::from(account_id),
            base_url: String::from(base_url),
            access_token: String::from(access_token),
            exclude_domains: false,
            trigger: SyncTrigger::WebSocket,
        };

        match self
            .sync_service
            .sync_folder_by_id(command, String::from(folder_id), push_type)
            .await
        {
            Ok(_) => EventOutcome::Continue,
            Err(error) => {
                log::warn!(
                    target: "vanguard::sync::ws",
                    "incremental websocket sync failed and will fallback to full sync account_id={} type={:?} folder_id={} status={} error_code={} message={}",
                    account_id,
                    push_type,
                    folder_id,
                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                    error.code(),
                    error
                );
                if is_auth_status_error(&error) {
                    return EventOutcome::Stop;
                }
                self.trigger_websocket_sync(account_id, base_url, access_token, push_type)
                    .await
            }
        }
    }

    async fn trigger_websocket_incremental_send_sync(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        push_type: PushType,
        send_id: Option<&str>,
    ) -> EventOutcome {
        let Some(send_id) = send_id else {
            log::debug!(
                target: "vanguard::sync::ws",
                "incremental websocket sync fallback to full sync due to missing send id account_id={} type={:?}",
                account_id,
                push_type
            );
            return self
                .trigger_websocket_sync(account_id, base_url, access_token, push_type)
                .await;
        };

        let command = SyncVaultCommand {
            account_id: String::from(account_id),
            base_url: String::from(base_url),
            access_token: String::from(access_token),
            exclude_domains: false,
            trigger: SyncTrigger::WebSocket,
        };

        match self
            .sync_service
            .sync_send_by_id(command, String::from(send_id), push_type)
            .await
        {
            Ok(_) => EventOutcome::Continue,
            Err(error) => {
                log::warn!(
                    target: "vanguard::sync::ws",
                    "incremental websocket sync failed and will fallback to full sync account_id={} type={:?} send_id={} status={} error_code={} message={}",
                    account_id,
                    push_type,
                    send_id,
                    error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                    error.code(),
                    error
                );
                if is_auth_status_error(&error) {
                    return EventOutcome::Stop;
                }
                self.trigger_websocket_sync(account_id, base_url, access_token, push_type)
                    .await
            }
        }
    }

    fn handle_logout_event(&self, account_id: &str) -> EventOutcome {
        let _ = self.sync_service.stop_polling_for_account(account_id);
        self.sync_event_port
            .emit_logged_out(account_id, "received websocket logout notification");
        self.sync_event_port.emit_auth_required(
            account_id,
            401,
            "received websocket logout notification",
        );
        EventOutcome::Stop
    }

    async fn update_ws_status(&self, account_id: &str, status: WsStatus) {
        if let Err(error) = self
            .vault_repository
            .set_ws_status(account_id, status)
            .await
        {
            log::warn!(
                target: "vanguard::sync::ws",
                "failed to persist websocket status account_id={} status={:?} message={}",
                account_id,
                status,
                error
            );
        }
    }

    fn handle_auth_error(&self, account_id: &str, error: &AppError) -> bool {
        if let Some(status) = auth_status(error) {
            let _ = self.sync_service.stop_polling_for_account(account_id);
            self.sync_event_port
                .emit_auth_required(account_id, status, &error.message());
            return true;
        }
        false
    }

    fn set_polling_interval(
        &self,
        account_id: &str,
        base_url: &str,
        access_token: &str,
        interval_seconds: u64,
    ) {
        if let Err(error) = self.sync_service.start_revision_polling_with_interval(
            String::from(account_id),
            String::from(base_url),
            String::from(access_token),
            interval_seconds,
        ) {
            log::warn!(
                target: "vanguard::sync::ws",
                "failed to adjust polling interval account_id={} interval_seconds={} status={} error_code={} message={}",
                account_id,
                interval_seconds,
                error.status().map(|value| value.to_string()).unwrap_or_else(|| String::from("n/a")),
                error.code(),
                error
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventOutcome {
    Continue,
    Stop,
}

enum BurstOutcome {
    Continue { merged_count: usize },
    Stop,
}

impl From<EventOutcome> for BurstOutcome {
    fn from(value: EventOutcome) -> Self {
        match value {
            EventOutcome::Continue => Self::Continue { merged_count: 0 },
            EventOutcome::Stop => Self::Stop,
        }
    }
}

fn reconnect_delay_ms(base_ms: u64, max_ms: u64, attempt: usize) -> u64 {
    let base = base_ms.max(1);
    let ceiling = max_ms.max(base);
    let shift = attempt.min(16) as u32;
    let exponential = base.saturating_mul(1_u64 << shift).min(ceiling);
    let jitter = ((attempt as u64) % 7).saturating_mul(53);
    exponential.saturating_add(jitter).min(ceiling)
}

fn require_non_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::ValidationRequired {
            field: field.to_string(),
        });
    }
    Ok(())
}

fn websocket_endpoint(base_url: &str) -> String {
    format!("{}/notifications/hub", base_url.trim_end_matches('/'))
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

fn is_retryable_ws_error(error: &AppError) -> bool {
    if is_auth_status_error(error) {
        return false;
    }

    !matches!(
        error,
        AppError::ValidationFieldError { .. }
            | AppError::ValidationFormatError { .. }
            | AppError::ValidationRequired { .. }
    )
}

fn extract_payload_id(payload: Option<&JsonValue>) -> Option<String> {
    payload.and_then(|value| extract_payload_id_from_value(value, 0))
}

fn extract_payload_id_from_value(value: &JsonValue, depth: usize) -> Option<String> {
    if depth > 4 {
        return None;
    }

    match value {
        JsonValue::String(value) => normalize_cipher_id(value),
        JsonValue::Object(map) => {
            for key in ["Id", "id", "CipherId", "cipherId", "CipherID", "cipherID"] {
                if let Some(JsonValue::String(value)) = map.get(key) {
                    if let Some(value) = normalize_cipher_id(value) {
                        return Some(value);
                    }
                }
            }

            for key in ["Cipher", "cipher", "Data", "data", "Payload", "payload"] {
                if let Some(inner) = map.get(key) {
                    if let Some(id) = extract_payload_id_from_value(inner, depth + 1) {
                        return Some(id);
                    }
                }
            }

            map.values()
                .find_map(|inner| extract_payload_id_from_value(inner, depth + 1))
        }
        JsonValue::Array(items) => items
            .iter()
            .find_map(|item| extract_payload_id_from_value(item, depth + 1)),
        _ => None,
    }
}

fn normalize_cipher_id(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(String::from(trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reconnect_delay_increases_before_hitting_ceiling() {
        let base_ms = 1_000;
        let max_ms = 30_000;

        let first = reconnect_delay_ms(base_ms, max_ms, 1);
        let second = reconnect_delay_ms(base_ms, max_ms, 2);
        let third = reconnect_delay_ms(base_ms, max_ms, 3);

        assert!(first < second);
        assert!(second < third);
    }

    #[test]
    fn reconnect_delay_is_capped_by_maximum() {
        let base_ms = 1_000;
        let max_ms = 5_000;

        for attempt in 0..32 {
            let delay = reconnect_delay_ms(base_ms, max_ms, attempt);
            assert!(delay <= max_ms);
            assert!(delay >= base_ms.min(max_ms));
        }
    }

    #[test]
    fn extract_payload_id_reads_top_level_id() {
        let payload = json!({
            "Id": "cipher-1",
            "Other": 1
        });

        let cipher_id = extract_payload_id(Some(&payload));
        assert_eq!(cipher_id.as_deref(), Some("cipher-1"));
    }

    #[test]
    fn extract_payload_id_reads_nested_payload() {
        let payload = json!({
            "Payload": {
                "Cipher": {
                    "id": "cipher-2"
                }
            }
        });

        let cipher_id = extract_payload_id(Some(&payload));
        assert_eq!(cipher_id.as_deref(), Some("cipher-2"));
    }

    #[test]
    fn extract_payload_id_returns_none_for_missing_id() {
        let payload = json!({
            "Payload": {
                "Type": 0
            }
        });

        assert!(extract_payload_id(Some(&payload)).is_none());
    }

    #[test]
    fn retryable_ws_error_reconnects_for_transient_disconnects() {
        assert!(is_retryable_ws_error(&AppError::NetworkRemoteError {
            status: 0,
            message: "connection reset by peer".to_string(),
        }));
        assert!(is_retryable_ws_error(&AppError::NetworkRemoteError {
            status: 502,
            message: "bad gateway".to_string()
        }));
    }

    #[test]
    fn retryable_ws_error_stops_for_auth_and_validation() {
        assert!(!is_retryable_ws_error(&AppError::NetworkRemoteError {
            status: 401,
            message: "unauthorized".to_string()
        }));
        assert!(!is_retryable_ws_error(&AppError::NetworkRemoteError {
            status: 403,
            message: "forbidden".to_string()
        }));
        assert!(!is_retryable_ws_error(&AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "invalid websocket endpoint".to_string(),
        }));
    }
}
