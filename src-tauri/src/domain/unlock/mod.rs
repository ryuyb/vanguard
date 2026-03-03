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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinProtectedUserKeyEnvelope {
    pub algorithm: String,
    pub kdf: String,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}
