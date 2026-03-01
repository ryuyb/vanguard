use serde::{Deserialize, Serialize};
use specta::Type;

use crate::interfaces::tauri::dto::sync::SyncStatusResponseDto;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultViewDataRequestDto {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailRequestDto {
    pub cipher_id: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultUnlockWithPasswordRequestDto {
    pub master_password: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultLockRequestDto {}

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
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailResponseDto {
    pub account_id: String,
    pub cipher: VaultCipherDetailDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetailDto {
    pub id: String,
    pub organization_id: Option<String>,
    pub folder_id: Option<String>,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub key: Option<String>,
    pub favorite: Option<bool>,
    pub edit: Option<bool>,
    pub view_password: Option<bool>,
    pub organization_use_totp: Option<bool>,
    pub creation_date: Option<String>,
    pub revision_date: Option<String>,
    pub deleted_date: Option<String>,
    pub archived_date: Option<String>,
    pub reprompt: Option<i32>,
    pub permissions: Option<VaultCipherPermissionsDetailDto>,
    pub object: Option<String>,
    pub fields: Vec<VaultCipherFieldDetailDto>,
    pub password_history: Vec<VaultCipherPasswordHistoryDetailDto>,
    pub collection_ids: Vec<String>,
    pub data: Option<VaultCipherDataDetailDto>,
    pub login: Option<VaultCipherLoginDetailDto>,
    pub secure_note: Option<VaultCipherSecureNoteDetailDto>,
    pub card: Option<VaultCipherCardDetailDto>,
    pub identity: Option<VaultCipherIdentityDetailDto>,
    pub ssh_key: Option<VaultCipherSshKeyDetailDto>,
    pub attachments: Vec<VaultAttachmentDetailDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultAttachmentDetailDto {
    pub id: String,
    pub key: Option<String>,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
    pub url: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherPermissionsDetailDto {
    pub delete: Option<bool>,
    pub restore: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherFieldDetailDto {
    pub name: Option<String>,
    pub value: Option<String>,
    pub r#type: Option<i32>,
    pub linked_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherPasswordHistoryDetailDto {
    pub password: Option<String>,
    pub last_used_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDataDetailDto {
    pub name: Option<String>,
    pub notes: Option<String>,
    pub fields: Vec<VaultCipherFieldDetailDto>,
    pub password_history: Vec<VaultCipherPasswordHistoryDetailDto>,
    pub uri: Option<String>,
    pub uris: Vec<VaultCipherLoginUriDetailDto>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<VaultCipherLoginFido2CredentialDetailDto>,
    pub r#type: Option<i32>,
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub address3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ssn: Option<String>,
    pub passport_number: Option<String>,
    pub license_number: Option<String>,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginDetailDto {
    pub uri: Option<String>,
    pub uris: Vec<VaultCipherLoginUriDetailDto>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<VaultCipherLoginFido2CredentialDetailDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginUriDetailDto {
    pub uri: Option<String>,
    pub r#match: Option<i32>,
    pub uri_checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginFido2CredentialDetailDto {
    pub credential_id: Option<String>,
    pub key_type: Option<String>,
    pub key_algorithm: Option<String>,
    pub key_curve: Option<String>,
    pub key_value: Option<String>,
    pub rp_id: Option<String>,
    pub rp_name: Option<String>,
    pub counter: Option<String>,
    pub user_handle: Option<String>,
    pub user_name: Option<String>,
    pub user_display_name: Option<String>,
    pub discoverable: Option<String>,
    pub creation_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherSecureNoteDetailDto {
    pub r#type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherCardDetailDto {
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherIdentityDetailDto {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub address3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ssn: Option<String>,
    pub username: Option<String>,
    pub passport_number: Option<String>,
    pub license_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherSshKeyDetailDto {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub key_fingerprint: Option<String>,
}
