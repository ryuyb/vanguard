use crate::domain::sync::{SyncContext, SyncItemCounts, SyncResult, SyncTrigger};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone)]
pub struct SyncVaultCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub exclude_domains: bool,
    pub trigger: SyncTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncVaultPayload {
    pub profile: SyncProfile,
    pub folders: Vec<SyncFolder>,
    pub collections: Vec<SyncCollection>,
    pub policies: Vec<SyncPolicy>,
    pub ciphers: Vec<SyncCipher>,
    pub domains: Option<SyncDomains>,
    pub sends: Vec<SyncSend>,
    pub user_decryption: Option<SyncUserDecryption>,
}

#[derive(Debug, Clone)]
pub struct RevisionDateQuery {
    pub base_url: String,
    pub access_token: String,
}

#[derive(Debug, Clone)]
pub struct SyncOutcome {
    pub context: SyncContext,
    pub result: SyncResult,
}

#[derive(Debug, Clone, Default)]
pub struct SyncMetricsSummary {
    pub window_size: u32,
    pub sample_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub failure_rate: f64,
    pub last_duration_ms: Option<i64>,
    pub average_duration_ms: Option<i64>,
    pub last_item_counts: Option<SyncItemCounts>,
    pub average_item_counts: Option<SyncItemCounts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncProfile {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncFolder {
    pub id: String,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCollection {
    pub id: String,
    pub organization_id: Option<String>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncPolicy {
    pub id: String,
    pub organization_id: Option<String>,
    pub r#type: Option<i32>,
    pub enabled: Option<bool>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipher {
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
    pub permissions: Option<SyncCipherPermissions>,
    pub object: Option<String>,
    pub fields: Vec<SyncCipherField>,
    pub password_history: Vec<SyncCipherPasswordHistory>,
    pub collection_ids: Vec<String>,
    pub data: Option<SyncCipherData>,
    pub login: Option<SyncCipherLogin>,
    pub secure_note: Option<SyncCipherSecureNote>,
    pub card: Option<SyncCipherCard>,
    pub identity: Option<SyncCipherIdentity>,
    pub ssh_key: Option<SyncCipherSshKey>,
    pub attachments: Vec<SyncAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncAttachment {
    pub id: String,
    pub key: Option<String>,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
    pub url: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherPermissions {
    pub delete: Option<bool>,
    pub restore: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherField {
    pub name: Option<String>,
    pub value: Option<String>,
    pub r#type: Option<i32>,
    pub linked_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherPasswordHistory {
    pub password: Option<String>,
    pub last_used_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherData {
    pub name: Option<String>,
    pub notes: Option<String>,
    pub fields: Vec<SyncCipherField>,
    pub password_history: Vec<SyncCipherPasswordHistory>,
    pub uri: Option<String>,
    pub uris: Vec<SyncCipherLoginUri>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<SyncCipherLoginFido2Credential>,
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
pub struct SyncCipherLogin {
    pub uri: Option<String>,
    pub uris: Vec<SyncCipherLoginUri>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    pub fido2_credentials: Vec<SyncCipherLoginFido2Credential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherLoginUri {
    pub uri: Option<String>,
    pub r#match: Option<i32>,
    pub uri_checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherLoginFido2Credential {
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
pub struct SyncCipherSecureNote {
    pub r#type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherCard {
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncCipherIdentity {
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
pub struct SyncCipherSshKey {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncSendText {
    pub text: Option<String>,
    pub hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncSendFile {
    pub id: Option<String>,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncSend {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub deletion_date: Option<String>,
    pub object: Option<String>,
    pub access_id: Option<String>,
    pub notes: Option<String>,
    pub key: Option<String>,
    pub password: Option<String>,
    pub text: Option<SyncSendText>,
    pub file: Option<SyncSendFile>,
    pub max_access_count: Option<i32>,
    pub access_count: Option<i32>,
    pub disabled: Option<bool>,
    pub hide_email: Option<bool>,
    pub expiration_date: Option<String>,
    pub emails: Option<String>,
    pub auth_type: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CreateSendCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub send: SyncSend,
}

#[derive(Debug, Clone)]
pub struct UpdateSendCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub send_id: String,
    pub send: SyncSend,
}

#[derive(Debug, Clone)]
pub struct DeleteSendCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub send_id: String,
}

#[derive(Debug, Clone)]
pub struct RemoveSendPasswordCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub send_id: String,
}

#[derive(Debug, Clone)]
pub struct SendMutationResult {
    pub send_id: String,
    pub revision_date: String,
}

#[derive(Debug, Clone)]
pub struct CreateFileSendResult {
    pub send_id: String,
    pub file_id: String,
    pub url: String,
    pub revision_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncDomains {
    pub equivalent_domains: Vec<Vec<String>>,
    pub global_equivalent_domains: Vec<Vec<String>>,
    pub excluded_global_equivalent_domains: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncUserDecryption {
    pub master_password_unlock: Option<SyncMasterPasswordUnlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncMasterPasswordUnlock {
    pub kdf: Option<SyncKdfParams>,
    pub master_key_encrypted_user_key: Option<String>,
    pub master_key_wrapped_user_key: Option<String>,
    pub salt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncKdfParams {
    pub kdf_type: Option<i32>,
    pub iterations: Option<i32>,
    pub memory: Option<i32>,
    pub parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CreateCipherCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub cipher: SyncCipher,
}

#[derive(Debug, Clone)]
pub struct UpdateCipherCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub cipher_id: String,
    pub cipher: SyncCipher,
}

#[derive(Debug, Clone)]
pub struct DeleteCipherCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub cipher_id: String,
}

#[derive(Debug, Clone)]
pub struct SoftDeleteCipherCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub cipher_id: String,
}

#[derive(Debug, Clone)]
pub struct RestoreCipherCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub cipher_id: String,
}

#[derive(Debug, Clone)]
pub struct CipherMutationResult {
    pub cipher_id: String,
    pub revision_date: String,
}
