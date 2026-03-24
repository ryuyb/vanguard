use serde::{Deserialize, Serialize};
use specta::Type;

use crate::application::dto::sync::SyncCipher;
use crate::domain::cipher::{Cipher, Decrypted};
use crate::interfaces::tauri::dto::sync::SyncStatusResponseDto;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherTotpCodeRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VaultCopyFieldDto {
    Username,
    Password,
    Totp,
    Notes,
    #[serde(rename_all = "camelCase")]
    CustomField {
        index: usize,
    },
    #[serde(rename_all = "camelCase")]
    Uri {
        index: usize,
    },
    CardNumber,
    CardCode,
    Email,
    Phone,
    SshPrivateKey,
    SshPublicKey,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCopyCipherFieldRequestDto {
    pub cipher_id: String,
    pub field: VaultCopyFieldDto,
    pub clear_after_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCopyCipherFieldResponseDto {
    pub copied: bool,
    pub clear_after_ms: Option<u64>,
    pub autofill_performed: bool,
    /// The copied value - will be set if autofill is enabled so frontend can trigger autofill after hiding spotlight
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultExecuteAutofillRequestDto {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultExecuteAutofillResponseDto {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultFolderItemDto {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherItemDto {
    pub id: String,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub uris: Vec<String>,
    pub favorite: Option<bool>,
    pub creation_date: Option<String>,
    pub revision_date: Option<String>,
    pub deleted_date: Option<String>,
    pub attachment_count: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultViewDataResponseDto {
    pub account_id: String,
    pub sync_status: SyncStatusResponseDto,
    pub folders: Vec<VaultFolderItemDto>,
    pub ciphers: Vec<VaultCipherItemDto>,
    pub total_ciphers: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailResponseDto {
    pub account_id: String,
    pub cipher: VaultCipherDetailDto,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherTotpCodeResponseDto {
    pub code: String,
    pub period_seconds: u64,
    pub remaining_seconds: u64,
    pub expires_at_ms: i64,
}

/// Vault cipher detail DTO using domain type with flattened fields
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailDto {
    /// Flatten all Cipher<Decrypted> fields to the top level
    #[serde(flatten)]
    pub inner: Cipher<Decrypted>,

    /// Computed field: whether the cipher has TOTP configured
    pub has_totp: bool,
}

impl From<Cipher<Decrypted>> for VaultCipherDetailDto {
    fn from(cipher: Cipher<Decrypted>) -> Self {
        Self {
            has_totp: cipher.has_totp(),
            inner: cipher,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateCipherRequestDto {
    pub cipher: SyncCipher,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCipherRequestDto {
    pub cipher_id: String,
    pub cipher: SyncCipher,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCipherRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SoftDeleteCipherRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCipherRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CipherMutationResponseDto {
    pub cipher_id: String,
    pub revision_date: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dto_from_conversion() {
        // Test that From<Cipher<Decrypted>> is implemented
        // This is a compile-time check
        fn assert_from_trait<T: Into<VaultCipherDetailDto>>() {}
        assert_from_trait::<crate::domain::cipher::Cipher<crate::domain::cipher::Decrypted>>();
    }
}
