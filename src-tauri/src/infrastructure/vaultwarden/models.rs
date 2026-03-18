use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloginRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreloginResponse {
    pub kdf: i32,
    pub kdf_iterations: i32,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordLoginRequest {
    pub client_id: String,
    pub username: String,
    pub password: String,
    pub scope: String,
    pub device_identifier: String,
    pub device_name: String,
    pub device_type: String,
    pub two_factor_provider: Option<i32>,
    pub two_factor_token: Option<String>,
    pub two_factor_remember: Option<i32>,
    pub authrequest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub refresh_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scope: Option<String>,
    pub device_identifier: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub two_factor_provider: Option<i32>,
    pub two_factor_token: Option<String>,
    pub two_factor_remember: Option<i32>,
    pub authrequest: Option<String>,
}

impl TokenRequest {
    pub fn from_password(request: PasswordLoginRequest) -> Self {
        Self {
            grant_type: String::from("password"),
            refresh_token: None,
            client_id: Some(request.client_id),
            client_secret: None,
            username: Some(request.username),
            password: Some(request.password),
            scope: Some(request.scope),
            device_identifier: Some(request.device_identifier),
            device_name: Some(request.device_name),
            device_type: Some(request.device_type),
            two_factor_provider: request.two_factor_provider,
            two_factor_token: request.two_factor_token,
            two_factor_remember: request.two_factor_remember,
            authrequest: request.authrequest,
        }
    }

    pub fn from_refresh_token(request: RefreshTokenRequest) -> Self {
        Self {
            grant_type: String::from("refresh_token"),
            refresh_token: Some(request.refresh_token),
            client_id: request.client_id,
            client_secret: None,
            username: None,
            password: None,
            scope: None,
            device_identifier: None,
            device_name: None,
            device_type: None,
            two_factor_provider: None,
            two_factor_token: None,
            two_factor_remember: None,
            authrequest: None,
        }
    }

    pub fn to_form_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        pairs.push((String::from("grant_type"), self.grant_type.clone()));
        push_pair(&mut pairs, "refresh_token", self.refresh_token.clone());
        push_pair(&mut pairs, "client_id", self.client_id.clone());
        push_pair(&mut pairs, "client_secret", self.client_secret.clone());
        push_pair(&mut pairs, "username", self.username.clone());
        push_pair(&mut pairs, "password", self.password.clone());
        push_pair(&mut pairs, "scope", self.scope.clone());
        push_pair(
            &mut pairs,
            "device_identifier",
            self.device_identifier.clone(),
        );
        push_pair(&mut pairs, "device_name", self.device_name.clone());
        push_pair(&mut pairs, "device_type", self.device_type.clone());
        push_pair(
            &mut pairs,
            "two_factor_provider",
            self.two_factor_provider.map(|v| v.to_string()),
        );
        push_pair(
            &mut pairs,
            "two_factor_token",
            self.two_factor_token.clone(),
        );
        push_pair(
            &mut pairs,
            "two_factor_remember",
            self.two_factor_remember.map(|v| v.to_string()),
        );
        push_pair(&mut pairs, "authrequest", self.authrequest.clone());
        pairs
    }
}

fn push_pair(pairs: &mut Vec<(String, String)>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        pairs.push((String::from(key), value));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub scope: Option<String>,
    #[serde(rename = "Key")]
    pub key: Option<String>,
    #[serde(rename = "PrivateKey")]
    pub private_key: Option<String>,
    #[serde(rename = "Kdf")]
    pub kdf: Option<i32>,
    #[serde(rename = "KdfIterations")]
    pub kdf_iterations: Option<i32>,
    #[serde(rename = "KdfMemory")]
    pub kdf_memory: Option<i32>,
    #[serde(rename = "KdfParallelism")]
    pub kdf_parallelism: Option<i32>,
    #[serde(rename = "TwoFactorToken")]
    pub two_factor_token: Option<String>,
    #[serde(rename = "MasterPasswordPolicy")]
    pub master_password_policy: Option<MasterPasswordPolicy>,
    #[serde(rename = "AccountKeys")]
    pub account_keys: Option<AccountKeys>,
    #[serde(rename = "UserDecryptionOptions")]
    pub user_decryption_options: Option<UserDecryptionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenErrorResponse {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "TwoFactorProviders")]
    pub two_factor_providers: Option<Vec<String>>,
    #[serde(rename = "TwoFactorProviders2")]
    pub two_factor_providers2: Option<HashMap<String, Option<TwoFactorProviderHint>>>,
    #[serde(rename = "MasterPasswordPolicy")]
    pub master_password_policy: Option<MasterPasswordPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterPasswordPolicy {
    pub min_complexity: Option<i32>,
    pub min_length: Option<i32>,
    #[serde(default)]
    pub require_lower: bool,
    #[serde(default)]
    pub require_upper: bool,
    #[serde(default)]
    pub require_numbers: bool,
    #[serde(default)]
    pub require_special: bool,
    #[serde(default)]
    pub enforce_on_login: bool,
    #[serde(rename = "Object")]
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountKeys {
    #[serde(rename = "publicKeyEncryptionKeyPair")]
    pub public_key_encryption_key_pair: Option<PublicKeyEncryptionKeyPair>,
    #[serde(rename = "Object")]
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyEncryptionKeyPair {
    #[serde(rename = "wrappedPrivateKey")]
    pub wrapped_private_key: Option<String>,
    #[serde(rename = "publicKey")]
    pub public_key: Option<String>,
    #[serde(rename = "Object")]
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDecryptionOptions {
    #[serde(rename = "HasMasterPassword")]
    pub has_master_password: Option<bool>,
    #[serde(rename = "MasterPasswordUnlock")]
    pub master_password_unlock: Option<TokenMasterPasswordUnlock>,
    #[serde(rename = "Object")]
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMasterPasswordUnlock {
    #[serde(rename = "Kdf")]
    pub kdf: Option<TokenKdfParams>,
    #[serde(rename = "MasterKeyEncryptedUserKey")]
    pub master_key_encrypted_user_key: Option<String>,
    #[serde(rename = "MasterKeyWrappedUserKey")]
    pub master_key_wrapped_user_key: Option<String>,
    #[serde(rename = "Salt")]
    pub salt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenKdfParams {
    #[serde(rename = "KdfType")]
    pub kdf_type: Option<i32>,
    #[serde(rename = "Iterations")]
    pub iterations: Option<i32>,
    #[serde(rename = "Memory")]
    pub memory: Option<i32>,
    #[serde(rename = "Parallelism")]
    pub parallelism: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorProviderHint {
    #[serde(rename = "Host")]
    pub host: Option<String>,
    #[serde(rename = "Signature")]
    pub signature: Option<String>,
    #[serde(rename = "AuthUrl")]
    pub auth_url: Option<String>,
    #[serde(rename = "Nfc")]
    pub nfc: Option<bool>,
    #[serde(rename = "Email")]
    pub email: Option<String>,
    pub challenge: Option<String>,
    pub timeout: Option<i32>,
    #[serde(rename = "rpId")]
    pub rp_id: Option<String>,
    #[serde(default)]
    pub allow_credentials: Vec<WebauthnAllowCredential>,
    pub user_verification: Option<String>,
    pub extensions: Option<WebauthnRequestExtensions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebauthnAllowCredential {
    pub r#type: Option<String>,
    pub id: Option<String>,
    #[serde(default)]
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebauthnRequestExtensions {
    pub appid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailLoginRequest {
    pub device_identifier: String,
    pub email: Option<String>,
    pub master_password_hash: Option<String>,
    pub auth_request_id: Option<String>,
    pub auth_request_access_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailTokenRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub profile: SyncProfile,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub folders: Vec<SyncFolder>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub collections: Vec<SyncCollection>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub policies: Vec<SyncPolicy>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub ciphers: Vec<SyncCipher>,
    pub domains: Option<SyncDomains>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub sends: Vec<SyncSend>,
    pub user_decryption: Option<SyncUserDecryption>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProfile {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFolder {
    pub id: String,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFoldersResponse {
    pub data: Vec<SyncFolder>,
    pub object: String,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCollection {
    pub id: String,
    pub organization_id: Option<String>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPolicy {
    pub id: String,
    pub organization_id: Option<String>,
    pub r#type: Option<i32>,
    pub enabled: Option<bool>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub fields: Vec<SyncCipherField>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub password_history: Vec<SyncCipherPasswordHistory>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub collection_ids: Vec<String>,
    pub data: Option<SyncCipherData>,
    pub login: Option<SyncCipherLogin>,
    pub secure_note: Option<SyncCipherSecureNote>,
    pub card: Option<SyncCipherCard>,
    pub identity: Option<SyncCipherIdentity>,
    pub ssh_key: Option<SyncCipherSshKey>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub attachments: Vec<SyncAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAttachment {
    pub id: String,
    pub key: Option<String>,
    pub file_name: Option<String>,
    pub size: Option<String>,
    pub size_name: Option<String>,
    pub url: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherPermissions {
    pub delete: Option<bool>,
    pub restore: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherField {
    pub name: Option<String>,
    pub value: Option<String>,
    pub r#type: Option<i32>,
    pub linked_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherPasswordHistory {
    pub password: Option<String>,
    pub last_used_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherData {
    pub name: Option<String>,
    pub notes: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub fields: Vec<SyncCipherField>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub password_history: Vec<SyncCipherPasswordHistory>,
    pub uri: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub uris: Vec<SyncCipherLoginUri>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherLogin {
    pub uri: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub uris: Vec<SyncCipherLoginUri>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_revision_date: Option<String>,
    pub totp: Option<String>,
    pub autofill_on_page_load: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub fido2_credentials: Vec<SyncCipherLoginFido2Credential>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherLoginUri {
    pub uri: Option<String>,
    pub r#match: Option<i32>,
    pub uri_checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherSecureNote {
    pub r#type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherCard {
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCipherSshKey {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSend {
    pub id: String,
    pub r#type: Option<i32>,
    pub name: Option<String>,
    pub revision_date: Option<String>,
    pub deletion_date: Option<String>,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDomains {
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub equivalent_domains: Vec<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub global_equivalent_domains: Vec<SyncGlobalEquivalentDomainEntry>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub excluded_global_equivalent_domains: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SyncGlobalEquivalentDomainEntry {
    Legacy(Vec<String>),
    Detailed(SyncGlobalEquivalentDomain),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncGlobalEquivalentDomain {
    pub r#type: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_null_seq_or_map_default")]
    pub domains: Vec<String>,
    pub excluded: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncUserDecryption {
    pub master_password_unlock: Option<SyncMasterPasswordUnlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMasterPasswordUnlock {
    pub kdf: Option<SyncKdfParams>,
    pub master_key_encrypted_user_key: Option<String>,
    pub master_key_wrapped_user_key: Option<String>,
    pub salt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncKdfParams {
    pub kdf_type: Option<i32>,
    pub iterations: Option<i32>,
    pub memory: Option<i32>,
    pub parallelism: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RevisionDateResponse {
    Number(i64),
    Text(String),
    Object(HashMap<String, RevisionDateScalar>),
}

impl RevisionDateResponse {
    pub fn to_revision_ms(&self) -> Option<i64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(value) => value.parse::<i64>().ok(),
            Self::Object(values) => {
                if let Some(value) = values
                    .get("revisionDate")
                    .or_else(|| values.get("revision_date"))
                {
                    return value.to_revision_ms();
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RevisionDateScalar {
    Number(i64),
    Text(String),
}

impl RevisionDateScalar {
    pub fn to_revision_ms(&self) -> Option<i64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(value) => value.parse::<i64>().ok(),
        }
    }
}

fn deserialize_null_seq_or_map_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<SeqOrMap<T>>::deserialize(deserializer)?;
    Ok(match value {
        None => Vec::new(),
        Some(SeqOrMap::Seq(items)) => items,
        Some(SeqOrMap::Map(map)) => map.into_values().collect(),
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SeqOrMap<T> {
    Seq(Vec<T>),
    Map(HashMap<String, T>),
}

#[derive(Debug, Clone)]
pub struct VaultSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CipherResponse {
    pub id: String,
    pub revision_date: String,
}

// Registration models

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub name: Option<String>,
    pub receive_marketing_emails: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterFinishRequest {
    pub email: String,
    pub master_password_hash: String,
    #[serde(serialize_with = "serialize_hint_as_empty_string")]
    pub master_password_hint: Option<String>,
    pub user_symmetric_key: String,
    pub user_asymmetric_keys: RegisterKeys,
    pub kdf: i32,
    pub kdf_iterations: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kdf_memory: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kdf_parallelism: Option<i32>,
    pub email_verification_token: Option<String>,
}

fn serialize_hint_as_empty_string<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value.as_deref().unwrap_or(""))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterKeys {
    pub public_key: String,
    pub encrypted_private_key: String,
}
