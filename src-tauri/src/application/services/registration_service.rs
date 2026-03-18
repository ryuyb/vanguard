use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::application::crypto::{key_derivation, rsa_keys, symmetric_key};
use crate::application::dto::auth::RegisterFinishCommand;
use crate::support::result::AppResult;

/// All cryptographic material needed for the register/finish API call.
pub struct RegistrationKeys {
    pub master_password_hash: String,
    pub encrypted_symmetric_key: String,
    pub public_key_b64: String,
    pub encrypted_private_key: String,
}

/// Derives all keys required for Bitwarden-compatible account registration.
///
/// Flow:
/// 1. PBKDF2: email + password → master_key
/// 2. PBKDF2: master_key + password → master_password_hash
/// 3. HKDF: master_key → stretched_key (enc + mac)
/// 4. Random 64-byte symmetric key, encrypted with stretched_key
/// 5. RSA-2048 key pair, private key encrypted with symmetric key
pub fn derive_registration_keys(command: &RegisterFinishCommand) -> AppResult<RegistrationKeys> {
    let master_key = key_derivation::derive_master_key_pbkdf2(
        &command.master_password,
        &command.email,
        Some(command.kdf_iterations as u32),
    )?;

    let hash_bytes =
        key_derivation::derive_master_password_hash(&master_key, &command.master_password)?;
    let master_password_hash = STANDARD.encode(&hash_bytes);

    let stretched = key_derivation::derive_stretched_master_key(&master_key)?;

    let (user_key, encrypted_symmetric_key) =
        symmetric_key::generate_encrypted_symmetric_key(&stretched)?;

    let rsa_pair = rsa_keys::generate_rsa_key_pair(&user_key)?;

    Ok(RegistrationKeys {
        master_password_hash,
        encrypted_symmetric_key,
        public_key_b64: rsa_pair.public_key_b64,
        encrypted_private_key: rsa_pair.encrypted_private_key,
    })
}
