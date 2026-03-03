use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac_array;
use sha2::{Digest, Sha256};

use crate::application::dto::vault::{UnlockVaultResult, VaultUnlockContext, VaultUserKeyMaterial};
use crate::application::ports::master_password_unlock_data_port::MasterPasswordUnlockDataPort;
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::use_cases::unlock_vault_use_case::MasterPasswordUnlockExecutor;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

type HmacSha256 = Hmac<Sha256>;

const KDF_PBKDF2: i32 = 0;
const KDF_ARGON2ID: i32 = 1;
const MASTER_KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct MasterPasswordUnlockUseCase {
    master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>,
}

impl MasterPasswordUnlockUseCase {
    pub fn new(master_password_unlock_data_port: Arc<dyn MasterPasswordUnlockDataPort>) -> Self {
        Self {
            master_password_unlock_data_port,
        }
    }
}

#[async_trait]
impl MasterPasswordUnlockExecutor for MasterPasswordUnlockUseCase {
    async fn execute_master_password_unlock(
        &self,
        runtime: &dyn VaultRuntimePort,
        master_password: String,
    ) -> AppResult<UnlockVaultResult> {
        let master_password = master_password.trim().to_string();
        if master_password.is_empty() {
            return Err(AppError::validation("master_password cannot be empty"));
        }

        let unlock_context = resolve_unlock_context(runtime)?;
        let unlock_data = self
            .master_password_unlock_data_port
            .load_master_password_unlock_data(&unlock_context.account_id)
            .await?
            .ok_or_else(|| {
                AppError::validation("missing canonical master_password_unlock data in local vault metadata")
            })?;

        let master_key = derive_master_key(
            &unlock_data.salt,
            &master_password,
            unlock_data.kdf.kdf_type,
            unlock_data.kdf.iterations,
            unlock_data.kdf.memory,
            unlock_data.kdf.parallelism,
        )
        .map_err(|error| {
            AppError::validation(format!(
                "failed to derive master key with canonical kdf params: {error}"
            ))
        })?;

        let wrapping_key = derive_wrapping_key_material(&master_key)?;
        let plaintext_user_key = vault_crypto::decrypt_cipher_bytes(
            unlock_data.master_key_wrapped_user_key.trim(),
            &wrapping_key,
        )
        .map_err(|error| {
            AppError::validation(format!(
                "failed to decrypt master_key_wrapped_user_key: {}",
                error.message()
            ))
        })?;
        let user_key = vault_crypto::parse_user_key_material(&plaintext_user_key).map_err(|error| {
            AppError::validation(format!("failed to parse decrypted user_key: {}", error.message()))
        })?;

        runtime.set_vault_user_key_material(unlock_context.account_id.clone(), user_key)?;

        log::info!(
            target: "vanguard::application::vault_unlock",
            "vault unlocked with password in memory account_id={}",
            unlock_context.account_id
        );

        Ok(UnlockVaultResult {
            account_id: unlock_context.account_id,
        })
    }
}

fn resolve_unlock_context(runtime: &dyn VaultRuntimePort) -> AppResult<VaultUnlockContext> {
    if let Some(auth_session) = runtime.auth_session_context()? {
        return Ok(auth_session);
    }

    runtime.persisted_auth_context()?.ok_or_else(|| {
        AppError::validation(
            "no authenticated or persisted account state found, please login first",
        )
    })
}

fn derive_wrapping_key_material(master_key: &[u8]) -> Result<VaultUserKeyMaterial, AppError> {
    if master_key.len() != MASTER_KEY_LEN {
        return Err(AppError::validation(format!(
            "master key length must be {MASTER_KEY_LEN} bytes, got {}",
            master_key.len()
        )));
    }

    Ok(VaultUserKeyMaterial {
        enc_key: hkdf_expand_from_prk(master_key, b"enc", 32),
        mac_key: Some(hkdf_expand_from_prk(master_key, b"mac", 32)),
    })
}

fn hkdf_expand_from_prk(prk: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let mut okm = Vec::with_capacity(len);
    let mut previous = Vec::new();
    let mut counter: u8 = 1;

    while okm.len() < len {
        let mut expand = HmacSha256::new_from_slice(prk).expect("hkdf prk");
        if !previous.is_empty() {
            expand.update(&previous);
        }
        expand.update(info);
        expand.update(&[counter]);
        previous = expand.finalize().into_bytes().to_vec();
        okm.extend_from_slice(&previous);
        counter = counter.saturating_add(1);
        if counter == 0 {
            break;
        }
    }

    okm.truncate(len);
    okm
}

fn derive_master_key(
    email: &str,
    plaintext_password: &str,
    kdf: i32,
    kdf_iterations: i32,
    kdf_memory: Option<i32>,
    kdf_parallelism: Option<i32>,
) -> Result<Vec<u8>, &'static str> {
    let normalized_email = email.trim().to_lowercase();
    let normalized_email_bytes = normalized_email.as_bytes();
    let password_bytes = plaintext_password.as_bytes();

    let master_key = match kdf {
        KDF_PBKDF2 => {
            let iterations = to_u32(kdf_iterations)?;
            pbkdf2_hmac_array::<Sha256, MASTER_KEY_LEN>(
                password_bytes,
                normalized_email_bytes,
                iterations,
            )
        }
        KDF_ARGON2ID => {
            let iterations = to_u32(kdf_iterations)?;
            let memory_mib = kdf_memory.ok_or("missing kdfMemory")?;
            let memory_kib = to_u32(memory_mib)?
                .checked_mul(1024)
                .ok_or("invalid kdfMemory")?;
            let parallelism = to_u32(kdf_parallelism.ok_or("missing kdfParallelism")?)?;

            let params = Params::new(memory_kib, iterations, parallelism, Some(MASTER_KEY_LEN))
                .map_err(|_| "invalid argon2 params")?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let salt_sha = Sha256::digest(normalized_email_bytes);

            let mut key = [0u8; MASTER_KEY_LEN];
            argon2
                .hash_password_into(password_bytes, salt_sha.as_slice(), &mut key)
                .map_err(|_| "argon2 failure")?;
            key
        }
        _ => return Err("unsupported kdf type"),
    };

    Ok(master_key.to_vec())
}

fn to_u32(value: i32) -> Result<u32, &'static str> {
    if value <= 0 {
        return Err("invalid kdf parameter");
    }

    value.try_into().map_err(|_| "invalid kdf parameter")
}
