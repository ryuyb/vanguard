use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockMethod {
    MasterPassword { password: String },
    Pin { pin: String },
    Biometric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinLockType {
    Disabled,
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasterPasswordUnlockKdf {
    pub kdf_type: i32,
    pub iterations: i32,
    pub memory: Option<i32>,
    pub parallelism: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasterPasswordUnlockData {
    pub kdf: MasterPasswordUnlockKdf,
    pub salt: String,
    pub master_key_wrapped_user_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinProtectedUserKeyEnvelope {
    pub algorithm: String,
    pub kdf: String,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    /// Optional refresh token encrypted alongside user keys (for session restoration)
    pub refresh_token: Option<String>,
}

/// Bundle for biometric unlock containing user keys and optional refresh token.
/// The refresh_token allows automatic session restoration after biometric authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiometricUnlockBundle {
    pub account_id: String,
    pub enc_key: Vec<u8>,
    pub mac_key: Option<Vec<u8>>,
    /// Optional refresh token for automatic session restoration
    pub refresh_token: Option<String>,
}
