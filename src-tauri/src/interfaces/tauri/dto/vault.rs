use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultUnlockRequestDto {
    pub method: VaultUnlockMethodDto,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VaultUnlockMethodDto {
    MasterPassword { password: String },
    Pin { pin: String },
    Biometric,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultEnableBiometricUnlockRequestDto {}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultDisableBiometricUnlockRequestDto {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VaultPinLockTypeDto {
    Disabled,
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultEnablePinUnlockRequestDto {
    pub pin: String,
    pub lock_type: VaultPinLockTypeDto,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultDisablePinUnlockRequestDto {}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultBiometricStatusResponseDto {
    pub supported: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultPinStatusResponseDto {
    pub supported: bool,
    pub enabled: bool,
    pub lock_type: VaultPinLockTypeDto,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultLockRequestDto {}
