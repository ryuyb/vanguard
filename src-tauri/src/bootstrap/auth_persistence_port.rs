use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use crate::bootstrap::auth_persistence::{
    decrypt_refresh_token, encrypt_refresh_token_with_runtime, PersistedAuthState,
    PersistedAuthStateContext, SessionWrapRuntime,
};

#[cfg(test)]
use crate::bootstrap::auth_persistence::encrypt_refresh_token;
use crate::bootstrap::unlock_state::AccountContext;
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Authentication state persistence interface
#[async_trait]
pub trait AuthPersistence: Send + Sync {
    /// Save authentication state to disk
    async fn save_auth_state(
        &self,
        account: &AccountContext,
        refresh_token: Option<&str>,
        runtime_key: Option<&SessionWrapRuntime>, // Runtime key for encryption
    ) -> AppResult<()>;

    /// Load authentication state from disk
    async fn load_auth_state(&self) -> AppResult<Option<(AccountContext, Option<String>)>>;

    /// Clear authentication state
    async fn clear_auth_state(&self) -> AppResult<()>;

    /// Decrypt refresh token using master password
    async fn decrypt_with_password(
        &self,
        master_password: &str,
    ) -> AppResult<Option<(AccountContext, String)>>;

    /// Get runtime key if cached
    fn runtime_key(&self) -> AppResult<Option<SessionWrapRuntime>>;

    /// Set runtime key
    fn set_runtime_key(&self, runtime_key: Option<SessionWrapRuntime>) -> AppResult<()>;
}

/// Implementation of AuthPersistence
pub struct AuthPersistenceService {
    auth_states_dir: Arc<PathBuf>,
    runtime_key: Arc<std::sync::Mutex<Option<SessionWrapRuntime>>>,
}

impl AuthPersistenceService {
    pub fn new(auth_states_dir: PathBuf) -> Self {
        Self {
            auth_states_dir: Arc::new(auth_states_dir),
            runtime_key: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Build AccountContext from persisted state
    fn build_account_context(persisted: &PersistedAuthState) -> AccountContext {
        AccountContext {
            account_id: persisted.account_id.clone(),
            email: persisted.email.clone(),
            base_url: persisted.base_url.clone(),
            kdf: persisted.kdf,
            kdf_iterations: persisted.kdf_iterations,
            kdf_memory: persisted.kdf_memory,
            kdf_parallelism: persisted.kdf_parallelism,
        }
    }

    /// Build persisted context from AccountContext
    fn build_persisted_context(account: &AccountContext) -> PersistedAuthStateContext {
        PersistedAuthStateContext {
            account_id: account.account_id.clone(),
            base_url: account.base_url.clone(),
            email: account.email.clone(),
            kdf: account.kdf,
            kdf_iterations: account.kdf_iterations,
            kdf_memory: account.kdf_memory,
            kdf_parallelism: account.kdf_parallelism,
        }
    }

    /// Load active persisted auth state
    async fn load_active_persisted_auth_state(&self) -> AppResult<Option<PersistedAuthState>> {
        let active_path = self.auth_states_dir.join("active.json");
        if !active_path.exists() {
            return Ok(None);
        }

        let active_raw = fs::read_to_string(&active_path).await.map_err(|error| {
            AppError::InternalUnexpected {
                message: format!(
                    "failed to read active account index {}: {error}",
                    active_path.display()
                ),
            }
        })?;

        let active_data: serde_json::Value =
            serde_json::from_str(&active_raw).map_err(|error| AppError::InternalUnexpected {
                message: format!(
                    "failed to parse active account index {}: {error}",
                    active_path.display()
                ),
            })?;

        let account_id = active_data
            .get("accountId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InternalUnexpected {
                message: format!(
                    "active account index missing accountId field: {}",
                    active_path.display()
                ),
            })?;

        self.load_persisted_auth_state_from_disk(account_id).await
    }

    /// Load persisted auth state for a specific account from disk
    async fn load_persisted_auth_state_from_disk(
        &self,
        account_id: &str,
    ) -> AppResult<Option<PersistedAuthState>> {
        let safe_name = sanitize_account_id_for_filename(account_id);
        let account_path = self.auth_states_dir.join(format!("{}.json", safe_name));
        if !account_path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&account_path).await.map_err(|error| {
            AppError::InternalUnexpected {
                message: format!(
                    "failed to read persisted auth state {}: {error}",
                    account_path.display()
                ),
            }
        })?;
        let parsed = serde_json::from_str::<PersistedAuthState>(&raw).map_err(|error| {
            AppError::InternalUnexpected {
                message: format!(
                    "failed to parse persisted auth state {}: {error}",
                    account_path.display()
                ),
            }
        })?;
        Ok(Some(parsed))
    }

    /// Persist auth state to disk
    async fn persist_auth_state_to_disk(
        &self,
        account_id: &str,
        value: Option<&PersistedAuthState>,
    ) -> AppResult<()> {
        let safe_name = sanitize_account_id_for_filename(account_id);
        let account_path = self.auth_states_dir.join(format!("{}.json", safe_name));
        match value {
            None => {
                if account_path.exists() {
                    fs::remove_file(&account_path).await.map_err(|error| {
                        AppError::InternalUnexpected {
                            message: format!(
                                "failed to delete persisted auth state {}: {error}",
                                account_path.display()
                            ),
                        }
                    })?;
                }
                Ok(())
            }
            Some(value) => {
                let serialized = serde_json::to_vec_pretty(value).map_err(|error| {
                    AppError::InternalUnexpected {
                        message: format!(
                            "failed to serialize persisted auth state {}: {error}",
                            account_path.display()
                        ),
                    }
                })?;
                let temp_path = build_temp_auth_state_path(&account_path);
                fs::write(&temp_path, serialized).await.map_err(|error| {
                    AppError::InternalUnexpected {
                        message: format!(
                            "failed to write temp auth state {}: {error}",
                            temp_path.display()
                        ),
                    }
                })?;
                fs::rename(&temp_path, &account_path)
                    .await
                    .map_err(|error| AppError::InternalUnexpected {
                        message: format!(
                            "failed to persist auth state {}: {error}",
                            account_path.display()
                        ),
                    })?;
                Ok(())
            }
        }
    }

    /// Update active account index
    async fn update_active_account_index(&self, account_id: &str) -> AppResult<()> {
        let active_path = self.auth_states_dir.join("active.json");
        let active_data = serde_json::json!({
            "accountId": account_id
        });
        let serialized = serde_json::to_vec_pretty(&active_data).map_err(|error| {
            AppError::InternalUnexpected {
                message: format!(
                    "failed to serialize active account index {}: {error}",
                    active_path.display()
                ),
            }
        })?;
        let temp_path = build_temp_auth_state_path(&active_path);
        fs::write(&temp_path, serialized)
            .await
            .map_err(|error| AppError::InternalUnexpected {
                message: format!(
                    "failed to write temp active account index {}: {error}",
                    temp_path.display()
                ),
            })?;
        fs::rename(&temp_path, &active_path)
            .await
            .map_err(|error| AppError::InternalUnexpected {
                message: format!(
                    "failed to persist active account index {}: {error}",
                    active_path.display()
                ),
            })?;
        Ok(())
    }

    /// Remove active account index
    async fn remove_active_account_index(&self) -> AppResult<()> {
        let active_path = self.auth_states_dir.join("active.json");
        if active_path.exists() {
            fs::remove_file(&active_path)
                .await
                .map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to remove active account index {}: {error}",
                        active_path.display()
                    ),
                })?;
        }
        Ok(())
    }

    /// List all remaining account IDs
    async fn list_remaining_account_ids(&self) -> AppResult<Vec<String>> {
        let mut account_ids = Vec::new();
        let mut entries = fs::read_dir(&*self.auth_states_dir)
            .await
            .map_err(|error| AppError::InternalUnexpected {
                message: format!(
                    "failed to read auth states dir {}: {error}",
                    self.auth_states_dir.display()
                ),
            })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|error| AppError::InternalUnexpected {
                    message: format!(
                        "failed to read dir entry in {}: {error}",
                        self.auth_states_dir.display()
                    ),
                })?
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name != "active.json" && file_name.ends_with(".json") {
                        if let Some(sanitized_id) = file_name.strip_suffix(".json") {
                            account_ids.push(unsanitize_account_id_from_filename(sanitized_id));
                        }
                    }
                }
            }
        }

        Ok(account_ids)
    }

    /// Refresh active account after logout
    async fn refresh_active_account_after_logout(
        &self,
        logged_out_account_id: &str,
    ) -> AppResult<()> {
        let remaining_accounts = self.list_remaining_account_ids().await?;

        if remaining_accounts.is_empty() {
            self.remove_active_account_index().await?;
        } else {
            // Pick the first remaining account as the new active account
            let new_active = &remaining_accounts[0];
            if new_active != logged_out_account_id {
                self.update_active_account_index(new_active).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl AuthPersistence for AuthPersistenceService {
    async fn save_auth_state(
        &self,
        account: &AccountContext,
        refresh_token: Option<&str>,
        runtime_key: Option<&SessionWrapRuntime>,
    ) -> AppResult<()> {
        // If no refresh_token, only save account context (unencrypted)
        let persisted = if let Some(token) = refresh_token {
            if let Some(runtime) = runtime_key {
                // Re-encrypt using existing runtime
                let encrypted_session = encrypt_refresh_token_with_runtime(
                    runtime,
                    &account.account_id,
                    &account.base_url,
                    &account.email,
                    token,
                )?;
                Some(PersistedAuthState::new(
                    Self::build_persisted_context(account),
                    encrypted_session,
                )?)
            } else {
                // No runtime available, cannot encrypt
                return Err(AppError::ValidationFieldError {
                    field: "unknown".to_string(),
                    message: "runtime_key is required to encrypt refresh token".to_string(),
                });
            }
        } else {
            // No refresh_token, don't save encrypted session
            None
        };

        // Save to disk
        if let Some(ref state) = persisted {
            self.persist_auth_state_to_disk(&account.account_id, Some(state))
                .await?;
            self.update_active_account_index(&account.account_id)
                .await?;
        } else {
            // Clear persisted state
            self.persist_auth_state_to_disk(&account.account_id, None)
                .await?;
        }

        Ok(())
    }

    async fn load_auth_state(&self) -> AppResult<Option<(AccountContext, Option<String>)>> {
        let persisted = match self.load_active_persisted_auth_state().await? {
            Some(value) => value,
            None => return Ok(None),
        };

        let account_ctx = Self::build_account_context(&persisted);

        // Try to decrypt refresh_token using cached runtime_key
        let runtime_guard = self
            .runtime_key
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock runtime_key".to_string(),
            })?;

        let refresh_token = if let Some(ref runtime) = *runtime_guard {
            match decrypt_refresh_token_with_runtime(runtime, &persisted) {
                Ok(token) => Some(token),
                Err(e) => {
                    log::warn!(
                        target: "vanguard::auth_persistence",
                        "Failed to decrypt refresh token with cached runtime: {}",
                        e.log_message()
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(Some((account_ctx, refresh_token)))
    }

    async fn clear_auth_state(&self) -> AppResult<()> {
        // Get current active account
        let persisted = self.load_active_persisted_auth_state().await?;

        if let Some(ref state) = persisted {
            // Clear persisted state for this account
            self.persist_auth_state_to_disk(&state.account_id, None)
                .await?;
            // Refresh active account index
            self.refresh_active_account_after_logout(&state.account_id)
                .await?;
        }

        // Clear runtime key
        let mut runtime_guard =
            self.runtime_key
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock runtime_key".to_string(),
                })?;
        *runtime_guard = None;

        Ok(())
    }

    async fn decrypt_with_password(
        &self,
        master_password: &str,
    ) -> AppResult<Option<(AccountContext, String)>> {
        let persisted = match self.load_active_persisted_auth_state().await? {
            Some(value) => value,
            None => return Ok(None),
        };

        let (refresh_token, runtime) = decrypt_refresh_token(
            master_password,
            &persisted.account_id,
            &persisted.base_url,
            &persisted.email,
            &persisted.encrypted_session,
        )?;

        // Cache runtime_key
        let mut runtime_guard =
            self.runtime_key
                .lock()
                .map_err(|_| AppError::InternalUnexpected {
                    message: "failed to lock runtime_key".to_string(),
                })?;
        *runtime_guard = Some(runtime);

        let account_ctx = Self::build_account_context(&persisted);
        Ok(Some((account_ctx, refresh_token)))
    }

    fn runtime_key(&self) -> AppResult<Option<SessionWrapRuntime>> {
        let guard = self
            .runtime_key
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock runtime_key".to_string(),
            })?;
        Ok(guard.clone())
    }

    fn set_runtime_key(&self, runtime_key: Option<SessionWrapRuntime>) -> AppResult<()> {
        let mut guard = self
            .runtime_key
            .lock()
            .map_err(|_| AppError::InternalUnexpected {
                message: "failed to lock runtime_key".to_string(),
            })?;
        *guard = runtime_key;
        Ok(())
    }
}

/// Decrypt refresh token using runtime (without re-deriving key)
/// Note: This function requires account_id, base_url, email from PersistedAuthState to build correct AAD
fn decrypt_refresh_token_with_runtime(
    runtime: &SessionWrapRuntime,
    persisted: &PersistedAuthState,
) -> AppResult<String> {
    use base64::engine::general_purpose::STANDARD_NO_PAD;
    use base64::Engine;
    use chacha20poly1305::aead::{Aead, Payload};
    use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

    const WRAP_NONCE_LEN: usize = 24;
    const AUTH_STATE_VERSION: u8 = 1;

    let nonce = decode_fixed_len(
        &persisted.encrypted_session.nonce_b64,
        WRAP_NONCE_LEN,
        "encrypted_session.nonce_b64",
    )?;
    let ciphertext = STANDARD_NO_PAD
        .decode(persisted.encrypted_session.ciphertext_b64.as_bytes())
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "encrypted_session.ciphertext_b64 is not valid base64".to_string(),
        })?;

    // Use the same AAD format as encryption
    let aad = format!(
        "vanguard:auth-state:v{AUTH_STATE_VERSION}:{}:{}:{}",
        persisted.account_id,
        persisted.base_url.trim().to_lowercase(),
        persisted.email.trim().to_lowercase()
    );

    let cipher = XChaCha20Poly1305::new((runtime.key()).into());
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: ciphertext.as_slice(),
                aad: aad.as_bytes(),
            },
        )
        .map_err(|_| AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: "failed to decrypt persisted session with cached runtime".to_string(),
        })?;

    String::from_utf8(plaintext).map_err(|error| AppError::ValidationFieldError {
        field: "unknown".to_string(),
        message: format!("decrypted session is not utf-8: {error}"),
    })
}

fn decode_fixed_len(value: &str, expected_len: usize, field_name: &str) -> AppResult<Vec<u8>> {
    use base64::engine::general_purpose::STANDARD_NO_PAD;
    use base64::Engine;

    let decoded =
        STANDARD_NO_PAD
            .decode(value.as_bytes())
            .map_err(|_| AppError::ValidationFieldError {
                field: "unknown".to_string(),
                message: format!("{field_name} is not valid base64"),
            })?;
    if decoded.len() != expected_len {
        return Err(AppError::ValidationFieldError {
            field: "unknown".to_string(),
            message: format!(
                "{field_name} must decode to {expected_len} bytes, got {}",
                decoded.len()
            ),
        });
    }
    Ok(decoded)
}

fn sanitize_account_id_for_filename(account_id: &str) -> String {
    account_id
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace(':', "%3A")
        .replace('\\', "%5C")
}

fn unsanitize_account_id_from_filename(sanitized: &str) -> String {
    sanitized
        .replace("%5C", "\\")
        .replace("%3A", ":")
        .replace("%2F", "/")
        .replace("%25", "%")
}

fn build_temp_auth_state_path(path: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static AUTH_STATE_TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("auth-state.json");
    let temp_file_name = format!(
        "{file_name}.tmp.{}.{}",
        std::process::id(),
        AUTH_STATE_TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    match path.parent() {
        Some(parent) => parent.join(temp_file_name),
        None => PathBuf::from(temp_file_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_account_context() -> AccountContext {
        AccountContext {
            account_id: "https://vault.example::user".to_string(),
            email: "user@example.com".to_string(),
            base_url: "https://vault.example".to_string(),
            kdf: Some(0),
            kdf_iterations: Some(100000),
            kdf_memory: None,
            kdf_parallelism: None,
        }
    }

    #[tokio::test]
    async fn test_save_and_load_auth_state() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let service = AuthPersistenceService::new(temp_dir.path().to_path_buf());

        let account = create_test_account_context();
        let master_password = "test-password";
        let refresh_token = "test-refresh-token";

        // First encrypt to get runtime_key
        let (_, runtime) = encrypt_refresh_token(
            master_password,
            &account.account_id,
            &account.base_url,
            &account.email,
            refresh_token,
        )
        .expect("encrypt");

        // Save auth state
        service
            .save_auth_state(&account, Some(refresh_token), Some(&runtime))
            .await
            .expect("save auth state");

        // Set runtime_key for decryption
        service
            .set_runtime_key(Some(runtime))
            .expect("set runtime key");

        // Load auth state
        let loaded = service.load_auth_state().await.expect("load auth state");
        assert!(loaded.is_some());

        let (loaded_account, loaded_token) = loaded.unwrap();
        assert_eq!(loaded_account.account_id, account.account_id);
        assert_eq!(loaded_account.email, account.email);
        assert_eq!(loaded_token, Some(refresh_token.to_string()));
    }

    #[tokio::test]
    async fn test_decrypt_with_password() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let service = AuthPersistenceService::new(temp_dir.path().to_path_buf());

        let account = create_test_account_context();
        let master_password = "test-password";
        let refresh_token = "test-refresh-token";

        // First encrypt to get runtime_key
        let (_, runtime) = encrypt_refresh_token(
            master_password,
            &account.account_id,
            &account.base_url,
            &account.email,
            refresh_token,
        )
        .expect("encrypt");

        // Save auth state
        service
            .save_auth_state(&account, Some(refresh_token), Some(&runtime))
            .await
            .expect("save auth state");

        // Decrypt with password
        let decrypted = service
            .decrypt_with_password(master_password)
            .await
            .expect("decrypt");
        assert!(decrypted.is_some());

        let (decrypted_account, decrypted_token) = decrypted.unwrap();
        assert_eq!(decrypted_account.account_id, account.account_id);
        assert_eq!(decrypted_token, refresh_token);
    }

    #[tokio::test]
    async fn test_clear_auth_state() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let service = AuthPersistenceService::new(temp_dir.path().to_path_buf());

        let account = create_test_account_context();
        let master_password = "test-password";
        let refresh_token = "test-refresh-token";

        // First encrypt to get runtime_key
        let (_, runtime) = encrypt_refresh_token(
            master_password,
            &account.account_id,
            &account.base_url,
            &account.email,
            refresh_token,
        )
        .expect("encrypt");

        // Save auth state
        service
            .save_auth_state(&account, Some(refresh_token), Some(&runtime))
            .await
            .expect("save auth state");

        // Set runtime_key
        service
            .set_runtime_key(Some(runtime))
            .expect("set runtime key");

        // Verify state exists
        let loaded = service.load_auth_state().await.expect("load auth state");
        assert!(loaded.is_some());

        // Clear state
        service.clear_auth_state().await.expect("clear auth state");

        // Verify state is cleared
        let loaded_after = service.load_auth_state().await.expect("load after clear");
        assert!(loaded_after.is_none());

        // Verify runtime_key is cleared
        assert!(service.runtime_key().expect("get runtime key").is_none());
    }
}
