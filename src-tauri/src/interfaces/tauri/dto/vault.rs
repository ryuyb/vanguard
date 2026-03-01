use serde::{Deserialize, Serialize};
use specta::Type;

use crate::interfaces::tauri::dto::sync::SyncStatusResponseDto;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultViewDataRequestDto {
    pub base_url: String,
    pub access_token: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultUnlockWithUserKeyRequestDto {
    pub base_url: String,
    pub access_token: String,
    pub user_key: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultUnlockWithPasswordRequestDto {
    pub base_url: String,
    pub access_token: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultLockRequestDto {
    pub base_url: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VaultDecryptionStatusDto {
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultFolderItemDto {
    pub id: String,
    pub name: Option<String>,
    pub encrypted_name: Option<String>,
    pub is_name_encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherItemDto {
    pub id: String,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub encrypted_name: Option<String>,
    pub is_name_encrypted: bool,
    pub revision_date: Option<String>,
    pub deleted_date: Option<String>,
    pub attachment_count: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultViewDataResponseDto {
    pub account_id: String,
    pub sync_status: SyncStatusResponseDto,
    pub decryption_status: VaultDecryptionStatusDto,
    pub folders: Vec<VaultFolderItemDto>,
    pub ciphers: Vec<VaultCipherItemDto>,
    pub total_ciphers: u32,
    pub page: u32,
    pub page_size: u32,
}
