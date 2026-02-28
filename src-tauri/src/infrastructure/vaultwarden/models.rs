use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub master_password_policy: Option<Value>,
    #[serde(rename = "AccountKeys")]
    pub account_keys: Option<Value>,
    #[serde(rename = "UserDecryptionOptions")]
    pub user_decryption_options: Option<Value>,
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenErrorResponse {
    pub error: Option<String>,
    pub error_description: Option<String>,
    #[serde(rename = "TwoFactorProviders")]
    pub two_factor_providers: Option<Vec<String>>,
    #[serde(rename = "TwoFactorProviders2")]
    pub two_factor_providers2: Option<Value>,
    #[serde(rename = "MasterPasswordPolicy")]
    pub master_password_policy: Option<Value>,
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
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
pub struct SyncResponse {
    pub profile: Value,
    pub folders: Vec<Value>,
    pub ciphers: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct VaultSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
}
