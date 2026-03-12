/// WebSocket push notification types from Bitwarden server
/// Reference: https://github.com/bitwarden/server/blob/main/src/Core/Platform/Push/PushType.cs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PushType {
    SyncCipherUpdate = 0,
    SyncCipherCreate = 1,
    SyncLoginDelete = 2,
    SyncFolderDelete = 3,
    SyncCiphers = 4,
    SyncVault = 5,
    SyncOrgKeys = 6,
    SyncFolderCreate = 7,
    SyncFolderUpdate = 8,
    SyncCipherDelete = 9,
    SyncSettings = 10,
    LogOut = 11,
    SyncSendCreate = 12,
    SyncSendUpdate = 13,
    SyncSendDelete = 14,
    AuthRequest = 15,
    AuthRequestResponse = 16,
    SyncOrganizations = 17,
    SyncOrganizationStatusChanged = 18,
    SyncOrganizationCollectionSettingChanged = 19,
    Notification = 20,
    NotificationStatus = 21,
    RefreshSecurityTasks = 22,
    OrganizationBankAccountVerified = 23,
    ProviderBankAccountVerified = 24,
    PolicyChanged = 25,
    AutoConfirm = 26,
}

impl PushType {
    /// Convert from i32 event type received from WebSocket
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::SyncCipherUpdate),
            1 => Some(Self::SyncCipherCreate),
            2 => Some(Self::SyncLoginDelete),
            3 => Some(Self::SyncFolderDelete),
            4 => Some(Self::SyncCiphers),
            5 => Some(Self::SyncVault),
            6 => Some(Self::SyncOrgKeys),
            7 => Some(Self::SyncFolderCreate),
            8 => Some(Self::SyncFolderUpdate),
            9 => Some(Self::SyncCipherDelete),
            10 => Some(Self::SyncSettings),
            11 => Some(Self::LogOut),
            12 => Some(Self::SyncSendCreate),
            13 => Some(Self::SyncSendUpdate),
            14 => Some(Self::SyncSendDelete),
            15 => Some(Self::AuthRequest),
            16 => Some(Self::AuthRequestResponse),
            17 => Some(Self::SyncOrganizations),
            18 => Some(Self::SyncOrganizationStatusChanged),
            19 => Some(Self::SyncOrganizationCollectionSettingChanged),
            20 => Some(Self::Notification),
            21 => Some(Self::NotificationStatus),
            22 => Some(Self::RefreshSecurityTasks),
            23 => Some(Self::OrganizationBankAccountVerified),
            24 => Some(Self::ProviderBankAccountVerified),
            25 => Some(Self::PolicyChanged),
            26 => Some(Self::AutoConfirm),
            _ => None,
        }
    }

    /// Check if this push type requires a full vault sync
    pub fn is_sync_event(self) -> bool {
        matches!(
            self,
            Self::SyncCipherUpdate
                | Self::SyncCipherCreate
                | Self::SyncLoginDelete
                | Self::SyncFolderDelete
                | Self::SyncCiphers
                | Self::SyncVault
                | Self::SyncOrgKeys
                | Self::SyncFolderCreate
                | Self::SyncFolderUpdate
                | Self::SyncSettings
                | Self::SyncSendCreate
                | Self::SyncSendUpdate
                | Self::SyncSendDelete
        )
    }

    /// Check if this push type supports incremental cipher sync
    pub fn is_incremental_cipher_event(self) -> bool {
        matches!(
            self,
            Self::SyncCipherUpdate | Self::SyncCipherCreate | Self::SyncLoginDelete
        )
    }

    /// Check if this push type supports incremental folder sync
    pub fn is_incremental_folder_event(self) -> bool {
        matches!(
            self,
            Self::SyncFolderDelete | Self::SyncFolderCreate | Self::SyncFolderUpdate
        )
    }

    /// Check if this push type supports incremental send sync
    pub fn is_incremental_send_event(self) -> bool {
        matches!(
            self,
            Self::SyncSendCreate | Self::SyncSendUpdate | Self::SyncSendDelete
        )
    }

    /// Check if this is a logout event
    pub fn is_logout(self) -> bool {
        matches!(self, Self::LogOut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_i32_converts_valid_types() {
        assert_eq!(PushType::from_i32(0), Some(PushType::SyncCipherUpdate));
        assert_eq!(PushType::from_i32(11), Some(PushType::LogOut));
        assert_eq!(PushType::from_i32(26), Some(PushType::AutoConfirm));
    }

    #[test]
    fn from_i32_returns_none_for_invalid() {
        assert_eq!(PushType::from_i32(-1), None);
        assert_eq!(PushType::from_i32(27), None);
        assert_eq!(PushType::from_i32(999), None);
    }

    #[test]
    fn is_sync_event_identifies_sync_types() {
        assert!(PushType::SyncCipherUpdate.is_sync_event());
        assert!(PushType::SyncVault.is_sync_event());
        assert!(PushType::SyncFolderCreate.is_sync_event());
        assert!(!PushType::LogOut.is_sync_event());
        assert!(!PushType::AuthRequest.is_sync_event());
    }

    #[test]
    fn is_incremental_cipher_event_identifies_cipher_types() {
        assert!(PushType::SyncCipherUpdate.is_incremental_cipher_event());
        assert!(PushType::SyncCipherCreate.is_incremental_cipher_event());
        assert!(PushType::SyncLoginDelete.is_incremental_cipher_event());
        assert!(!PushType::SyncFolderCreate.is_incremental_cipher_event());
    }

    #[test]
    fn is_incremental_folder_event_identifies_folder_types() {
        assert!(PushType::SyncFolderDelete.is_incremental_folder_event());
        assert!(PushType::SyncFolderCreate.is_incremental_folder_event());
        assert!(PushType::SyncFolderUpdate.is_incremental_folder_event());
        assert!(!PushType::SyncCipherUpdate.is_incremental_folder_event());
    }

    #[test]
    fn is_incremental_send_event_identifies_send_types() {
        assert!(PushType::SyncSendCreate.is_incremental_send_event());
        assert!(PushType::SyncSendUpdate.is_incremental_send_event());
        assert!(PushType::SyncSendDelete.is_incremental_send_event());
        assert!(!PushType::SyncCipherUpdate.is_incremental_send_event());
    }

    #[test]
    fn is_logout_identifies_logout_event() {
        assert!(PushType::LogOut.is_logout());
        assert!(!PushType::SyncVault.is_logout());
    }
}
