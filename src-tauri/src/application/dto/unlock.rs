use crate::domain::unlock::{PinLockType, UnlockMethod};

#[derive(Debug, Clone)]
pub struct UnlockVaultCommand {
    pub method: UnlockMethod,
}

#[derive(Debug, Clone)]
pub struct UnlockVaultResult {
    pub account_id: String,
}

#[derive(Debug, Clone)]
pub struct EnablePinUnlockCommand {
    pub pin: String,
    pub lock_type: PinLockType,
}

#[derive(Debug, Clone)]
pub struct VaultPinStatus {
    pub supported: bool,
    pub enabled: bool,
    pub lock_type: PinLockType,
}

#[derive(Debug, Clone)]
pub struct VaultUnlockContext {
    pub account_id: String,
    pub base_url: String,
    pub email: String,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct VaultBiometricStatus {
    pub supported: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct VaultBiometricBundle {
    pub account_id: String,
    pub enc_key_b64: String,
    pub mac_key_b64: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VaultUserKeyMaterial {
    pub enc_key: Vec<u8>,
    pub mac_key: Option<Vec<u8>>,
}
