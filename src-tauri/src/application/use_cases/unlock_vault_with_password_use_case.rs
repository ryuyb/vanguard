use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac_array;
use sha2::{Digest, Sha256};

use crate::application::dto::sync::SyncUserDecryption;
use crate::application::dto::vault::{
    UnlockVaultWithPasswordCommand, UnlockVaultWithPasswordResult, VaultUnlockContext,
    VaultUserKeyMaterial,
};
use crate::application::ports::vault_runtime_port::VaultRuntimePort;
use crate::application::services::sync_service::SyncService;
use crate::application::vault_crypto;
use crate::support::error::AppError;
use crate::support::result::AppResult;

type HmacSha256 = Hmac<Sha256>;

const BITWARDEN_HKDF_SALT: &[u8] = b"bitwarden";
const KDF_PBKDF2: i32 = 0;
const KDF_ARGON2ID: i32 = 1;
const MASTER_KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct UnlockVaultWithPasswordUseCase {
    sync_service: Arc<SyncService>,
}

#[derive(Debug, Clone)]
struct UnlockKdfParams {
    kdf_type: i32,
    iterations: i32,
    memory: Option<i32>,
    parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
struct UnlockMaterial {
    encrypted_user_keys: Vec<String>,
    kdf: Option<UnlockKdfParams>,
    salt: Option<String>,
}

impl UnlockVaultWithPasswordUseCase {
    pub fn new(sync_service: Arc<SyncService>) -> Self {
        Self { sync_service }
    }

    pub async fn execute(
        &self,
        runtime: &dyn VaultRuntimePort,
        command: UnlockVaultWithPasswordCommand,
    ) -> AppResult<UnlockVaultWithPasswordResult> {
        let master_password = command.master_password.trim().to_string();
        if master_password.is_empty() {
            return Err(AppError::validation("master_password cannot be empty"));
        }

        let unlock_context = resolve_unlock_context(runtime)?;
        let unlock_material = self
            .sync_service
            .load_live_user_decryption(unlock_context.account_id.clone())
            .await
            .and_then(extract_unlock_material)?;
        let master_keys = self
            .derive_master_key_candidates_for_unlock(
                &unlock_context,
                &master_password,
                &unlock_material,
            )
            ?;
        let user_key =
            decrypt_user_key_with_master_keys(&unlock_material.encrypted_user_keys, &master_keys)?;
        runtime.set_vault_user_key_material(unlock_context.account_id.clone(), user_key)?;

        log::info!(
            target: "vanguard::application::vault_unlock",
            "vault unlocked with password in memory account_id={}",
            unlock_context.account_id
        );

        Ok(UnlockVaultWithPasswordResult {
            account_id: unlock_context.account_id,
        })
    }

    fn derive_master_key_candidates_for_unlock(
        &self,
        unlock_context: &VaultUnlockContext,
        master_password: &str,
        unlock_material: &UnlockMaterial,
    ) -> AppResult<Vec<Vec<u8>>> {
        let mut candidates = Vec::new();

        if let Some(kdf) = &unlock_material.kdf {
            let salt = unlock_material
                .salt
                .clone()
                .unwrap_or_else(|| unlock_context.email.clone());
            maybe_push_master_key(
                &mut candidates,
                &salt,
                master_password,
                kdf.kdf_type,
                kdf.iterations,
                kdf.memory,
                kdf.parallelism,
            )?;
        }

        if let (Some(kdf), Some(iterations)) = (unlock_context.kdf, unlock_context.kdf_iterations) {
            maybe_push_master_key(
                &mut candidates,
                &unlock_context.email,
                master_password,
                kdf,
                iterations,
                unlock_context.kdf_memory,
                unlock_context.kdf_parallelism,
            )?;
        }

        if candidates.is_empty() {
            return Err(AppError::validation(
                "unable to derive local master key candidates for unlock; login and sync once to refresh local unlock metadata",
            ));
        }

        Ok(candidates)
    }
}

pub fn has_master_password_unlock_material(value: Option<SyncUserDecryption>) -> AppResult<bool> {
    match extract_unlock_material(value) {
        Ok(_) => Ok(true),
        Err(error) if is_unlock_material_missing(&error) => Ok(false),
        Err(error) => Err(error),
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

fn extract_unlock_material(value: Option<SyncUserDecryption>) -> Result<UnlockMaterial, AppError> {
    let value = value.ok_or_else(|| {
        AppError::validation("missing local user_decryption data; run vault sync first")
    })?;
    let unlock = value.master_password_unlock.ok_or_else(|| {
        AppError::validation("missing master_password_unlock in local vault metadata")
    })?;

    let mut encrypted_user_keys = Vec::new();
    if let Some(value) = unlock.master_key_wrapped_user_key {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            encrypted_user_keys.push(String::from(trimmed));
        }
    }
    if let Some(value) = unlock.master_key_encrypted_user_key {
        let trimmed = value.trim();
        if !trimmed.is_empty() && encrypted_user_keys.iter().all(|item| item != trimmed) {
            encrypted_user_keys.push(String::from(trimmed));
        }
    }
    if encrypted_user_keys.is_empty() {
        return Err(AppError::validation(
            "encrypted user key is missing in local vault metadata",
        ));
    }

    let kdf = unlock.kdf.and_then(|value| {
        Some(UnlockKdfParams {
            kdf_type: value.kdf_type?,
            iterations: value.iterations?,
            memory: value.memory,
            parallelism: value.parallelism,
        })
    });

    let salt = unlock
        .salt
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(UnlockMaterial {
        encrypted_user_keys,
        kdf,
        salt,
    })
}

fn is_unlock_material_missing(error: &AppError) -> bool {
    let AppError::Validation(message) = error else {
        return false;
    };

    message.contains("missing local user_decryption data")
        || message.contains("missing master_password_unlock")
        || message.contains("encrypted user key is missing")
}

fn maybe_push_master_key(
    candidates: &mut Vec<Vec<u8>>,
    email_or_salt: &str,
    master_password: &str,
    kdf: i32,
    kdf_iterations: i32,
    kdf_memory: Option<i32>,
    kdf_parallelism: Option<i32>,
) -> Result<(), AppError> {
    let key = derive_master_key(
        email_or_salt,
        master_password,
        kdf,
        kdf_iterations,
        kdf_memory,
        kdf_parallelism,
    )
    .map_err(|error| {
        AppError::validation(format!(
            "failed to derive master key with provided kdf params: {error}"
        ))
    })?;
    if candidates.iter().all(|existing| existing != &key) {
        candidates.push(key);
    }
    Ok(())
}

fn decrypt_user_key_with_master_keys(
    encrypted_user_keys: &[String],
    master_keys: &[Vec<u8>],
) -> Result<VaultUserKeyMaterial, AppError> {
    if encrypted_user_keys.is_empty() {
        return Err(AppError::validation(
            "encrypted_user_key list cannot be empty",
        ));
    }

    for encrypted_user_key in encrypted_user_keys {
        let trimmed = encrypted_user_key.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !vault_crypto::looks_like_cipher_string(trimmed) {
            if let Ok(parsed) = vault_crypto::parse_user_key(trimmed) {
                return Ok(parsed);
            }
            continue;
        }

        for master_key in master_keys {
            for candidate in candidate_keys_from_master_key(master_key) {
                if let Ok(plaintext_user_key) =
                    vault_crypto::decrypt_cipher_bytes(trimmed, &candidate)
                {
                    if let Ok(user_key) = vault_crypto::parse_user_key_material(&plaintext_user_key)
                    {
                        return Ok(user_key);
                    }
                }
            }
        }
    }

    let enc_types: Vec<String> = encrypted_user_keys
        .iter()
        .filter_map(|value| value.trim().split_once('.').map(|(enc_type, _)| enc_type))
        .map(String::from)
        .collect();

    log::warn!(
        target: "vanguard::application::vault_unlock",
        "failed to decrypt encrypted_user_key candidates_count={} master_key_candidates={} enc_types={:?}",
        encrypted_user_keys.len(),
        master_keys.len(),
        enc_types
    );

    Err(AppError::validation(
        "unable to unlock encrypted_user_key with provided password",
    ))
}

fn candidate_keys_from_master_key(master_key: &[u8]) -> Vec<VaultUserKeyMaterial> {
    let mut candidates = Vec::new();

    if master_key.len() == 32 {
        candidates.push(VaultUserKeyMaterial {
            enc_key: hkdf_expand_from_prk(master_key, b"enc", 32),
            mac_key: Some(hkdf_expand_from_prk(master_key, b"mac", 32)),
        });
        candidates.push(VaultUserKeyMaterial {
            enc_key: master_key.to_vec(),
            mac_key: None,
        });
        candidates.push(VaultUserKeyMaterial {
            enc_key: master_key.to_vec(),
            mac_key: Some(hmac_derive(master_key, b"mac")),
        });
        candidates.push(VaultUserKeyMaterial {
            enc_key: hkdf_expand_with_salt(master_key, &[0u8; 32], b"enc", 32),
            mac_key: Some(hkdf_expand_with_salt(master_key, &[0u8; 32], b"mac", 32)),
        });
        candidates.push(VaultUserKeyMaterial {
            enc_key: hkdf_expand_with_salt(master_key, BITWARDEN_HKDF_SALT, b"enc", 32),
            mac_key: Some(hkdf_expand_with_salt(
                master_key,
                BITWARDEN_HKDF_SALT,
                b"mac",
                32,
            )),
        });
        let hashed = Sha256::digest(master_key).to_vec();
        candidates.push(VaultUserKeyMaterial {
            enc_key: hashed.clone(),
            mac_key: Some(hmac_derive(&hashed, b"mac")),
        });
    } else if master_key.len() == 64 {
        candidates.push(VaultUserKeyMaterial {
            enc_key: master_key[..32].to_vec(),
            mac_key: Some(master_key[32..].to_vec()),
        });
    }

    candidates
}

fn hmac_derive(key: &[u8], label: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key must be valid");
    mac.update(label);
    mac.finalize().into_bytes().to_vec()
}

fn hkdf_expand_with_salt(ikm: &[u8], salt: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let mut extract = HmacSha256::new_from_slice(salt).expect("hkdf salt");
    extract.update(ikm);
    let prk = extract.finalize().into_bytes();

    let mut okm = Vec::with_capacity(len);
    let mut previous = Vec::new();
    let mut counter: u8 = 1;

    while okm.len() < len {
        let mut expand = HmacSha256::new_from_slice(&prk).expect("hkdf prk");
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

#[cfg(test)]
mod tests {
    use super::*;
    use aes::Aes256;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};

    type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;

    #[test]
    fn decrypt_user_key_with_master_keys_supports_hmac_derived_mac_key() {
        let master_key = [7u8; 32];
        let enc_key = master_key;
        let mac_key_vec = hmac_derive(&master_key, b"mac");
        let mac_key: [u8; 32] = mac_key_vec
            .try_into()
            .expect("hmac output should be 32 bytes");
        let plain_user_key = STANDARD.encode([3u8; 64]);
        let encrypted_user_key = encrypt_type2(&plain_user_key, &enc_key, &mac_key);

        let parsed =
            decrypt_user_key_with_master_keys(&[encrypted_user_key], &[master_key.to_vec()])
                .expect("unlock with password candidate key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn decrypt_user_key_with_master_keys_supports_raw_64_byte_material() {
        let master_key = [7u8; 32];
        let enc_key = hkdf_expand_from_prk(&master_key, b"enc", 32);
        let mac_key_vec = hkdf_expand_from_prk(&master_key, b"mac", 32);
        let mac_key: [u8; 32] = mac_key_vec
            .try_into()
            .expect("hkdf output should be 32 bytes");
        let plain_user_key = vec![3u8; 64];
        let encrypted_user_key = encrypt_type2_bytes(&plain_user_key, &enc_key, &mac_key);

        let parsed =
            decrypt_user_key_with_master_keys(&[encrypted_user_key], &[master_key.to_vec()])
                .expect("unlock with password candidate key");
        assert_eq!(parsed.enc_key.len(), 32);
        assert_eq!(parsed.mac_key.as_ref().map(Vec::len), Some(32));
    }

    #[test]
    fn extract_unlock_material_prefers_wrapped_variant() {
        let extracted =
            extract_unlock_material(Some(crate::application::dto::sync::SyncUserDecryption {
                master_password_unlock: Some(
                    crate::application::dto::sync::SyncMasterPasswordUnlock {
                        kdf: None,
                        master_key_encrypted_user_key: Some(String::from(
                            "2.encrypted|payload|mac",
                        )),
                        master_key_wrapped_user_key: Some(String::from("2.wrapped|payload|mac")),
                        salt: None,
                    },
                ),
            }))
            .expect("extract wrapped");

        assert_eq!(
            extracted.encrypted_user_keys,
            vec![
                String::from("2.wrapped|payload|mac"),
                String::from("2.encrypted|payload|mac")
            ]
        );
    }

    #[test]
    fn extract_unlock_material_fallbacks_to_encrypted_variant() {
        let extracted =
            extract_unlock_material(Some(crate::application::dto::sync::SyncUserDecryption {
                master_password_unlock: Some(
                    crate::application::dto::sync::SyncMasterPasswordUnlock {
                        kdf: None,
                        master_key_encrypted_user_key: Some(String::from(
                            "2.encrypted|payload|mac",
                        )),
                        master_key_wrapped_user_key: None,
                        salt: None,
                    },
                ),
            }))
            .expect("extract encrypted");

        assert_eq!(
            extracted.encrypted_user_keys,
            vec![String::from("2.encrypted|payload|mac")]
        );
    }

    fn encrypt_type2(plaintext: &str, enc_key: &[u8; 32], mac_key: &[u8; 32]) -> String {
        encrypt_type2_bytes(plaintext.as_bytes(), enc_key, mac_key)
    }

    fn encrypt_type2_bytes(plaintext: &[u8], enc_key: &[u8], mac_key: &[u8]) -> String {
        let iv = [9u8; 16];
        let mut buffer = plaintext.to_vec();
        let message_len = buffer.len();
        buffer.resize(message_len + 16, 0);

        let ciphertext = Aes256CbcEncryptor::new_from_slices(enc_key, &iv)
            .expect("build encryptor")
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .expect("encrypt")
            .to_vec();

        let mut mac = HmacSha256::new_from_slice(mac_key).expect("build hmac");
        mac.update(&iv);
        mac.update(&ciphertext);
        let mac = mac.finalize().into_bytes();

        format!(
            "2.{}|{}|{}",
            STANDARD.encode(iv),
            STANDARD.encode(ciphertext),
            STANDARD.encode(mac)
        )
    }
}
