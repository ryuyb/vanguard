use aes::Aes256;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use rand::RngExt;
use sha2::Sha256;

use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;
type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;
type HmacSha256 = Hmac<Sha256>;

/// Result of AES-256-CBC + HMAC-SHA256 encryption: (iv, ciphertext, mac).
pub type EncryptedParts = (Vec<u8>, Vec<u8>, Vec<u8>);

/// Encrypts plaintext bytes using AES-256-CBC + HMAC-SHA256 (type 2).
/// Returns `(iv, ciphertext, mac)`.
pub fn encrypt_aes256_hmac(
    plaintext: &[u8],
    key: &VaultUserKeyMaterial,
) -> Result<EncryptedParts, AppError> {
    let mac_key = key.mac_key.as_deref().ok_or(AppError::CryptoInvalidKey)?;

    let mut iv = [0u8; 16];
    rand::rng().fill(&mut iv);

    let ciphertext = encrypt_aes_cbc_inner(&iv, plaintext, &key.enc_key)?;
    let mac = compute_hmac(&iv, &ciphertext, mac_key)?;

    Ok((iv.to_vec(), ciphertext, mac))
}

/// Decrypts ciphertext using AES-256-CBC + HMAC-SHA256 verification.
pub fn decrypt_aes256_hmac(
    iv: &[u8],
    ciphertext: &[u8],
    mac: &[u8],
    key: &VaultUserKeyMaterial,
) -> Result<Vec<u8>, AppError> {
    let mac_key = key.mac_key.as_deref().ok_or(AppError::CryptoInvalidKey)?;
    verify_hmac(iv, ciphertext, mac, mac_key)?;
    decrypt_aes_cbc_inner(iv, ciphertext, &key.enc_key)
}

/// Encrypts plaintext with AES-256-CBC only (no HMAC). Returns `(iv, ciphertext)`.
pub fn encrypt_aes_cbc_raw(
    plaintext: &[u8],
    enc_key: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let mut iv = [0u8; 16];
    rand::rng().fill(&mut iv);
    let ciphertext = encrypt_aes_cbc_inner(&iv, plaintext, enc_key)?;
    Ok((iv.to_vec(), ciphertext))
}

/// Decrypts ciphertext with AES-256-CBC only (no HMAC verification).
pub fn decrypt_aes_cbc_raw(
    iv: &[u8],
    ciphertext: &[u8],
    enc_key: &[u8],
) -> Result<Vec<u8>, AppError> {
    decrypt_aes_cbc_inner(iv, ciphertext, enc_key)
}

/// Computes HMAC-SHA256 over IV + ciphertext.
pub fn compute_hmac(iv: &[u8], ciphertext: &[u8], mac_key: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut signer = HmacSha256::new_from_slice(mac_key).map_err(|_| AppError::CryptoInvalidKey)?;
    signer.update(iv);
    signer.update(ciphertext);
    Ok(signer.finalize().into_bytes().to_vec())
}

/// Verifies HMAC-SHA256 over IV + ciphertext.
pub fn verify_hmac(
    iv: &[u8],
    ciphertext: &[u8],
    mac: &[u8],
    mac_key: &[u8],
) -> Result<(), AppError> {
    let mut signer = HmacSha256::new_from_slice(mac_key).map_err(|_| AppError::CryptoInvalidKey)?;
    signer.update(iv);
    signer.update(ciphertext);
    signer
        .verify_slice(mac)
        .map_err(|_| AppError::CryptoDecryptionFailed)
}

fn encrypt_aes_cbc_inner(iv: &[u8], plaintext: &[u8], enc_key: &[u8]) -> Result<Vec<u8>, AppError> {
    let encryptor =
        Aes256CbcEncryptor::new_from_slices(enc_key, iv).map_err(|_| AppError::CryptoInvalidKey)?;

    let block_size = 16;
    let padding_len = block_size - (plaintext.len() % block_size);
    let mut buffer = vec![0u8; plaintext.len() + padding_len];
    buffer[..plaintext.len()].copy_from_slice(plaintext);

    let ciphertext = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.len())
        .map_err(|_| AppError::CryptoEncryptionFailed)?;

    Ok(ciphertext.to_vec())
}

fn decrypt_aes_cbc_inner(
    iv: &[u8],
    ciphertext: &[u8],
    enc_key: &[u8],
) -> Result<Vec<u8>, AppError> {
    let decryptor =
        Aes256CbcDecryptor::new_from_slices(enc_key, iv).map_err(|_| AppError::CryptoInvalidKey)?;

    let mut buffer = ciphertext.to_vec();
    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| AppError::CryptoDecryptionFailed)?;

    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> VaultUserKeyMaterial {
        VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: Some(vec![2u8; 32]),
        }
    }

    #[test]
    fn aes256_hmac_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello bitwarden";
        let (iv, ct, mac) = encrypt_aes256_hmac(plaintext, &key).unwrap();
        let decrypted = decrypt_aes256_hmac(&iv, &ct, &mac, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes256_hmac_tampered_mac_fails() {
        let key = test_key();
        let (iv, ct, mut mac) = encrypt_aes256_hmac(b"secret", &key).unwrap();
        mac[0] ^= 0xFF;
        assert!(decrypt_aes256_hmac(&iv, &ct, &mac, &key).is_err());
    }

    #[test]
    fn aes256_hmac_tampered_ciphertext_fails() {
        let key = test_key();
        let (iv, mut ct, mac) = encrypt_aes256_hmac(b"secret", &key).unwrap();
        ct[0] ^= 0xFF;
        assert!(decrypt_aes256_hmac(&iv, &ct, &mac, &key).is_err());
    }

    #[test]
    fn aes256_hmac_requires_mac_key() {
        let key = VaultUserKeyMaterial {
            enc_key: vec![1u8; 32],
            mac_key: None,
        };
        assert!(encrypt_aes256_hmac(b"test", &key).is_err());
    }

    #[test]
    fn aes_cbc_raw_roundtrip() {
        let enc_key = [3u8; 32];
        let plaintext = b"raw aes test data";
        let (iv, ct) = encrypt_aes_cbc_raw(plaintext, &enc_key).unwrap();
        let decrypted = decrypt_aes_cbc_raw(&iv, &ct, &enc_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_cbc_raw_wrong_key_fails() {
        let enc_key = [3u8; 32];
        let (iv, ct) = encrypt_aes_cbc_raw(b"test", &enc_key).unwrap();
        let wrong_key = [4u8; 32];
        assert!(decrypt_aes_cbc_raw(&iv, &ct, &wrong_key).is_err());
    }

    #[test]
    fn hmac_compute_and_verify() {
        let mac_key = [5u8; 32];
        let iv = [0u8; 16];
        let ct = b"some ciphertext";
        let mac = compute_hmac(&iv, ct, &mac_key).unwrap();
        assert!(verify_hmac(&iv, ct, &mac, &mac_key).is_ok());
    }

    #[test]
    fn hmac_verify_rejects_wrong_mac() {
        let mac_key = [5u8; 32];
        let iv = [0u8; 16];
        let ct = b"some ciphertext";
        let mut mac = compute_hmac(&iv, ct, &mac_key).unwrap();
        mac[0] ^= 0xFF;
        assert!(verify_hmac(&iv, ct, &mac, &mac_key).is_err());
    }

    #[test]
    fn encrypt_produces_unique_iv_each_time() {
        let key = test_key();
        let (iv1, _, _) = encrypt_aes256_hmac(b"same", &key).unwrap();
        let (iv2, _, _) = encrypt_aes256_hmac(b"same", &key).unwrap();
        assert_ne!(iv1, iv2);
    }

    #[test]
    fn aes_cbc_handles_empty_plaintext() {
        let key = test_key();
        let (iv, ct, mac) = encrypt_aes256_hmac(b"", &key).unwrap();
        let decrypted = decrypt_aes256_hmac(&iv, &ct, &mac, &key).unwrap();
        assert!(decrypted.is_empty());
    }
}
