use std::fmt::{Display, Formatter};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use pbkdf2::pbkdf2_hmac_array;
use sha2::{Digest, Sha256};

use super::models::PreloginResponse;

const KDF_PBKDF2: i32 = 0;
const KDF_ARGON2ID: i32 = 1;
const MASTER_KEY_LEN: usize = 32;
const SERVER_AUTHORIZATION_ROUNDS: u32 = 1;

#[derive(Debug)]
pub enum PasswordHashError {
    UnsupportedKdf(i32),
    InvalidKdfParameter(&'static str),
    MissingKdfParameter(&'static str),
    Argon2(String),
}

impl Display for PasswordHashError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedKdf(kdf) => write!(f, "unsupported kdf type: {kdf}"),
            Self::InvalidKdfParameter(param) => write!(f, "invalid kdf parameter: {param}"),
            Self::MissingKdfParameter(param) => write!(f, "missing kdf parameter: {param}"),
            Self::Argon2(message) => write!(f, "argon2 failure: {message}"),
        }
    }
}

impl std::error::Error for PasswordHashError {}

pub fn derive_master_password_hash(
    email: &str,
    plaintext_password: &str,
    prelogin: &PreloginResponse,
) -> Result<String, PasswordHashError> {
    let master_key = derive_master_key(
        email,
        plaintext_password,
        prelogin.kdf,
        prelogin.kdf_iterations,
        prelogin.kdf_memory,
        prelogin.kdf_parallelism,
    )?;
    let hash = pbkdf2_hmac_array::<Sha256, MASTER_KEY_LEN>(
        &master_key,
        plaintext_password.as_bytes(),
        SERVER_AUTHORIZATION_ROUNDS,
    );
    Ok(base64::engine::general_purpose::STANDARD.encode(hash))
}

pub fn derive_master_key(
    email: &str,
    plaintext_password: &str,
    kdf: i32,
    kdf_iterations: i32,
    kdf_memory: Option<i32>,
    kdf_parallelism: Option<i32>,
) -> Result<Vec<u8>, PasswordHashError> {
    let normalized_email = email.trim().to_lowercase();
    let normalized_email_bytes = normalized_email.as_bytes();
    let password_bytes = plaintext_password.as_bytes();

    let master_key = match kdf {
        KDF_PBKDF2 => {
            let iterations = to_u32(kdf_iterations, "kdfIterations")?;
            pbkdf2_hmac_array::<Sha256, MASTER_KEY_LEN>(
                password_bytes,
                normalized_email_bytes,
                iterations,
            )
        }
        KDF_ARGON2ID => {
            let iterations = to_u32(kdf_iterations, "kdfIterations")?;
            let memory_mib =
                kdf_memory.ok_or(PasswordHashError::MissingKdfParameter("kdfMemory"))?;
            let memory_kib = to_u32(memory_mib, "kdfMemory")?
                .checked_mul(1024)
                .ok_or(PasswordHashError::InvalidKdfParameter("kdfMemory"))?;
            let parallelism = to_u32(
                kdf_parallelism.ok_or(PasswordHashError::MissingKdfParameter("kdfParallelism"))?,
                "kdfParallelism",
            )?;

            let params = Params::new(memory_kib, iterations, parallelism, Some(MASTER_KEY_LEN))
                .map_err(|error| PasswordHashError::Argon2(error.to_string()))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let salt_sha = Sha256::digest(normalized_email_bytes);

            let mut key = [0u8; MASTER_KEY_LEN];
            argon2
                .hash_password_into(password_bytes, salt_sha.as_slice(), &mut key)
                .map_err(|error| PasswordHashError::Argon2(error.to_string()))?;
            key
        }
        kdf => return Err(PasswordHashError::UnsupportedKdf(kdf)),
    };

    Ok(master_key.to_vec())
}

fn to_u32(value: i32, name: &'static str) -> Result<u32, PasswordHashError> {
    if value <= 0 {
        return Err(PasswordHashError::InvalidKdfParameter(name));
    }

    value
        .try_into()
        .map_err(|_| PasswordHashError::InvalidKdfParameter(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_pbkdf2_master_password_hash() {
        let password = "asdfasdf";
        let prelogin = PreloginResponse {
            kdf: 0,
            kdf_iterations: 100_000,
            kdf_memory: None,
            kdf_parallelism: None,
        };

        let hash = derive_master_password_hash(" test@bitwarden.com", password, &prelogin)
            .expect("pbkdf2 derivation should succeed");

        assert_eq!(hash, "wmyadRMyBZOH7P/a/ucTCbSghKgdzDpPqUnu/DAVtSw=");
    }

    #[test]
    fn derives_argon2_master_password_hash() {
        let password = "asdfasdf";
        let prelogin = PreloginResponse {
            kdf: 1,
            kdf_iterations: 4,
            kdf_memory: Some(32),
            kdf_parallelism: Some(2),
        };

        let hash = derive_master_password_hash("test_salt", password, &prelogin)
            .expect("argon2 derivation should succeed");

        assert_eq!(hash, "PR6UjYmjmppTYcdyTiNbAhPJuQQOmynKbdEl1oyi/iQ=");
    }
}
