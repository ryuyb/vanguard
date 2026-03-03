use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;

use crate::application::dto::vault::{
    EnablePinUnlockCommand, UnlockVaultResult, VaultPinStatus, VaultUserKeyMaterial,
};
use crate::application::ports::pin_unlock_port::PinUnlockPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::unlock_vault_use_case::PinUnlockExecutor;
use crate::application::vault_crypto;
use crate::domain::unlock::{PinLockType, PinProtectedUserKeyEnvelope};
use crate::support::error::AppError;
use crate::support::result::AppResult;

const PIN_ENVELOPE_ALGORITHM: &str = "xchacha20poly1305";
const PIN_ENVELOPE_KDF: &str = "argon2id-v1";
const PIN_SALT_LEN: usize = 16;
const PIN_NONCE_LEN: usize = 24;
const PIN_DERIVED_KEY_LEN: usize = 32;
const PIN_KDF_MEMORY_KIB: u32 = 32 * 1024;
const PIN_KDF_ITERATIONS: u32 = 3;
const PIN_KDF_PARALLELISM: u32 = 1;

#[derive(Clone)]
pub struct VaultPinUseCase {
    pin_unlock_port: Arc<dyn PinUnlockPort>,
}

impl VaultPinUseCase {
    pub fn new(pin_unlock_port: Arc<dyn PinUnlockPort>) -> Self {
        Self { pin_unlock_port }
    }

    pub async fn pin_status(&self, runtime: &dyn VaultRuntimePort) -> AppResult<VaultPinStatus> {
        if !self.pin_unlock_port.is_supported() {
            return Ok(VaultPinStatus {
                supported: false,
                enabled: false,
                lock_type: PinLockType::Disabled,
            });
        }

        let account_id = match runtime.active_account_id() {
            Ok(value) => value,
            Err(AppError::Validation(_)) => {
                return Ok(VaultPinStatus {
                    supported: true,
                    enabled: false,
                    lock_type: PinLockType::Disabled,
                });
            }
            Err(error) => return Err(error),
        };

        let lock_type = self.resolve_enabled_lock_type(&account_id).await?;
        Ok(VaultPinStatus {
            supported: true,
            enabled: lock_type != PinLockType::Disabled,
            lock_type,
        })
    }

    pub async fn enable_pin_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
        command: EnablePinUnlockCommand,
    ) -> AppResult<()> {
        self.ensure_supported()?;
        if command.lock_type == PinLockType::Disabled {
            return Err(AppError::validation(
                "pin lock type cannot be disabled when enabling pin unlock",
            ));
        }

        let pin = command.pin.trim().to_string();
        if pin.is_empty() {
            return Err(AppError::validation("pin cannot be empty"));
        }

        let account_id = runtime.active_account_id()?;
        let user_key = runtime
            .get_vault_user_key_material(&account_id)?
            .ok_or_else(|| {
                AppError::validation(
                    "vault is locked, please unlock with password or biometric before enabling pin",
                )
            })?;

        let envelope = encrypt_user_key_with_pin(&pin, &user_key)?;
        self.pin_unlock_port
            .save_pin_envelope(&account_id, command.lock_type, &envelope)
            .await?;

        let other_lock_type = opposite_lock_type(command.lock_type);
        self.pin_unlock_port
            .delete_pin_envelope(&account_id, other_lock_type)
            .await?;

        log::info!(
            target: "vanguard::application::vault_pin",
            "pin unlock enabled account_id={} lock_type={}",
            account_id,
            pin_lock_type_name(command.lock_type)
        );

        Ok(())
    }

    pub async fn disable_pin_unlock(&self, runtime: &dyn VaultRuntimePort) -> AppResult<()> {
        if !self.pin_unlock_port.is_supported() {
            return Ok(());
        }

        let account_id = match runtime.active_account_id() {
            Ok(value) => value,
            Err(AppError::Validation(_)) => return Ok(()),
            Err(error) => return Err(error),
        };

        self.pin_unlock_port
            .delete_pin_envelope(&account_id, PinLockType::Persistent)
            .await?;
        self.pin_unlock_port
            .delete_pin_envelope(&account_id, PinLockType::Ephemeral)
            .await?;

        log::info!(
            target: "vanguard::application::vault_pin",
            "pin unlock disabled account_id={}",
            account_id,
        );
        Ok(())
    }

    async fn resolve_enabled_lock_type(&self, account_id: &str) -> AppResult<PinLockType> {
        if self
            .pin_unlock_port
            .has_pin_envelope(account_id, PinLockType::Persistent)
            .await?
        {
            return Ok(PinLockType::Persistent);
        }

        if self
            .pin_unlock_port
            .has_pin_envelope(account_id, PinLockType::Ephemeral)
            .await?
        {
            return Ok(PinLockType::Ephemeral);
        }

        Ok(PinLockType::Disabled)
    }

    fn ensure_supported(&self) -> AppResult<()> {
        if !self.pin_unlock_port.is_supported() {
            return Err(AppError::validation(
                "pin unlock is only supported on macOS",
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl PinUnlockExecutor for VaultPinUseCase {
    async fn execute_pin_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
        pin: String,
    ) -> AppResult<UnlockVaultResult> {
        self.ensure_supported()?;

        let account_id = runtime.active_account_id()?;
        let lock_type = self.resolve_enabled_lock_type(&account_id).await?;
        if lock_type == PinLockType::Disabled {
            return Err(AppError::validation(
                "pin unlock is not configured for this account",
            ));
        }

        let envelope = self
            .pin_unlock_port
            .load_pin_envelope(&account_id, lock_type)
            .await?;
        let user_key = decrypt_user_key_with_pin(pin.trim(), &envelope)?;
        runtime.set_vault_user_key_material(account_id.clone(), user_key)?;

        log::info!(
            target: "vanguard::application::vault_pin",
            "vault unlocked with pin account_id={} lock_type={}",
            account_id,
            pin_lock_type_name(lock_type)
        );

        Ok(UnlockVaultResult { account_id })
    }
}

fn encrypt_user_key_with_pin(
    pin: &str,
    user_key: &VaultUserKeyMaterial,
) -> AppResult<PinProtectedUserKeyEnvelope> {
    let trimmed_pin = pin.trim();
    if trimmed_pin.is_empty() {
        return Err(AppError::validation("pin cannot be empty"));
    }

    let plaintext_user_key = serialize_user_key(user_key)?;

    let mut salt = [0u8; PIN_SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let derived_key = derive_pin_key(trimmed_pin, &salt)?;

    let mut nonce = [0u8; PIN_NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);

    let cipher = XChaCha20Poly1305::new_from_slice(&derived_key)
        .map_err(|_| AppError::internal("failed to initialize pin envelope cipher"))?;
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext_user_key.as_ref())
        .map_err(|_| AppError::internal("failed to encrypt pin envelope"))?;

    Ok(PinProtectedUserKeyEnvelope {
        algorithm: String::from(PIN_ENVELOPE_ALGORITHM),
        kdf: String::from(PIN_ENVELOPE_KDF),
        salt_b64: STANDARD_NO_PAD.encode(salt),
        nonce_b64: STANDARD_NO_PAD.encode(nonce),
        ciphertext_b64: STANDARD_NO_PAD.encode(ciphertext),
    })
}

fn decrypt_user_key_with_pin(
    pin: &str,
    envelope: &PinProtectedUserKeyEnvelope,
) -> AppResult<VaultUserKeyMaterial> {
    if envelope.algorithm != PIN_ENVELOPE_ALGORITHM {
        return Err(AppError::validation(
            "pin envelope algorithm is unsupported or legacy",
        ));
    }
    if envelope.kdf != PIN_ENVELOPE_KDF {
        return Err(AppError::validation(
            "pin envelope kdf is unsupported or legacy",
        ));
    }

    let salt = vault_crypto::decode_base64_flexible(&envelope.salt_b64, "pin.salt_b64")?;
    if salt.len() != PIN_SALT_LEN {
        return Err(AppError::validation(format!(
            "pin envelope salt length must be {PIN_SALT_LEN} bytes"
        )));
    }

    let nonce = vault_crypto::decode_base64_flexible(&envelope.nonce_b64, "pin.nonce_b64")?;
    if nonce.len() != PIN_NONCE_LEN {
        return Err(AppError::validation(format!(
            "pin envelope nonce length must be {PIN_NONCE_LEN} bytes"
        )));
    }

    let ciphertext =
        vault_crypto::decode_base64_flexible(&envelope.ciphertext_b64, "pin.ciphertext_b64")?;
    if ciphertext.is_empty() {
        return Err(AppError::validation(
            "pin envelope ciphertext cannot be empty",
        ));
    }

    let derived_key = derive_pin_key(pin, &salt)?;
    let cipher = XChaCha20Poly1305::new_from_slice(&derived_key)
        .map_err(|_| AppError::internal("failed to initialize pin envelope cipher"))?;
    let plaintext = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| AppError::validation("invalid pin or corrupted pin envelope"))?;

    vault_crypto::parse_user_key_material(&plaintext)
}

fn serialize_user_key(user_key: &VaultUserKeyMaterial) -> AppResult<Vec<u8>> {
    vault_crypto::validate_key_lengths(&user_key.enc_key, user_key.mac_key.as_deref())?;

    let mut output = Vec::with_capacity(64);
    output.extend_from_slice(&user_key.enc_key);
    if let Some(mac_key) = &user_key.mac_key {
        output.extend_from_slice(mac_key);
    }
    Ok(output)
}

fn derive_pin_key(pin: &str, salt: &[u8]) -> AppResult<[u8; PIN_DERIVED_KEY_LEN]> {
    if pin.trim().is_empty() {
        return Err(AppError::validation("pin cannot be empty"));
    }

    let params = Params::new(
        PIN_KDF_MEMORY_KIB,
        PIN_KDF_ITERATIONS,
        PIN_KDF_PARALLELISM,
        Some(PIN_DERIVED_KEY_LEN),
    )
    .map_err(|error| AppError::internal(format!("invalid pin argon2 params: {error}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut derived = [0u8; PIN_DERIVED_KEY_LEN];
    argon2
        .hash_password_into(pin.as_bytes(), salt, &mut derived)
        .map_err(|_| AppError::validation("failed to derive pin key"))?;
    Ok(derived)
}

fn opposite_lock_type(lock_type: PinLockType) -> PinLockType {
    match lock_type {
        PinLockType::Persistent => PinLockType::Ephemeral,
        PinLockType::Ephemeral => PinLockType::Persistent,
        PinLockType::Disabled => PinLockType::Disabled,
    }
}

fn pin_lock_type_name(lock_type: PinLockType) -> &'static str {
    match lock_type {
        PinLockType::Disabled => "disabled",
        PinLockType::Ephemeral => "ephemeral",
        PinLockType::Persistent => "persistent",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::VaultPinUseCase;
    use crate::application::dto::vault::{
        EnablePinUnlockCommand, VaultUnlockContext, VaultUserKeyMaterial,
    };
    use crate::application::ports::pin_unlock_port::PinUnlockPort;
    use crate::application::ports::vault_runtime_port::VaultRuntimePort;
    use crate::application::use_cases::unlock_vault_use_case::PinUnlockExecutor;
    use crate::domain::unlock::{PinLockType, PinProtectedUserKeyEnvelope};
    use crate::support::error::AppError;
    use crate::support::result::AppResult;

    #[derive(Default)]
    struct FakeRuntime {
        user_key: Mutex<Option<VaultUserKeyMaterial>>,
    }

    impl FakeRuntime {
        fn with_user_key(user_key: Option<VaultUserKeyMaterial>) -> Self {
            Self {
                user_key: Mutex::new(user_key),
            }
        }
    }

    impl VaultRuntimePort for FakeRuntime {
        fn active_account_id(&self) -> AppResult<String> {
            Ok(String::from("account-1"))
        }

        fn auth_session_context(&self) -> AppResult<Option<VaultUnlockContext>> {
            Ok(None)
        }

        fn persisted_auth_context(&self) -> AppResult<Option<VaultUnlockContext>> {
            Ok(None)
        }

        fn get_vault_user_key_material(
            &self,
            _account_id: &str,
        ) -> AppResult<Option<VaultUserKeyMaterial>> {
            Ok(self.user_key.lock().expect("user_key lock").clone())
        }

        fn set_vault_user_key_material(
            &self,
            _account_id: String,
            key: VaultUserKeyMaterial,
        ) -> AppResult<()> {
            *self.user_key.lock().expect("user_key lock") = Some(key);
            Ok(())
        }

        fn remove_vault_user_key_material(&self, _account_id: &str) -> AppResult<()> {
            *self.user_key.lock().expect("user_key lock") = None;
            Ok(())
        }
    }

    #[derive(Clone)]
    struct FakePinUnlockPort {
        supported: bool,
        persistent: Arc<Mutex<HashMap<String, PinProtectedUserKeyEnvelope>>>,
        ephemeral: Arc<Mutex<HashMap<String, PinProtectedUserKeyEnvelope>>>,
    }

    impl FakePinUnlockPort {
        fn new_with_shared_persistent(
            supported: bool,
            persistent: Arc<Mutex<HashMap<String, PinProtectedUserKeyEnvelope>>>,
        ) -> Self {
            Self {
                supported,
                persistent,
                ephemeral: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl PinUnlockPort for FakePinUnlockPort {
        fn is_supported(&self) -> bool {
            self.supported
        }

        async fn save_pin_envelope(
            &self,
            account_id: &str,
            lock_type: PinLockType,
            envelope: &PinProtectedUserKeyEnvelope,
        ) -> AppResult<()> {
            match lock_type {
                PinLockType::Persistent => {
                    self.persistent
                        .lock()
                        .expect("persistent lock")
                        .insert(String::from(account_id), envelope.clone());
                }
                PinLockType::Ephemeral => {
                    self.ephemeral
                        .lock()
                        .expect("ephemeral lock")
                        .insert(String::from(account_id), envelope.clone());
                }
                PinLockType::Disabled => {
                    return Err(AppError::validation("invalid lock type for save"));
                }
            }
            Ok(())
        }

        async fn load_pin_envelope(
            &self,
            account_id: &str,
            lock_type: PinLockType,
        ) -> AppResult<PinProtectedUserKeyEnvelope> {
            let value = match lock_type {
                PinLockType::Persistent => self
                    .persistent
                    .lock()
                    .expect("persistent lock")
                    .get(account_id)
                    .cloned(),
                PinLockType::Ephemeral => self
                    .ephemeral
                    .lock()
                    .expect("ephemeral lock")
                    .get(account_id)
                    .cloned(),
                PinLockType::Disabled => None,
            };
            value.ok_or_else(|| AppError::validation("pin envelope not found"))
        }

        async fn has_pin_envelope(
            &self,
            account_id: &str,
            lock_type: PinLockType,
        ) -> AppResult<bool> {
            Ok(match lock_type {
                PinLockType::Persistent => self
                    .persistent
                    .lock()
                    .expect("persistent lock")
                    .contains_key(account_id),
                PinLockType::Ephemeral => self
                    .ephemeral
                    .lock()
                    .expect("ephemeral lock")
                    .contains_key(account_id),
                PinLockType::Disabled => false,
            })
        }

        async fn delete_pin_envelope(
            &self,
            account_id: &str,
            lock_type: PinLockType,
        ) -> AppResult<()> {
            match lock_type {
                PinLockType::Persistent => {
                    self.persistent
                        .lock()
                        .expect("persistent lock")
                        .remove(account_id);
                }
                PinLockType::Ephemeral => {
                    self.ephemeral
                        .lock()
                        .expect("ephemeral lock")
                        .remove(account_id);
                }
                PinLockType::Disabled => {}
            }
            Ok(())
        }
    }

    fn sample_user_key() -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: vec![3u8; 32],
            mac_key: Some(vec![7u8; 32]),
        }
    }

    #[tokio::test]
    async fn enable_pin_requires_unlocked_vault() {
        let persistent = Arc::new(Mutex::new(HashMap::new()));
        let pin_port = Arc::new(FakePinUnlockPort::new_with_shared_persistent(
            true, persistent,
        ));
        let use_case = VaultPinUseCase::new(pin_port);

        let error = use_case
            .enable_pin_unlock(
                &FakeRuntime::with_user_key(None),
                EnablePinUnlockCommand {
                    pin: String::from("123456"),
                    lock_type: PinLockType::Persistent,
                },
            )
            .await
            .expect_err("enabling pin while locked should fail");

        match error {
            AppError::Validation(message) => {
                assert!(message.contains("vault is locked"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn ephemeral_pin_is_lost_after_restart_simulation() {
        let persistent = Arc::new(Mutex::new(HashMap::new()));
        let runtime = FakeRuntime::with_user_key(Some(sample_user_key()));

        let use_case_a = VaultPinUseCase::new(Arc::new(
            FakePinUnlockPort::new_with_shared_persistent(true, Arc::clone(&persistent)),
        ));
        use_case_a
            .enable_pin_unlock(
                &runtime,
                EnablePinUnlockCommand {
                    pin: String::from("123456"),
                    lock_type: PinLockType::Ephemeral,
                },
            )
            .await
            .expect("enable ephemeral pin");

        let status_a = use_case_a.pin_status(&runtime).await.expect("status_a");
        assert!(status_a.enabled);
        assert_eq!(status_a.lock_type, PinLockType::Ephemeral);

        runtime
            .remove_vault_user_key_material("account-1")
            .expect("lock vault before unlock test");
        use_case_a
            .execute_pin_unlock(&runtime, String::from("123456"))
            .await
            .expect("ephemeral pin unlock before restart");

        runtime
            .remove_vault_user_key_material("account-1")
            .expect("lock vault before restart");
        let use_case_b = VaultPinUseCase::new(Arc::new(
            FakePinUnlockPort::new_with_shared_persistent(true, persistent),
        ));

        let status_b = use_case_b.pin_status(&runtime).await.expect("status_b");
        assert!(!status_b.enabled);
        assert_eq!(status_b.lock_type, PinLockType::Disabled);

        let error = use_case_b
            .execute_pin_unlock(&runtime, String::from("123456"))
            .await
            .expect_err("ephemeral pin should be unavailable after restart");
        match error {
            AppError::Validation(message) => {
                assert!(message.contains("not configured"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn persistent_pin_survives_restart_and_unlocks_vault() {
        let persistent = Arc::new(Mutex::new(HashMap::new()));
        let runtime = FakeRuntime::with_user_key(Some(sample_user_key()));

        let use_case_a = VaultPinUseCase::new(Arc::new(
            FakePinUnlockPort::new_with_shared_persistent(true, Arc::clone(&persistent)),
        ));
        use_case_a
            .enable_pin_unlock(
                &runtime,
                EnablePinUnlockCommand {
                    pin: String::from("654321"),
                    lock_type: PinLockType::Persistent,
                },
            )
            .await
            .expect("enable persistent pin");

        runtime
            .remove_vault_user_key_material("account-1")
            .expect("lock vault");

        let use_case_b = VaultPinUseCase::new(Arc::new(
            FakePinUnlockPort::new_with_shared_persistent(true, persistent),
        ));
        let status = use_case_b.pin_status(&runtime).await.expect("status");
        assert!(status.enabled);
        assert_eq!(status.lock_type, PinLockType::Persistent);

        let result = use_case_b
            .execute_pin_unlock(&runtime, String::from("654321"))
            .await
            .expect("persistent pin unlock");

        assert_eq!(result.account_id, "account-1");
        assert!(runtime
            .get_vault_user_key_material("account-1")
            .expect("read user key")
            .is_some());
    }
}
