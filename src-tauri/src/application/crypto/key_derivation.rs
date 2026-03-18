use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;

use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

type HmacSha256 = Hmac<Sha256>;

const MASTER_KEY_LEN: usize = 32;
const DEFAULT_PBKDF2_ITERATIONS: u32 = 600_000;

/// Derives a 32-byte Master Key from password + email using PBKDF2-SHA256.
pub fn derive_master_key_pbkdf2(
    password: &str,
    email: &str,
    iterations: Option<u32>,
) -> Result<Vec<u8>, AppError> {
    let iterations = iterations.unwrap_or(DEFAULT_PBKDF2_ITERATIONS);
    if iterations == 0 {
        return Err(AppError::CryptoKeyDerivationFailed);
    }
    let salt = email.trim().to_lowercase();
    let key = pbkdf2_hmac_array::<Sha256, MASTER_KEY_LEN>(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
    );
    Ok(key.to_vec())
}

/// Generates the Master Password Hash for server authentication.
/// PBKDF2-SHA256 with master_key as input, password as salt, 1 iteration.
pub fn derive_master_password_hash(master_key: &[u8], password: &str) -> Result<Vec<u8>, AppError> {
    if master_key.len() != MASTER_KEY_LEN {
        return Err(AppError::CryptoKeyDerivationFailed);
    }
    let hash = pbkdf2_hmac_array::<Sha256, MASTER_KEY_LEN>(master_key, password.as_bytes(), 1);
    Ok(hash.to_vec())
}

/// Derives a stretched key (enc_key + mac_key) from master_key using HKDF-Expand.
/// Returns a `VaultUserKeyMaterial` with 32-byte enc_key (info="enc") and 32-byte mac_key (info="mac").
pub fn derive_stretched_master_key(master_key: &[u8]) -> Result<VaultUserKeyMaterial, AppError> {
    if master_key.len() != MASTER_KEY_LEN {
        return Err(AppError::CryptoKeyDerivationFailed);
    }
    Ok(VaultUserKeyMaterial {
        enc_key: hkdf_expand(master_key, b"enc", 32),
        mac_key: Some(hkdf_expand(master_key, b"mac", 32)),
    })
}

/// HKDF-Expand (RFC 5869) using HMAC-SHA256.
/// `prk` is the pseudo-random key, `info` is context, `len` is desired output length.
pub fn hkdf_expand(prk: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let mut okm = Vec::with_capacity(len);
    let mut previous = Vec::new();
    let mut counter: u8 = 1;

    while okm.len() < len {
        let mut hmac = HmacSha256::new_from_slice(prk).expect("valid prk length");
        if !previous.is_empty() {
            hmac.update(&previous);
        }
        hmac.update(info);
        hmac.update(&[counter]);
        previous = hmac.finalize().into_bytes().to_vec();
        okm.extend_from_slice(&previous);
        counter = counter.saturating_add(1);
        if counter == 0 {
            break;
        }
    }

    okm.truncate(len);
    okm
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    // Bitwarden reference vector: email="test@bitwarden.com", password="asdfasdf", 100k iterations
    // Known master_password_hash = "wmyadRMyBZOH7P/a/ucTCbSghKgdzDpPqUnu/DAVtSw="
    const TEST_EMAIL: &str = " test@bitwarden.com";
    const TEST_PASSWORD: &str = "asdfasdf";
    const TEST_ITERATIONS: u32 = 100_000;
    const EXPECTED_HASH_B64: &str = "wmyadRMyBZOH7P/a/ucTCbSghKgdzDpPqUnu/DAVtSw=";

    #[test]
    fn pbkdf2_derives_32_byte_master_key() {
        let key = derive_master_key_pbkdf2(TEST_PASSWORD, TEST_EMAIL, Some(TEST_ITERATIONS))
            .expect("should derive");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn pbkdf2_email_is_lowercased_and_trimmed() {
        let k1 =
            derive_master_key_pbkdf2(TEST_PASSWORD, " Test@Bitwarden.com", Some(TEST_ITERATIONS))
                .unwrap();
        let k2 =
            derive_master_key_pbkdf2(TEST_PASSWORD, "test@bitwarden.com", Some(TEST_ITERATIONS))
                .unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn pbkdf2_zero_iterations_fails() {
        assert!(derive_master_key_pbkdf2(TEST_PASSWORD, TEST_EMAIL, Some(0)).is_err());
    }

    #[test]
    fn pbkdf2_default_iterations_produces_key() {
        let key = derive_master_key_pbkdf2(TEST_PASSWORD, TEST_EMAIL, None).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn master_password_hash_matches_bitwarden_vector() {
        let master_key =
            derive_master_key_pbkdf2(TEST_PASSWORD, TEST_EMAIL, Some(TEST_ITERATIONS)).unwrap();
        let hash = derive_master_password_hash(&master_key, TEST_PASSWORD).unwrap();
        assert_eq!(STANDARD.encode(&hash), EXPECTED_HASH_B64);
    }

    #[test]
    fn master_password_hash_rejects_wrong_key_length() {
        assert!(derive_master_password_hash(&[0u8; 16], TEST_PASSWORD).is_err());
    }

    #[test]
    fn hkdf_expand_produces_correct_length() {
        let prk = [0xABu8; 32];
        assert_eq!(hkdf_expand(&prk, b"enc", 32).len(), 32);
        assert_eq!(hkdf_expand(&prk, b"mac", 32).len(), 32);
        assert_eq!(hkdf_expand(&prk, b"test", 64).len(), 64);
    }

    #[test]
    fn hkdf_expand_different_info_produces_different_keys() {
        let prk = [0xABu8; 32];
        let enc = hkdf_expand(&prk, b"enc", 32);
        let mac = hkdf_expand(&prk, b"mac", 32);
        assert_ne!(enc, mac);
    }

    #[test]
    fn stretched_master_key_has_enc_and_mac() {
        let master_key =
            derive_master_key_pbkdf2(TEST_PASSWORD, TEST_EMAIL, Some(TEST_ITERATIONS)).unwrap();
        let stretched = derive_stretched_master_key(&master_key).unwrap();
        assert_eq!(stretched.enc_key.len(), 32);
        assert_eq!(stretched.mac_key.as_ref().unwrap().len(), 32);
        assert_ne!(stretched.enc_key, *stretched.mac_key.as_ref().unwrap());
    }

    #[test]
    fn stretched_master_key_rejects_wrong_length() {
        assert!(derive_stretched_master_key(&[0u8; 16]).is_err());
    }

    #[test]
    fn hkdf_expand_is_deterministic() {
        let prk = [0x42u8; 32];
        let a = hkdf_expand(&prk, b"enc", 32);
        let b = hkdf_expand(&prk, b"enc", 32);
        assert_eq!(a, b);
    }
}
