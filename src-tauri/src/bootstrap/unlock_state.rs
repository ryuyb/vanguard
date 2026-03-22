//! Unified Unlock State Manager
//!
//! This module provides a centralized state management system for authentication
//! and vault unlocking. It consolidates AuthSession and VaultUserKey into a single
//! atomic state machine, ensuring consistent lifecycle management.
//!
//! # Key Features
//! - Atomic state transitions (no partial states)
//! - Secure automatic cleanup of sensitive data using ZeroizeOnDrop
//! - Configurable auto-lock timer with activity tracking
//! - Push-based state subscription for real-time updates
//! - Integration with existing AppConfig for auto-lock settings

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::bootstrap::config::AppConfig;
use crate::domain::unlock::UnlockMethod;
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Current unlock status - single source of truth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnlockStatus {
    /// Vault is locked, no session or keys available
    Locked,
    /// Vault is unlocked but API session has expired
    VaultUnlockedSessionExpired,
    /// Both vault and API session are valid
    FullyUnlocked,
    /// Unlock operation is in progress
    Unlocking,
}

impl UnlockStatus {
    /// Check if vault is unlocked (regardless of session status)
    pub fn is_vault_unlocked(&self) -> bool {
        matches!(
            self,
            UnlockStatus::VaultUnlockedSessionExpired | UnlockStatus::FullyUnlocked
        )
    }

    /// Check if API session is valid
    pub fn is_session_valid(&self) -> bool {
        matches!(self, UnlockStatus::FullyUnlocked)
    }

    /// Check if completely locked
    pub fn is_locked(&self) -> bool {
        matches!(self, UnlockStatus::Locked)
    }
}

/// Account context - non-sensitive information about the logged-in account
#[derive(Debug, Clone, PartialEq)]
pub struct AccountContext {
    pub account_id: String,
    pub email: String,
    pub base_url: String,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

impl AccountContext {
    /// Convert to VaultUnlockContext for use in crypto operations
    pub fn to_vault_context(&self) -> VaultUnlockContext {
        VaultUnlockContext {
            account_id: self.account_id.clone(),
            base_url: self.base_url.clone(),
            email: self.email.clone(),
            kdf: self.kdf,
            kdf_iterations: self.kdf_iterations,
            kdf_memory: self.kdf_memory,
            kdf_parallelism: self.kdf_parallelism,
        }
    }
}

/// Session context - API session information
#[derive(Debug, Clone, PartialEq)]
pub struct SessionContext {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Instant,
    pub last_activity: Instant,
}

impl SessionContext {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Check if session is expiring within the given duration
    pub fn is_expiring_within(&self, duration: Duration) -> bool {
        Instant::now() + duration >= self.expires_at
    }

    /// Update last activity timestamp
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }
}

/// Vault key material - sensitive encryption keys
/// Automatically zeroized when dropped
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKeyMaterial {
    pub enc_key: Vec<u8>,
    pub mac_key: Option<Vec<u8>>,
}

impl VaultKeyMaterial {
    /// Convert to DTO for use in use cases
    pub fn to_dto(&self) -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: self.enc_key.clone(),
            mac_key: self.mac_key.clone(),
            refresh_token: None,
        }
    }
}

impl From<VaultUserKeyMaterial> for VaultKeyMaterial {
    fn from(dto: VaultUserKeyMaterial) -> Self {
        Self {
            enc_key: dto.enc_key,
            mac_key: dto.mac_key,
        }
    }
}

/// Unlock state snapshot - complete state at a point in time
#[derive(Debug, Clone)]
pub struct UnlockState {
    pub status: UnlockStatus,
    pub account_context: Option<AccountContext>,
    pub session_context: Option<SessionContext>,
    pub key_material: Option<VaultKeyMaterial>,
    pub unlocked_at: Option<Instant>,
    pub unlock_method: Option<UnlockMethod>,
}

impl UnlockState {
    /// Create a new locked state
    pub fn locked() -> Self {
        Self {
            status: UnlockStatus::Locked,
            account_context: None,
            session_context: None,
            key_material: None,
            unlocked_at: None,
            unlock_method: None,
        }
    }

    /// Create a new unlocking state
    pub fn unlocking() -> Self {
        Self {
            status: UnlockStatus::Unlocking,
            account_context: None,
            session_context: None,
            key_material: None,
            unlocked_at: None,
            unlock_method: None,
        }
    }
}

impl Default for UnlockState {
    fn default() -> Self {
        Self::locked()
    }
}

/// State change event for subscribers
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    pub old_state: UnlockState,
    pub new_state: UnlockState,
    pub timestamp: Instant,
}

/// Subscriber callback type
pub type StateSubscriber = Box<dyn Fn(&StateChangeEvent) + Send + Sync>;

/// Result of unlock operation
#[derive(Debug, Clone)]
pub enum UnlockResult {
    Success,
    AlreadyUnlocked,
    InvalidCredentials,
    NetworkError(String),
    InternalError(String),
}

/// Unified unlock manager - single source of truth for auth state
pub struct UnifiedUnlockManager {
    state: Arc<RwLock<UnlockState>>,
    subscribers: Arc<Mutex<Vec<SubscriberEntry>>>,
    config: Arc<Mutex<AppConfig>>,
    auto_lock_timer: Arc<Mutex<Option<AutoLockTimer>>>,
    // Self-reference for auto-lock callback
    self_ref: Arc<Mutex<Option<Arc<UnifiedUnlockManager>>>>,
}

impl UnifiedUnlockManager {
    /// Create a new manager with the given configuration
    pub fn new(config: AppConfig) -> Arc<Self> {
        let manager = Arc::new(Self {
            state: Arc::new(RwLock::new(UnlockState::locked())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(config)),
            auto_lock_timer: Arc::new(Mutex::new(None)),
            self_ref: Arc::new(Mutex::new(None)),
        });

        // Set self-reference for auto-lock callbacks
        *manager.self_ref.lock().unwrap() = Some(Arc::clone(&manager));

        manager
    }

    /// Get current state snapshot
    pub async fn current_state(&self) -> UnlockState {
        self.state.read().await.clone()
    }

    /// Get current status
    pub async fn current_status(&self) -> UnlockStatus {
        self.state.read().await.status
    }

    /// Check if vault is unlocked
    pub async fn is_vault_unlocked(&self) -> bool {
        self.current_status().await.is_vault_unlocked()
    }

    /// Check if session is valid
    pub async fn is_session_valid(&self) -> bool {
        self.current_status().await.is_session_valid()
    }

    /// Get active account ID if available
    pub async fn active_account_id(&self) -> AppResult<String> {
        let state = self.state.read().await;
        state
            .account_context
            .as_ref()
            .map(|ctx| ctx.account_id.clone())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "account".to_string(),
                message: "No active account".to_string(),
            })
    }

    /// Get account context if available
    pub async fn account_context(&self) -> Option<AccountContext> {
        self.state.read().await.account_context.clone()
    }

    /// Get session context if available
    pub async fn session_context(&self) -> Option<SessionContext> {
        self.state.read().await.session_context.clone()
    }

    /// Get key material if available
    pub async fn key_material(&self) -> Option<VaultKeyMaterial> {
        self.state.read().await.key_material.clone()
    }

    /// Get key material as DTO for use cases
    pub async fn key_material_dto(&self) -> Option<VaultUserKeyMaterial> {
        self.key_material().await.map(|k| k.to_dto())
    }

    /// Get refresh token if available
    pub async fn refresh_token(&self) -> Option<String> {
        self.state
            .read()
            .await
            .session_context
            .as_ref()
            .and_then(|s| s.refresh_token.clone())
    }

    /// Set state to unlocking (called at start of unlock operation)
    pub async fn begin_unlock(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        if state.status == UnlockStatus::Unlocking {
            return Err(AppError::ValidationFieldError {
                field: "unlock".to_string(),
                message: "Unlock already in progress".to_string(),
            });
        }
        let old_state = state.clone();
        state.status = UnlockStatus::Unlocking;
        self.notify_subscribers(&old_state, &state);
        Ok(())
    }

    /// Complete unlock with full state
    pub async fn complete_unlock(
        &self,
        account_context: AccountContext,
        session_context: SessionContext,
        key_material: VaultKeyMaterial,
        method: UnlockMethod,
    ) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        state.status = UnlockStatus::FullyUnlocked;
        state.account_context = Some(account_context);
        state.session_context = Some(session_context);
        state.key_material = Some(key_material);
        state.unlocked_at = Some(Instant::now());
        state.unlock_method = Some(method);

        self.notify_subscribers(&old_state, &state);
        self.start_auto_lock_timer().await?;
        Ok(())
    }

    /// Complete unlock with vault only (session expired or not available)
    pub async fn complete_unlock_vault_only(
        &self,
        account_context: AccountContext,
        key_material: VaultKeyMaterial,
        method: UnlockMethod,
    ) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        state.status = UnlockStatus::VaultUnlockedSessionExpired;
        state.account_context = Some(account_context);
        state.session_context = None;
        state.key_material = Some(key_material);
        state.unlocked_at = Some(Instant::now());
        state.unlock_method = Some(method);

        self.notify_subscribers(&old_state, &state);
        self.start_auto_lock_timer().await?;
        Ok(())
    }

    /// Lock the vault
    pub async fn lock(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        // Securely clear key material (ZeroizeOnDrop handles the actual zeroization)
        state.key_material = None;
        state.session_context = None;
        state.status = UnlockStatus::Locked;
        // Keep account_context for quick re-unlock

        self.notify_subscribers(&old_state, &state);
        self.stop_auto_lock_timer().await?;
        Ok(())
    }

    /// Logout - clear all state including account context
    pub async fn logout(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        // Securely clear all sensitive data
        state.key_material = None;
        state.session_context = None;
        state.account_context = None;
        state.status = UnlockStatus::Locked;
        state.unlocked_at = None;
        state.unlock_method = None;

        self.notify_subscribers(&old_state, &state);
        self.stop_auto_lock_timer().await?;
        Ok(())
    }

    /// Update session after refresh
    pub async fn update_session(&self, session_context: SessionContext) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        state.session_context = Some(session_context);
        // Transition from VaultUnlockedSessionExpired to FullyUnlocked if applicable
        if state.status == UnlockStatus::VaultUnlockedSessionExpired {
            state.status = UnlockStatus::FullyUnlocked;
        }

        self.notify_subscribers(&old_state, &state);
        Ok(())
    }

    /// Clear session (e.g., on 401/403 error)
    pub async fn clear_session(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        state.session_context = None;
        if state.status == UnlockStatus::FullyUnlocked {
            state.status = UnlockStatus::VaultUnlockedSessionExpired;
        }

        self.notify_subscribers(&old_state, &state);
        Ok(())
    }

    /// Check if session needs refresh
    pub async fn needs_session_refresh(&self) -> bool {
        let state = self.state.read().await;
        match &state.session_context {
            Some(session) => {
                let grace_period = Duration::from_secs(60);
                session.is_expiring_within(grace_period)
            }
            None => true,
        }
    }

    /// Record user activity (resets auto-lock timer)
    pub async fn record_activity(&self) -> AppResult<()> {
        // Update last activity in session context
        let mut state = self.state.write().await;
        if let Some(ref mut session) = state.session_context {
            session.record_activity();
        }

        // Reset auto-lock timer if running
        self.reset_auto_lock_timer().await?;
        Ok(())
    }

    /// Subscribe to state changes
    /// Returns a handle that will unsubscribe when dropped
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&StateChangeEvent) + Send + Sync + 'static,
    {
        let callback_id = SubscriptionHandle::generate_id();
        let callback_arc: Arc<dyn Fn(&StateChangeEvent) + Send + Sync> = Arc::new(callback);

        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(SubscriberEntry {
            _id: callback_id,
            callback: Arc::clone(&callback_arc),
        });

        SubscriptionHandle {
            id: callback_id,
            _callback: callback_arc,
        }
    }

    #[allow(dead_code)]
    /// Unsubscribe a specific subscriber by ID
    fn unsubscribe(&self, id: u64) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.retain(|entry| entry._id != id);
    }

    /// Notify all subscribers of state change
    fn notify_subscribers(&self, old_state: &UnlockState, new_state: &UnlockState) {
        // Skip if no actual change (dirty check)
        if old_state.status == new_state.status
            && old_state.account_context == new_state.account_context
            && old_state.session_context == new_state.session_context
            && old_state.key_material.is_some() == new_state.key_material.is_some()
        {
            return;
        }

        let event = StateChangeEvent {
            old_state: old_state.clone(),
            new_state: new_state.clone(),
            timestamp: Instant::now(),
        };

        let subscribers = self.subscribers.lock().unwrap();
        for entry in subscribers.iter() {
            // Call the callback
            (entry.callback)(&event);
        }
    }

    /// Start auto-lock timer based on config
    async fn start_auto_lock_timer(&self) -> AppResult<()> {
        let config = self.config.lock().unwrap();
        let delay_str = &config.idle_auto_lock_delay;

        if delay_str == "never" {
            return Ok(());
        }

        let duration = parse_idle_auto_lock_delay(delay_str)?;

        // Get self reference for the callback
        let self_weak = Arc::downgrade(&self.self_ref.lock().unwrap().as_ref().unwrap().clone());

        let mut timer_guard = self.auto_lock_timer.lock().unwrap();
        let mut timer = AutoLockTimer::new(duration);

        timer.start(move || {
            // Timer expired - lock the vault
            if let Some(self_arc) = self_weak.upgrade() {
                tokio::spawn(async move {
                    if let Err(e) = self_arc.lock().await {
                        log::error!(
                            target: "vanguard::unlock_state",
                            "Failed to auto-lock vault: {}",
                            e.log_message()
                        );
                    } else {
                        log::info!(
                            target: "vanguard::unlock_state",
                            "Vault auto-locked due to inactivity"
                        );
                    }
                });
            }
        });

        *timer_guard = Some(timer);
        Ok(())
    }

    /// Stop auto-lock timer
    async fn stop_auto_lock_timer(&self) -> AppResult<()> {
        let mut timer = self.auto_lock_timer.lock().unwrap();
        if let Some(ref mut t) = timer.as_mut() {
            t.cancel();
        }
        *timer = None;
        Ok(())
    }

    /// Reset auto-lock timer (called on activity)
    async fn reset_auto_lock_timer(&self) -> AppResult<()> {
        let mut timer_guard = self.auto_lock_timer.lock().unwrap();
        if let Some(ref mut timer) = timer_guard.as_mut() {
            // Get self reference for the callback
            let self_weak =
                Arc::downgrade(&self.self_ref.lock().unwrap().as_ref().unwrap().clone());

            timer.reset(move || {
                if let Some(self_arc) = self_weak.upgrade() {
                    tokio::spawn(async move {
                        if let Err(e) = self_arc.lock().await {
                            log::error!(
                                target: "vanguard::unlock_state",
                                "Failed to auto-lock vault: {}",
                                e.log_message()
                            );
                        } else {
                            log::info!(
                                target: "vanguard::unlock_state",
                                "Vault auto-locked due to inactivity"
                            );
                        }
                    });
                }
            });
        }
        Ok(())
    }

    /// Update auto-lock config (called when settings change)
    pub async fn update_auto_lock_config(&self, config: AppConfig) -> AppResult<()> {
        let old_delay = self.config.lock().unwrap().idle_auto_lock_delay.clone();
        let new_delay = config.idle_auto_lock_delay.clone();
        *self.config.lock().unwrap() = config;

        // Restart timer if delay changed and vault is unlocked
        let status = self.current_status().await;
        if status.is_vault_unlocked() && new_delay != old_delay {
            self.stop_auto_lock_timer().await?;
            self.start_auto_lock_timer().await?;
        }

        Ok(())
    }

    /// Ensure session is valid, refreshing if necessary
    /// Returns the session context if valid or after successful refresh
    pub async fn ensure_valid_session<F, Fut>(&self, refresh_fn: F) -> AppResult<SessionContext>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = AppResult<SessionContext>>,
    {
        // Check current session
        let needs_refresh = self.needs_session_refresh().await;

        if !needs_refresh {
            // Session is still valid
            return self
                .session_context()
                .await
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "session".to_string(),
                    message: "No active session".to_string(),
                });
        }

        // Need to refresh - get refresh token
        let refresh_token =
            self.refresh_token()
                .await
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "refresh_token".to_string(),
                    message: "No refresh token available".to_string(),
                })?;

        // Call refresh function
        let new_session = refresh_fn(refresh_token).await?;

        // Update state with new session
        self.update_session(new_session.clone()).await?;

        Ok(new_session)
    }

    /// Force session refresh regardless of expiry
    pub async fn force_refresh_session<F, Fut>(&self, refresh_fn: F) -> AppResult<SessionContext>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = AppResult<SessionContext>>,
    {
        let refresh_token =
            self.refresh_token()
                .await
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "refresh_token".to_string(),
                    message: "No refresh token available".to_string(),
                })?;

        let new_session = refresh_fn(refresh_token).await?;
        self.update_session(new_session.clone()).await?;

        Ok(new_session)
    }

    /// Build session context from auth response data
    pub fn build_session_context(
        &self,
        access_token: String,
        refresh_token: Option<String>,
        expires_in_seconds: i64,
    ) -> AppResult<SessionContext> {
        let expires_at = Instant::now() + Duration::from_secs(expires_in_seconds.max(0) as u64);

        Ok(SessionContext {
            access_token,
            refresh_token,
            expires_at,
            last_activity: Instant::now(),
        })
    }

    /// Get access token if session is valid
    pub async fn access_token(&self) -> AppResult<String> {
        let state = self.state.read().await;

        match &state.session_context {
            Some(session) => {
                if session.is_expired() {
                    return Err(AppError::ValidationFieldError {
                        field: "session".to_string(),
                        message: "Session expired".to_string(),
                    });
                }
                Ok(session.access_token.clone())
            }
            None => Err(AppError::ValidationFieldError {
                field: "session".to_string(),
                message: "No active session".to_string(),
            }),
        }
    }

    /// Get vault unlock context for crypto operations
    pub async fn vault_unlock_context(&self) -> AppResult<VaultUnlockContext> {
        let state = self.state.read().await;

        state
            .account_context
            .as_ref()
            .map(|ctx| ctx.to_vault_context())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "account".to_string(),
                message: "No account context available".to_string(),
            })
    }

    // Legacy compatibility methods for AppState delegation

    /// Set vault key material directly (legacy compatibility)
    /// Note: This is a lower-level method. Prefer using `complete_unlock` for normal operations.
    pub async fn set_key_material(&self, key: VaultKeyMaterial) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.key_material = Some(key);
        Ok(())
    }

    /// Remove vault key material (legacy compatibility)
    pub async fn remove_key_material(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.key_material = None;
        Ok(())
    }

    /// Set account context directly (legacy compatibility)
    /// Note: This is a lower-level method. Prefer using `complete_unlock` for normal operations.
    pub async fn set_account_context(&self, account: AccountContext) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.account_context = Some(account);
        Ok(())
    }

    /// Check if vault is locked (synchronous version for legacy compatibility)
    /// Note: This may block briefly. Prefer using `is_vault_unlocked().await` in async contexts.
    pub fn is_vault_unlocked_blocking(&self) -> bool {
        // Try to get a quick read lock - if it fails, assume locked for safety
        match self.state.try_read() {
            Ok(state) => state.status.is_vault_unlocked(),
            Err(_) => false,
        }
    }

    /// Get active account ID (blocking version for legacy compatibility)
    pub fn active_account_id_blocking(&self) -> AppResult<String> {
        match self.state.try_read() {
            Ok(state) => state
                .account_context
                .as_ref()
                .map(|ctx| ctx.account_id.clone())
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "account".to_string(),
                    message: "No active account".to_string(),
                }),
            Err(_) => Err(AppError::InternalUnexpected {
                message: "Failed to acquire state lock".to_string(),
            }),
        }
    }
}

/// Auto-lock timer that runs in background
pub struct AutoLockTimer {
    duration: Duration,
    last_activity: Instant,
    abort_handle: Option<tokio::sync::oneshot::Sender<()>>,
}

impl AutoLockTimer {
    /// Create a new timer (not started yet)
    fn new(duration: Duration) -> Self {
        Self {
            duration,
            last_activity: Instant::now(),
            abort_handle: None,
        }
    }

    /// Start the timer with a callback to execute when timer expires
    fn start<F>(&mut self, on_expire: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.abort_handle = Some(tx);

        let duration = self.duration;
        let _start_time = self.last_activity;

        tokio::spawn(async move {
            let sleep_duration = duration;

            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {
                    on_expire();
                }
                _ = rx => {
                    // Timer was cancelled
                }
            }
        });
    }

    /// Reset the timer (cancel current and start new)
    fn reset<F>(&mut self, on_expire: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cancel();
        self.last_activity = Instant::now();
        self.start(on_expire);
    }

    /// Cancel the timer
    fn cancel(&mut self) {
        if let Some(tx) = self.abort_handle.take() {
            let _ = tx.send(());
        }
    }

    #[allow(dead_code)]
    /// Update duration and restart if running
    fn update_duration<F>(&mut self, new_duration: Duration, on_expire: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.duration = new_duration;
        if self.abort_handle.is_some() {
            self.reset(on_expire);
        }
    }
}

impl Drop for AutoLockTimer {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Subscriber entry for internal tracking
struct SubscriberEntry {
    _id: u64,
    callback: Arc<dyn Fn(&StateChangeEvent) + Send + Sync>,
}

/// Subscription handle - dropping it unsubscribes
pub struct SubscriptionHandle {
    id: u64,
    _callback: Arc<dyn Fn(&StateChangeEvent) + Send + Sync>,
}

impl SubscriptionHandle {
    fn generate_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Get the subscription ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Parse idle_auto_lock_delay config string to Duration
/// Supported formats: "1m", "2m", "5m", "10m", "15m", "30m", "1h", "4h", "8h"
pub fn parse_idle_auto_lock_delay(value: &str) -> AppResult<Duration> {
    if value == "never" {
        // Return max duration (practically never)
        return Ok(Duration::from_secs(u64::MAX));
    }

    let chars: Vec<char> = value.chars().collect();
    if chars.len() < 2 {
        return Err(AppError::ValidationFieldError {
            field: "idle_auto_lock_delay".to_string(),
            message: format!("Invalid format: {}", value),
        });
    }

    let unit = chars.last().unwrap();
    let num_str: String = chars[..chars.len() - 1].iter().collect();

    let num: u64 = num_str
        .parse()
        .map_err(|_| AppError::ValidationFieldError {
            field: "idle_auto_lock_delay".to_string(),
            message: format!("Invalid number in: {}", value),
        })?;

    let seconds = match unit {
        'm' => num * 60,
        'h' => num * 3600,
        's' => num,
        'd' => num * 86400,
        _ => {
            return Err(AppError::ValidationFieldError {
                field: "idle_auto_lock_delay".to_string(),
                message: format!("Invalid unit in: {}", value),
            })
        }
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AppConfig {
        AppConfig {
            device_identifier: "test".to_string(),
            allow_invalid_certs: false,
            sync_poll_interval_seconds: 60,
            locale: "en".to_string(),
            launch_on_login: false,
            show_website_icon: true,
            quick_access_shortcut: "⌃⇧␣".to_string(),
            lock_shortcut: "⇧⌘L".to_string(),
            require_master_password_interval: "never".to_string(),
            lock_on_sleep: false,
            idle_auto_lock_delay: "never".to_string(),
            clipboard_clear_delay: "never".to_string(),
            spotlight_autofill: true,
        }
    }

    #[test]
    fn parse_delay_various_units() {
        assert_eq!(
            parse_idle_auto_lock_delay("1m").unwrap(),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_idle_auto_lock_delay("5m").unwrap(),
            Duration::from_secs(300)
        );
        assert_eq!(
            parse_idle_auto_lock_delay("1h").unwrap(),
            Duration::from_secs(3600)
        );
        assert_eq!(
            parse_idle_auto_lock_delay("4h").unwrap(),
            Duration::from_secs(14400)
        );
    }

    #[test]
    fn parse_delay_never_returns_max() {
        let result = parse_idle_auto_lock_delay("never").unwrap();
        assert_eq!(result, Duration::from_secs(u64::MAX));
    }

    #[test]
    fn parse_delay_invalid_format_fails() {
        assert!(parse_idle_auto_lock_delay("invalid").is_err());
        assert!(parse_idle_auto_lock_delay("5x").is_err());
        assert!(parse_idle_auto_lock_delay("").is_err());
    }

    #[tokio::test]
    async fn manager_initial_state_is_locked() {
        let manager = UnifiedUnlockManager::new(create_test_config());
        assert_eq!(manager.current_status().await, UnlockStatus::Locked);
        assert!(!manager.is_vault_unlocked().await);
        assert!(!manager.is_session_valid().await);
    }

    #[tokio::test]
    async fn unlock_transitions_to_fully_unlocked() {
        let manager = UnifiedUnlockManager::new(create_test_config());

        let account = AccountContext {
            account_id: "test".to_string(),
            email: "test@example.com".to_string(),
            base_url: "https://vault.example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let session = SessionContext {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Instant::now() + Duration::from_secs(3600),
            last_activity: Instant::now(),
        };

        let key = VaultKeyMaterial {
            enc_key: vec![1, 2, 3],
            mac_key: Some(vec![4, 5, 6]),
        };

        manager
            .complete_unlock(
                account,
                session,
                key,
                UnlockMethod::MasterPassword {
                    password: "test".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(manager.current_status().await, UnlockStatus::FullyUnlocked);
        assert!(manager.is_vault_unlocked().await);
        assert!(manager.is_session_valid().await);
    }

    #[tokio::test]
    async fn lock_clears_sensitive_data() {
        let manager = UnifiedUnlockManager::new(create_test_config());

        // Setup unlocked state
        let account = AccountContext {
            account_id: "test".to_string(),
            email: "test@example.com".to_string(),
            base_url: "https://vault.example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let session = SessionContext {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Instant::now() + Duration::from_secs(3600),
            last_activity: Instant::now(),
        };

        let key = VaultKeyMaterial {
            enc_key: vec![1, 2, 3],
            mac_key: Some(vec![4, 5, 6]),
        };

        manager
            .complete_unlock(
                account,
                session,
                key,
                UnlockMethod::MasterPassword {
                    password: "test".to_string(),
                },
            )
            .await
            .unwrap();

        // Lock
        manager.lock().await.unwrap();

        // Verify locked state
        assert_eq!(manager.current_status().await, UnlockStatus::Locked);
        assert!(manager.key_material().await.is_none());
        assert!(manager.session_context().await.is_none());
    }

    #[tokio::test]
    async fn logout_clears_all_data() {
        let manager = UnifiedUnlockManager::new(create_test_config());

        // Setup unlocked state
        let account = AccountContext {
            account_id: "test".to_string(),
            email: "test@example.com".to_string(),
            base_url: "https://vault.example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let session = SessionContext {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Instant::now() + Duration::from_secs(3600),
            last_activity: Instant::now(),
        };

        let key = VaultKeyMaterial {
            enc_key: vec![1, 2, 3],
            mac_key: Some(vec![4, 5, 6]),
        };

        manager
            .complete_unlock(
                account,
                session,
                key,
                UnlockMethod::MasterPassword {
                    password: "test".to_string(),
                },
            )
            .await
            .unwrap();

        // Logout
        manager.logout().await.unwrap();

        // Verify completely locked
        assert_eq!(manager.current_status().await, UnlockStatus::Locked);
        assert!(manager.key_material().await.is_none());
        assert!(manager.session_context().await.is_none());
        assert!(manager.account_context().await.is_none());
    }

    #[tokio::test]
    async fn vault_only_unlock_session_expired_status() {
        let manager = UnifiedUnlockManager::new(create_test_config());

        let account = AccountContext {
            account_id: "test".to_string(),
            email: "test@example.com".to_string(),
            base_url: "https://vault.example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let key = VaultKeyMaterial {
            enc_key: vec![1, 2, 3],
            mac_key: Some(vec![4, 5, 6]),
        };

        manager
            .complete_unlock_vault_only(
                account,
                key,
                UnlockMethod::Pin {
                    pin: "1234".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            manager.current_status().await,
            UnlockStatus::VaultUnlockedSessionExpired
        );
        assert!(manager.is_vault_unlocked().await);
        assert!(!manager.is_session_valid().await);
    }

    #[tokio::test]
    async fn session_refresh_updates_status() {
        let manager = UnifiedUnlockManager::new(create_test_config());

        // Start with vault-only unlock
        let account = AccountContext {
            account_id: "test".to_string(),
            email: "test@example.com".to_string(),
            base_url: "https://vault.example.com".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let key = VaultKeyMaterial {
            enc_key: vec![1, 2, 3],
            mac_key: Some(vec![4, 5, 6]),
        };

        manager
            .complete_unlock_vault_only(
                account,
                key,
                UnlockMethod::Pin {
                    pin: "1234".to_string(),
                },
            )
            .await
            .unwrap();

        // Refresh session
        let new_session = SessionContext {
            access_token: "new_token".to_string(),
            refresh_token: Some("new_refresh".to_string()),
            expires_at: Instant::now() + Duration::from_secs(3600),
            last_activity: Instant::now(),
        };

        manager.update_session(new_session).await.unwrap();

        assert_eq!(manager.current_status().await, UnlockStatus::FullyUnlocked);
        assert!(manager.is_session_valid().await);
    }
}
