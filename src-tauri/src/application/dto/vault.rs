use serde::Serialize;

#[derive(Debug, Clone)]
pub struct GetCipherDetailQuery {
    pub account_id: String,
    pub cipher_id: String,
    pub user_key: VaultUserKeyMaterial,
}

#[derive(Debug, Clone)]
pub struct VaultUserKeyMaterial {
    pub enc_key: Vec<u8>,
    pub mac_key: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDetail {
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
    pub permissions: Option<VaultCipherPermissionsDetail>,
    pub object: Option<String>,
    pub fields: Vec<VaultCipherFieldDetail>,
    pub password_history: Vec<VaultCipherPasswordHistoryDetail>,
    pub collection_ids: Vec<String>,
    pub data: Option<VaultCipherDataDetail>,
    pub login: Option<VaultCipherLoginDetail>,
    pub secure_note: Option<VaultCipherSecureNoteDetail>,
    pub card: Option<VaultCipherCardDetail>,
    pub identity: Option<VaultCipherIdentityDetail>,
    pub ssh_key: Option<VaultCipherSshKeyDetail>,
    pub attachments: Vec<VaultAttachmentDetail>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultAttachmentDetail {
    pub id: String,
    pub key: Option<String>,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
    pub url: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherPermissionsDetail {
    pub delete: Option<bool>,
    pub restore: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherFieldDetail {
    pub name: Option<String>,
    pub value: Option<String>,
    pub r#type: Option<i32>,
    pub linked_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherPasswordHistoryDetail {
    pub password: Option<String>,
    pub last_used_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherDataDetail {
    pub name: Option<String>,
    pub notes: Option<String>,
    pub fields: Vec<VaultCipherFieldDetail>,
    pub password_history: Vec<VaultCipherPasswordHistoryDetail>,
    pub uri: Option<String>,
    pub uris: Vec<VaultCipherLoginUriDetail>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<VaultCipherLoginFido2CredentialDetail>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginDetail {
    pub uri: Option<String>,
    pub uris: Vec<VaultCipherLoginUriDetail>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<VaultCipherLoginFido2CredentialDetail>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginUriDetail {
    pub uri: Option<String>,
    pub r#match: Option<i32>,
    pub uri_checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherLoginFido2CredentialDetail {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherSecureNoteDetail {
    pub r#type: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherCardDetail {
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherIdentityDetail {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCipherSshKeyDetail {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub key_fingerprint: Option<String>,
}
