use crate::application::dto::sync::SyncMetricsSummary;
use crate::application::dto::unlock::VaultUserKeyMaterial;
use crate::domain::sync::SyncContext;

#[derive(Debug, Clone)]
pub struct GetCipherDetailQuery {
    pub account_id: String,
    pub cipher_id: String,
    pub user_key: VaultUserKeyMaterial,
}

#[derive(Debug, Clone, Copy)]
pub enum VaultCopyField {
    Username,
    Password,
    Totp,
    Notes,
    CustomField { index: usize },
    Uri { index: usize },
    CardNumber,
    CardCode,
    Email,
    Phone,
    SshPrivateKey,
    SshPublicKey,
}

#[derive(Debug, Clone)]
pub struct CopyCipherFieldCommand {
    pub cipher_id: String,
    pub field: VaultCopyField,
    pub clear_after_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CopyCipherFieldResult {
    pub copied: bool,
    pub clear_after_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GetCipherTotpCodeCommand {
    pub cipher_id: String,
}

#[derive(Debug, Clone)]
pub struct GetCipherTotpCodeResult {
    pub code: String,
    pub period_seconds: u64,
    pub remaining_seconds: u64,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct VaultFolderItem {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VaultCipherItem {
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

#[derive(Debug, Clone)]
pub struct GetVaultViewDataResult {
    pub account_id: String,
    pub sync_context: SyncContext,
    pub sync_metrics: SyncMetricsSummary,
    pub folders: Vec<VaultFolderItem>,
    pub ciphers: Vec<VaultCipherItem>,
    pub total_ciphers: u32,
}
