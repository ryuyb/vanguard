use crate::domain::sync::{SyncContext, SyncResult, SyncTrigger};

#[derive(Debug, Clone)]
pub struct SyncVaultCommand {
    pub account_id: String,
    pub base_url: String,
    pub access_token: String,
    pub exclude_domains: bool,
    pub trigger: SyncTrigger,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct SyncProfile {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncFolder {
    pub id: String,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncCollection {
    pub id: String,
    pub organization_id: Option<String>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub id: String,
    pub organization_id: Option<String>,
    pub r#type: Option<i32>,
    pub enabled: Option<bool>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncCipher {
    pub id: String,
    pub organization_id: Option<String>,
    pub folder_id: Option<String>,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub deleted_date: Option<String>,
    pub object: Option<String>,
    pub attachments: Vec<SyncAttachment>,
}

#[derive(Debug, Clone)]
pub struct SyncAttachment {
    pub id: String,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub url: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncSend {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub deletion_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncDomains {
    pub equivalent_domains: Vec<Vec<String>>,
    pub global_equivalent_domains: Vec<Vec<String>>,
    pub excluded_global_equivalent_domains: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct SyncUserDecryption {
    pub master_password_unlock: Option<SyncMasterPasswordUnlock>,
}

#[derive(Debug, Clone)]
pub struct SyncMasterPasswordUnlock {
    pub kdf: Option<SyncKdfParams>,
    pub master_key_encrypted_user_key: Option<String>,
    pub master_key_wrapped_user_key: Option<String>,
    pub salt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncKdfParams {
    pub kdf_type: Option<i32>,
    pub iterations: Option<i32>,
    pub memory: Option<i32>,
    pub parallelism: Option<i32>,
}
