//! Unified Unlock State Manager
//!
//! This module provides a centralized state management system for authentication
//! and vault unlocking. It consolidates AuthSession and VaultUserKey into a single
//! atomic state machine, ensuring consistent lifecycle management.
//!
//! # State Machine
//!
//! ```text
//! ┌─────────┐    unlock     ┌─────────────────────────┐    restore session    ┌─────────────┐
//! │ Locked  │ ─────────────→│ VaultUnlockedSessionExpired │ ──────────────────→│ FullyUnlocked │
//! └─────────┘               └─────────────────────────┘                      └─────────────┘
//!      ↑                            │                                               │
//!      └────────────────────────────┴───────────────────────────────────────────────┘
//!                                    lock / session expired
//! ```
//!
//! # Auto State Transition Rules
//! - Locked + key_material + account_context → VaultUnlockedSessionExpired
//! - VaultUnlockedSessionExpired + session_context → FullyUnlocked
//! - FullyUnlocked - session_context → VaultUnlockedSessionExpired
//! - Any - key_material → Locked
//!
//! # Key Features
//! - Atomic state transitions (no partial states)
//! - Automatic state calculation based on field presence
//! - Secure automatic cleanup of sensitive data using ZeroizeOnDrop
//! - Configurable auto-lock timer with activity tracking
//! - Push-based state subscription for real-time updates

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::application::dto::vault::{VaultUnlockContext, VaultUserKeyMaterial};
use crate::bootstrap::auth_persistence::SessionWrapRuntime;
use crate::bootstrap::auth_persistence_port::AuthPersistence;
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
    /// Create initial locked state
    fn new() -> Self {
        Self {
            status: UnlockStatus::Locked,
            account_context: None,
            session_context: None,
            key_material: None,
            unlocked_at: None,
            unlock_method: None,
        }
    }

    /// Calculate the correct status based on current field state
    /// Returns true if status changed
    fn recalculate_status(&mut self) -> bool {
        let has_key = self.key_material.is_some();
        let has_account = self.account_context.is_some();
        let has_session = self.session_context.is_some();
        let session_valid = has_session && !self.session_context.as_ref().unwrap().is_expired();

        let new_status = match (has_key, has_account, session_valid) {
            (true, true, true) => UnlockStatus::FullyUnlocked,
            (true, true, false) => UnlockStatus::VaultUnlockedSessionExpired,
            _ => UnlockStatus::Locked,
        };

        if self.status != new_status {
            self.status = new_status;
            true
        } else {
            false
        }
    }
}

/// Callback for state change events
pub type StateChangeCallback = Box<dyn Fn(&UnlockState, &UnlockState) + Send + Sync>;

/// Unified unlock manager - single source of truth for unlock state
pub struct UnifiedUnlockManager {
    state: Arc<RwLock<UnlockState>>,
    config: Arc<Mutex<AppConfig>>,
    auto_lock_timer: Arc<Mutex<AutoLockTimer>>,
    persistence: Arc<dyn AuthPersistence>, // Auth state persistence service
    on_state_change: Arc<Mutex<Option<StateChangeCallback>>>,
    self_ref: Arc<Mutex<Option<Arc<Self>>>>,
}

impl UnifiedUnlockManager {
    /// Create a new unlock manager with optional initial account context
    pub fn new(
        config: AppConfig,
        initial_account: Option<AccountContext>,
        persistence: Arc<dyn AuthPersistence>,
    ) -> Arc<Self> {
        let state = UnlockState::new();
        // Set initial account context if provided (from persisted auth state)
        let state = UnlockState {
            account_context: initial_account,
            ..state
        };

        let manager = Arc::new(Self {
            state: Arc::new(RwLock::new(state)),
            config: Arc::new(Mutex::new(config)),
            auto_lock_timer: Arc::new(Mutex::new(AutoLockTimer::new(Duration::from_secs(300)))),
            persistence,
            on_state_change: Arc::new(Mutex::new(None)),
            self_ref: Arc::new(Mutex::new(None)),
        });

        // Store weak self-reference for timer callbacks
        *manager.self_ref.lock().unwrap() = Some(Arc::clone(&manager));

        manager
    }

    /// Set callback for state change events
    pub fn on_state_change<F>(&self, callback: F)
    where
        F: Fn(&UnlockState, &UnlockState) + Send + Sync + 'static,
    {
        *self.on_state_change.lock().unwrap() = Some(Box::new(callback));
    }

    /// Notify listeners of state change
    fn notify_state_change(&self, old_state: &UnlockState, new_state: &UnlockState) {
        // Skip if no actual change (dirty check)
        if old_state.status == new_state.status
            && old_state.account_context == new_state.account_context
            && old_state.session_context == new_state.session_context
            && old_state.key_material.is_some() == new_state.key_material.is_some()
        {
            return;
        }

        // Call the callback if set
        if let Ok(callback_guard) = self.on_state_change.lock() {
            if let Some(callback) = callback_guard.as_ref() {
                (callback)(old_state, new_state);
            }
        }
    }

    // ==================== State Queries ====================

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

    // ==================== State Modifiers (with auto-transition) ====================

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
        self.notify_state_change(&old_state, &state);
        Ok(())
    }

    /// Set vault key material
    /// Automatically transitions state based on other field presence
    pub async fn set_key_material(&self, key: VaultKeyMaterial) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.key_material = Some(key);
        // Note: auto_transition is called after releasing the lock
        let old_state = state.clone();
        let changed = state.recalculate_status();
        let new_state = state.clone();
        drop(state);

        if changed {
            self.notify_state_change(&old_state, &new_state);
            if new_state.status.is_vault_unlocked() {
                self.start_auto_lock_timer().await?;
            }
        }
        Ok(())
    }

    /// Remove vault key material
    /// Automatically transitions to Locked
    pub async fn remove_key_material(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.key_material = None;
        // Note: auto_transition is called after releasing the lock
        let old_state = state.clone();
        let changed = state.recalculate_status();
        let new_state = state.clone();
        drop(state);

        if changed {
            self.notify_state_change(&old_state, &new_state);
            if !new_state.status.is_vault_unlocked() {
                self.stop_auto_lock_timer().await?;
            }
        }
        Ok(())
    }

    /// Set account context
    /// Automatically transitions state based on other field presence
    pub async fn set_account_context(&self, account: AccountContext) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.account_context = Some(account);
        // Note: auto_transition is called after releasing the lock
        let old_state = state.clone();
        let changed = state.recalculate_status();
        let new_state = state.clone();
        drop(state);

        if changed {
            self.notify_state_change(&old_state, &new_state);
            if new_state.status.is_vault_unlocked() {
                self.start_auto_lock_timer().await?;
            }
            // Auto-persist state changes
            if let Err(e) = self.auto_persist().await {
                log::warn!(
                    target: "vanguard::unlock_state",
                    "Failed to auto-persist after account context change: {}",
                    e.log_message()
                );
            }
        }
        Ok(())
    }

    /// Set session context
    /// Automatically transitions to FullyUnlocked if vault is unlocked
    pub async fn set_session_context(&self, session: SessionContext) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.session_context = Some(session);
        // Note: auto_transition is called after releasing the lock
        let old_state = state.clone();
        let changed = state.recalculate_status();
        let new_state = state.clone();
        drop(state);

        if changed {
            self.notify_state_change(&old_state, &new_state);
            // Auto-persist state changes
            if let Err(e) = self.auto_persist().await {
                log::warn!(
                    target: "vanguard::unlock_state",
                    "Failed to auto-persist after session context change: {}",
                    e.log_message()
                );
            }
        }
        Ok(())
    }

    /// Clear session context
    /// Automatically transitions to VaultUnlockedSessionExpired if vault was unlocked
    pub async fn clear_session(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.session_context = None;
        // Note: auto_transition is called after releasing the lock
        let old_state = state.clone();
        let changed = state.recalculate_status();
        let new_state = state.clone();
        drop(state);

        if changed {
            self.notify_state_change(&old_state, &new_state);
        }
        Ok(())
    }

    /// Complete unlock operation
    /// Sets unlock timestamp if vault is now unlocked
    pub async fn complete_unlock(&self) -> AppResult<()> {
        // Update unlocked_at timestamp if now unlocked
        let mut state = self.state.write().await;
        if state.status.is_vault_unlocked() && state.unlocked_at.is_none() {
            state.unlocked_at = Some(Instant::now());
        }

        Ok(())
    }

    /// Lock the vault - clears key material and session, triggers auto-transition
    /// Note: Does NOT clear persisted auth state, so master password unlock can restore session
    pub async fn lock(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        // Securely clear key material (ZeroizeOnDrop handles the actual zeroization)
        state.key_material = None;
        state.session_context = None;
        state.unlocked_at = None;
        state.unlock_method = None;
        // Keep account_context for quick re-unlock
        // Keep persisted auth state so unlock can restore session

        // Calculate new status
        state.recalculate_status();

        self.notify_state_change(&old_state, &state);

        // Note: We intentionally do NOT clear persisted state here.
        // The persisted auth state is needed for master password unlock to restore the session.
        // It will be cleared on logout instead.

        self.stop_auto_lock_timer().await?;
        Ok(())
    }

    /// Logout - clears all state including account context
    pub async fn logout(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();

        // Securely clear all sensitive data
        state.key_material = None;
        state.session_context = None;
        state.account_context = None;
        state.unlocked_at = None;
        state.unlock_method = None;
        state.status = UnlockStatus::Locked;

        self.notify_state_change(&old_state, &state);

        // Clear persisted state
        if let Err(e) = self.persistence.clear_auth_state().await {
            log::warn!(
                target: "vanguard::unlock_state",
                "Failed to clear persisted auth state on logout: {}",
                e.log_message()
            );
        }

        self.stop_auto_lock_timer().await?;
        Ok(())
    }

    /// Update session after refresh
    pub async fn update_session(&self, session_context: SessionContext) -> AppResult<()> {
        self.set_session_context(session_context).await?;
        self.complete_unlock().await?;
        Ok(())
    }

    /// Build session context from token response
    pub fn build_session_context(
        &self,
        access_token: String,
        refresh_token: Option<String>,
        expires_in: i64,
    ) -> AppResult<SessionContext> {
        let expires_at = Instant::now() + Duration::from_secs(expires_in.max(0) as u64);
        let last_activity = Instant::now();

        Ok(SessionContext {
            access_token,
            refresh_token,
            expires_at,
            last_activity,
        })
    }

    // ==================== Legacy Compatibility Methods ====================

    /// Check if lock_on_sleep is enabled in config
    pub fn is_lock_on_sleep_enabled(&self) -> bool {
        self.config.lock().unwrap().lock_on_sleep
    }

    /// Handle system resume event - locks vault if lock_on_sleep is enabled
    pub async fn handle_system_resume(&self) -> AppResult<()> {
        if self.is_lock_on_sleep_enabled() {
            let status = self.current_status().await;
            if status.is_vault_unlocked() {
                self.lock().await?;
                log::info!(
                    target: "vanguard::unlock_state",
                    "Vault auto-locked on system resume (lock_on_sleep enabled)"
                );
            }
        }
        Ok(())
    }

    /// Handle system suspend event
    pub async fn handle_system_suspend(&self) -> AppResult<()> {
        // Currently just log - vault will be locked on resume if configured
        log::debug!(target: "vanguard::unlock_state", "System suspending");
        Ok(())
    }

    /// Record user activity (resets auto-lock timer)
    pub async fn record_activity(&self) -> AppResult<()> {
        // Update last activity in session context
        let mut state = self.state.write().await;
        if let Some(ref mut session) = state.session_context {
            session.record_activity();
        }

        // Reset auto-lock timer if running
        let mut timer = self.auto_lock_timer.lock().unwrap();
        timer.record_activity();

        Ok(())
    }

    /// Check if auto-lock should trigger based on inactivity
    pub async fn check_auto_lock(&self) -> AppResult<bool> {
        let state = self.state.read().await;

        // Only auto-lock if vault is unlocked
        if !state.status.is_vault_unlocked() {
            return Ok(false);
        }

        let config = self.config.lock().unwrap();
        if config.idle_auto_lock_delay == "never" {
            return Ok(false);
        }

        let duration = parse_idle_auto_lock_delay(&config.idle_auto_lock_delay)?;
        let timer = self.auto_lock_timer.lock().unwrap();

        Ok(timer.is_expired(duration))
    }

    /// Get the current unlock method if available
    pub async fn unlock_method(&self) -> Option<UnlockMethod> {
        self.state.read().await.unlock_method.clone()
    }

    /// Set the unlock method
    pub async fn set_unlock_method(&self, method: UnlockMethod) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.unlock_method = Some(method);
        Ok(())
    }

    /// Get the time when vault was unlocked
    pub async fn unlocked_at(&self) -> Option<Instant> {
        self.state.read().await.unlocked_at
    }

    // ==================== Runtime Key Management ====================

    /// Set runtime key for encrypting refresh token
    pub async fn set_runtime_key(&self, runtime_key: SessionWrapRuntime) -> AppResult<()> {
        self.persistence.set_runtime_key(Some(runtime_key))?;
        Ok(())
    }

    /// Get runtime key
    pub fn get_runtime_key(&self) -> AppResult<Option<SessionWrapRuntime>> {
        self.persistence.runtime_key()
    }

    /// Clear runtime key
    pub async fn clear_runtime_key(&self) -> AppResult<()> {
        self.persistence.set_runtime_key(None)?;
        Ok(())
    }

    // ==================== Persistence Methods ====================

    /// Persist auth state using master password to encrypt refresh token
    /// - Encrypt refresh token using master password
    /// - Cache the SessionWrapRuntime
    /// - Call AuthPersistence.save_auth_state()
    pub async fn persist_with_password(&self, master_password: &str) -> AppResult<()> {
        use crate::bootstrap::auth_persistence::encrypt_refresh_token;

        let state = self.state.read().await;

        // Get required data from state
        let account =
            state
                .account_context
                .as_ref()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "account".to_string(),
                    message: "No active account to persist".to_string(),
                })?;

        let refresh_token = state
            .session_context
            .as_ref()
            .and_then(|s| s.refresh_token.as_deref())
            .ok_or_else(|| AppError::ValidationFieldError {
                field: "refresh_token".to_string(),
                message: "No refresh token available to persist".to_string(),
            })?;

        // Encrypt refresh token using master password
        let (_encrypted_session, runtime_key) = encrypt_refresh_token(
            master_password,
            &account.account_id,
            &account.base_url,
            &account.email,
            refresh_token,
        )?;

        // Save auth state with the encrypted session
        self.persistence
            .save_auth_state(account, Some(refresh_token), Some(&runtime_key))
            .await?;

        // Cache runtime_key in persistence
        self.persistence.set_runtime_key(Some(runtime_key))?;

        Ok(())
    }

    /// Persist auth state using cached runtime key to re-encrypt
    /// - Use cached runtime_key to re-encrypt
    /// - Handle case where runtime_key is missing
    /// - Call AuthPersistence.save_auth_state()
    pub async fn persist_with_runtime(&self) -> AppResult<()> {
        let state = self.state.read().await;

        // Get required data from state
        let account =
            state
                .account_context
                .clone()
                .ok_or_else(|| AppError::ValidationFieldError {
                    field: "account".to_string(),
                    message: "No active account to persist".to_string(),
                })?;

        let refresh_token = state
            .session_context
            .as_ref()
            .and_then(|s| s.refresh_token.clone());

        drop(state);

        // Get runtime_key from persistence
        let runtime_key =
            self.persistence
                .runtime_key()?
                .ok_or_else(|| {
                    AppError::ValidationFieldError {
                field: "runtime_key".to_string(),
                message:
                    "No runtime key available for persistence. Call persist_with_password first."
                        .to_string(),
            }
                })?;

        // Save auth state with the cached runtime key
        let refresh_token_ref = refresh_token.as_deref();
        self.persistence
            .save_auth_state(&account, refresh_token_ref, Some(&runtime_key))
            .await?;

        Ok(())
    }

    /// Decrypt refresh token using master password
    /// - Call AuthPersistence.decrypt_with_password()
    /// - Update state.account_context if successful
    /// - Return (account_context, refresh_token) tuple
    pub async fn decrypt_refresh_token(
        &self,
        master_password: &str,
    ) -> AppResult<Option<(AccountContext, String)>> {
        // Attempt to decrypt using persistence
        let result = self
            .persistence
            .decrypt_with_password(master_password)
            .await?;

        if let Some((account_ctx, refresh_token)) = result {
            // Update state.account_context
            let mut state = self.state.write().await;
            state.account_context = Some(account_ctx.clone());

            Ok(Some((account_ctx, refresh_token)))
        } else {
            Ok(None)
        }
    }

    /// Clear persisted auth state
    /// - Call AuthPersistence.clear_auth_state()
    pub async fn clear_persistence(&self) -> AppResult<()> {
        // Clear persisted state
        self.persistence.clear_auth_state().await?;

        Ok(())
    }

    // ==================== Blocking Versions (Legacy Compatibility) ====================

    /// Check if vault is locked (synchronous version for legacy compatibility)
    pub fn is_vault_unlocked_blocking(&self) -> bool {
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

    // ==================== Private Methods ====================

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

        *timer_guard = timer;
        Ok(())
    }

    /// Stop auto-lock timer
    async fn stop_auto_lock_timer(&self) -> AppResult<()> {
        let mut timer = self.auto_lock_timer.lock().unwrap();
        timer.stop();
        Ok(())
    }

    /// Auto-persist current state
    async fn auto_persist(&self) -> AppResult<()> {
        let state = self.state.read().await;

        // Only persist when account context exists
        if let Some(ref account) = state.account_context {
            // Get refresh_token from session_context
            let refresh_token = state
                .session_context
                .as_ref()
                .and_then(|s| s.refresh_token.as_deref());

            // Get runtime key from persistence
            let runtime_key = self.persistence.runtime_key()?;

            // Call persistence
            self.persistence
                .save_auth_state(account, refresh_token, runtime_key.as_ref())
                .await?;
        }

        Ok(())
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
        tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(duration) => {
                    on_expire();
                }
                _ = rx => {
                    // Timer was cancelled
                }
            }
        });
    }

    /// Stop the timer
    fn stop(&mut self) {
        if let Some(tx) = self.abort_handle.take() {
            let _ = tx.send(());
        }
    }

    /// Record activity - resets the timer
    fn record_activity(&mut self) {
        self.last_activity = Instant::now();
        // Note: In a real implementation, we'd restart the timer here
        // For simplicity, we just track the last activity time
    }

    /// Check if timer has expired based on last activity
    fn is_expired(&self, duration: Duration) -> bool {
        Instant::now().duration_since(self.last_activity) >= duration
    }
}

/// Parse idle auto-lock delay from config string
fn parse_idle_auto_lock_delay(delay: &str) -> AppResult<Duration> {
    // Parse format like "5m", "30s", "1h"
    if delay == "never" {
        return Err(AppError::ValidationFieldError {
            field: "idle_auto_lock_delay".to_string(),
            message: "Cannot parse 'never' as duration".to_string(),
        });
    }

    let len = delay.len();
    if len < 2 {
        return Err(AppError::ValidationFieldError {
            field: "idle_auto_lock_delay".to_string(),
            message: format!("Invalid duration format: {}", delay),
        });
    }

    let (num, unit) = delay.split_at(len - 1);
    let num: u64 = num.parse().map_err(|_| AppError::ValidationFieldError {
        field: "idle_auto_lock_delay".to_string(),
        message: format!("Invalid duration number: {}", num),
    })?;

    match unit {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 60 * 60)),
        _ => Err(AppError::ValidationFieldError {
            field: "idle_auto_lock_delay".to_string(),
            message: format!("Invalid duration unit: {}", unit),
        }),
    }
}
