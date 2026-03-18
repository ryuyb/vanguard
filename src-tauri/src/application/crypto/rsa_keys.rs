use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};
use rsa::rand_core::OsRng;
use rsa::RsaPrivateKey;

use crate::application::crypto::cipher_string;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

const RSA_KEY_SIZE: usize = 2048;

/// Generated RSA key pair with encrypted private key.
pub struct RsaKeyPair {
    /// Public key in SPKI DER format, Base64-encoded.
    pub public_key_b64: String,
    /// Private key encrypted as CipherString "2.iv|ct|mac".
    pub encrypted_private_key: String,
}

/// Generates an RSA-2048 key pair.
/// The private key is encrypted with the provided symmetric key.
/// The public key is returned as Base64-encoded SPKI DER.
pub fn generate_rsa_key_pair(symmetric_key: &VaultUserKeyMaterial) -> Result<RsaKeyPair, AppError> {
    let private_key = RsaPrivateKey::new(&mut OsRng, RSA_KEY_SIZE)
        .map_err(|_| AppError::CryptoEncryptionFailed)?;

    // Export private key as PKCS#1 DER
    let private_key_der = private_key
        .to_pkcs1_der()
        .map_err(|_| AppError::CryptoEncryptionFailed)?;

    // Export public key as PKCS#1 DER (Bitwarden uses PKCS#1 for public key)
    let public_key_der = private_key
        .to_public_key()
        .to_pkcs1_der()
        .map_err(|_| AppError::CryptoEncryptionFailed)?;

    // Encrypt private key with symmetric key
    let encrypted_private_key = cipher_string::encrypt(private_key_der.as_bytes(), symmetric_key)?;

    Ok(RsaKeyPair {
        public_key_b64: STANDARD.encode(public_key_der.as_bytes()),
        encrypted_private_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::DecodeRsaPublicKey;
    use rsa::traits::PublicKeyParts;
    use rsa::RsaPublicKey;

    fn test_key() -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        }
    }

    #[test]
    fn generates_valid_rsa_key_pair() {
        let key = test_key();
        let pair = generate_rsa_key_pair(&key).unwrap();

        // Public key should be valid base64 and decodable as PKCS#1 DER
        let pub_der = STANDARD.decode(&pair.public_key_b64).unwrap();
        let pub_key = RsaPublicKey::from_pkcs1_der(&pub_der);
        assert!(pub_key.is_ok());
        assert_eq!(pub_key.unwrap().n().bits(), 2048);
    }

    #[test]
    fn encrypted_private_key_is_type_2_cipher_string() {
        let key = test_key();
        let pair = generate_rsa_key_pair(&key).unwrap();
        assert!(pair.encrypted_private_key.starts_with("2."));
        let parts: Vec<&str> = pair.encrypted_private_key[2..].split('|').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn encrypted_private_key_can_be_decrypted() {
        let key = test_key();
        let pair = generate_rsa_key_pair(&key).unwrap();
        let decrypted = cipher_string::decrypt(&pair.encrypted_private_key, &key).unwrap();
        // PKCS#1 DER private key should be non-empty and start with ASN.1 SEQUENCE tag
        assert!(!decrypted.is_empty());
        assert_eq!(decrypted[0], 0x30); // ASN.1 SEQUENCE
    }
}
