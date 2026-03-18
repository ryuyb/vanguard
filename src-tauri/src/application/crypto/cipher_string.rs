use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::application::crypto::encryption;
use crate::application::dto::vault::VaultUserKeyMaterial;
use crate::support::error::AppError;

/// Encryption type 2: AES-256-CBC + HMAC-SHA256 + Base64
const ENC_TYPE_AESCBC256_HMACSHA256_B64: u8 = 2;

/// Parsed CipherString components.
pub struct CipherStringParts {
    pub enc_type: u8,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub mac: Option<Vec<u8>>,
}

/// Parses a CipherString in format "type.iv|ciphertext|mac".
pub fn parse(value: &str) -> Result<CipherStringParts, AppError> {
    let (enc_type_str, payload) = value
        .trim()
        .split_once('.')
        .ok_or(AppError::CryptoDecryptionFailed)?;

    let enc_type: u8 = enc_type_str
        .parse()
        .map_err(|_| AppError::CryptoDecryptionFailed)?;

    let parts: Vec<&str> = payload.split('|').collect();

    match (enc_type, parts.len()) {
        (0, 2) => Ok(CipherStringParts {
            enc_type,
            iv: decode_b64(parts[0])?,
            ciphertext: decode_b64(parts[1])?,
            mac: None,
        }),
        (2, 3) => Ok(CipherStringParts {
            enc_type,
            iv: decode_b64(parts[0])?,
            ciphertext: decode_b64(parts[1])?,
            mac: Some(decode_b64(parts[2])?),
        }),
        _ => Err(AppError::CryptoDecryptionFailed),
    }
}

/// Encrypts plaintext bytes and returns a CipherString "2.iv|ciphertext|mac".
pub fn encrypt(plaintext: &[u8], key: &VaultUserKeyMaterial) -> Result<String, AppError> {
    let (iv, ciphertext, mac) = encryption::encrypt_aes256_hmac(plaintext, key)?;
    Ok(format!(
        "{}.{}|{}|{}",
        ENC_TYPE_AESCBC256_HMACSHA256_B64,
        STANDARD.encode(&iv),
        STANDARD.encode(&ciphertext),
        STANDARD.encode(&mac),
    ))
}

/// Decrypts a CipherString and returns plaintext bytes.
pub fn decrypt(value: &str, key: &VaultUserKeyMaterial) -> Result<Vec<u8>, AppError> {
    let parts = parse(value)?;
    match parts.enc_type {
        2 => {
            let mac = parts.mac.ok_or(AppError::CryptoDecryptionFailed)?;
            encryption::decrypt_aes256_hmac(&parts.iv, &parts.ciphertext, &mac, key)
        }
        _ => Err(AppError::CryptoDecryptionFailed),
    }
}

fn decode_b64(value: &str) -> Result<Vec<u8>, AppError> {
    STANDARD
        .decode(value)
        .map_err(|_| AppError::CryptoDecryptionFailed)
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
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello cipher string";
        let cs = encrypt(plaintext, &key).unwrap();
        assert!(cs.starts_with("2."));
        let decrypted = decrypt(&cs, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_type_2_format() {
        let key = test_key();
        let cs = encrypt(b"test", &key).unwrap();
        assert!(cs.starts_with("2."));
        let parts: Vec<&str> = cs[2..].split('|').collect();
        assert_eq!(parts.len(), 3); // iv|ct|mac
    }

    #[test]
    fn parse_type_2_cipher_string() {
        let key = test_key();
        let cs = encrypt(b"parse test", &key).unwrap();
        let parts = parse(&cs).unwrap();
        assert_eq!(parts.enc_type, 2);
        assert_eq!(parts.iv.len(), 16);
        assert!(parts.mac.is_some());
    }

    #[test]
    fn parse_invalid_format_fails() {
        assert!(parse("invalid").is_err());
        assert!(parse("").is_err());
        assert!(parse("2.").is_err());
        assert!(parse("x.abc|def|ghi").is_err());
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = test_key();
        let cs = encrypt(b"secret", &key).unwrap();
        let wrong_key = VaultUserKeyMaterial {
            enc_key: vec![9u8; 32],
            mac_key: Some(vec![8u8; 32]),
        };
        assert!(decrypt(&cs, &wrong_key).is_err());
    }

    #[test]
    fn decrypt_unsupported_type_fails() {
        // Construct a fake type 3 cipher string
        assert!(decrypt("3.AAAA|BBBB|CCCC", &test_key()).is_err());
    }

    #[test]
    fn encrypt_empty_plaintext() {
        let key = test_key();
        let cs = encrypt(b"", &key).unwrap();
        let decrypted = decrypt(&cs, &key).unwrap();
        assert!(decrypted.is_empty());
    }
}
