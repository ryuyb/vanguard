use rand::RngExt;

use crate::application::crypto::cipher_string;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

/// Generates a random 64-byte symmetric key (32 enc + 32 mac) and encrypts it
/// with the stretched master key, returning the CipherString.
pub fn generate_encrypted_symmetric_key(
    stretched_master_key: &VaultUserKeyMaterial,
) -> Result<(VaultUserKeyMaterial, String), AppError> {
    let mut raw = [0u8; 64];
    rand::rng().fill(&mut raw);

    let user_key = VaultUserKeyMaterial {
        enc_key: raw[..32].to_vec(),
        mac_key: Some(raw[32..].to_vec()),
        refresh_token: None,
    };

    let encrypted = cipher_string::encrypt(&raw, stretched_master_key)?;

    Ok((user_key, encrypted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::crypto::key_derivation;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    fn test_stretched_key() -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
            refresh_token: None,
        }
    }

    #[test]
    fn generates_64_byte_symmetric_key() {
        let stretched = test_stretched_key();
        let (user_key, _encrypted) = generate_encrypted_symmetric_key(&stretched).unwrap();
        assert_eq!(user_key.enc_key.len(), 32);
        assert_eq!(user_key.mac_key.as_ref().unwrap().len(), 32);
    }

    #[test]
    fn encrypted_symmetric_key_is_decryptable() {
        let stretched = test_stretched_key();
        let (user_key, encrypted) = generate_encrypted_symmetric_key(&stretched).unwrap();

        let decrypted = cipher_string::decrypt(&encrypted, &stretched).unwrap();
        assert_eq!(decrypted.len(), 64);
        assert_eq!(&decrypted[..32], &user_key.enc_key);
        assert_eq!(
            &decrypted[32..],
            user_key.mac_key.as_ref().unwrap().as_slice()
        );
    }

    #[test]
    fn encrypted_symmetric_key_is_type_2_cipher_string() {
        let stretched = test_stretched_key();
        let (_user_key, encrypted) = generate_encrypted_symmetric_key(&stretched).unwrap();
        assert!(encrypted.starts_with("2."));
    }

    /// Verifies the full Bitwarden-compatible key derivation chain:
    /// password + email → master_key → password_hash + stretched_key → symmetric_key + RSA
    #[test]
    fn bitwarden_compatible_full_chain() {
        let email = "test@bitwarden.com";
        let password = "asdfasdf";
        let iterations = 100_000u32;

        // Step 1: Derive master key
        let master_key =
            key_derivation::derive_master_key_pbkdf2(password, email, Some(iterations)).unwrap();
        assert_eq!(master_key.len(), 32);

        // Step 2: Derive password hash (must match known Bitwarden vector)
        let hash = key_derivation::derive_master_password_hash(&master_key, password).unwrap();
        assert_eq!(
            STANDARD.encode(&hash),
            "wmyadRMyBZOH7P/a/ucTCbSghKgdzDpPqUnu/DAVtSw="
        );

        // Step 3: Derive stretched key
        let stretched = key_derivation::derive_stretched_master_key(&master_key).unwrap();
        assert_eq!(stretched.enc_key.len(), 32);
        assert!(stretched.mac_key.is_some());

        // Step 4: Generate and encrypt symmetric key
        let (user_key, encrypted_key) = generate_encrypted_symmetric_key(&stretched).unwrap();
        assert!(encrypted_key.starts_with("2."));

        // Step 5: Verify decryption roundtrip
        let decrypted = cipher_string::decrypt(&encrypted_key, &stretched).unwrap();
        assert_eq!(&decrypted[..32], &user_key.enc_key);
        assert_eq!(
            &decrypted[32..],
            user_key.mac_key.as_ref().unwrap().as_slice()
        );
    }
}
