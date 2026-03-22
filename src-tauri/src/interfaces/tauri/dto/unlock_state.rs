use serde::{Deserialize, Serialize};
use specta::Type;

/// Current unlock status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UnlockStatusDto {
    /// Vault is locked, no session or keys available
    Locked,
    /// Vault is unlocked but API session has expired
    VaultUnlockedSessionExpired,
    /// Both vault and API session are valid
    FullyUnlocked,
    /// Unlock operation is in progress
    Unlocking,
}

/// Account context information (non-sensitive)
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AccountContextDto {
    pub account_id: String,
    pub email: String,
    pub base_url: String,
}

/// Session context information
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextDto {
    /// Whether the session is valid (not expired)
    pub is_valid: bool,
    /// Whether the session is expiring soon (within grace period)
    pub is_expiring_soon: bool,
}

/// Complete unlock state response
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UnlockStateResponseDto {
    pub status: UnlockStatusDto,
    pub account: Option<AccountContextDto>,
    pub session: Option<SessionContextDto>,
    /// Whether vault keys are available in memory
    pub has_key_material: bool,
    /// The method used to unlock (if currently unlocked)
    pub unlock_method: Option<UnlockMethodDto>,
}

/// Unlock method used
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UnlockMethodDto {
    MasterPassword,
    Pin,
    Biometric,
}

/// Request to refresh session
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RefreshSessionRequestDto {}

/// Response from refresh session
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RefreshSessionResponseDto {
    pub success: bool,
    pub is_session_valid: bool,
}

/// Request to get unlock state
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetUnlockStateRequestDto {}

/// Request to subscribe to unlock state changes
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeUnlockStateRequestDto {}
