use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PasswordLoginRequestDto {
    pub base_url: String,
    pub email: String,
    pub master_password: String,
    pub two_factor_provider: Option<i32>,
    pub two_factor_token: Option<String>,
    pub two_factor_remember: Option<bool>,
    pub authrequest: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailLoginRequestDto {
    pub base_url: String,
    pub email: Option<String>,
    pub master_password: Option<String>,
    pub auth_request_id: Option<String>,
    pub auth_request_access_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailTokenRequestDto {
    pub base_url: String,
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponseDto {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i32,
    pub token_type: String,
    pub scope: Option<String>,
    pub key: Option<String>,
    pub private_key: Option<String>,
    pub kdf: Option<i32>,
    pub kdf_iterations: Option<i32>,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
    pub two_factor_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorChallengeDto {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub providers: Vec<String>,
    pub providers2: Option<HashMap<String, Option<TwoFactorProviderHintDto>>>,
    pub master_password_policy: Option<MasterPasswordPolicyDto>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum PasswordLoginResponseDto {
    Authenticated(SessionResponseDto),
    TwoFactorRequired(TwoFactorChallengeDto),
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MasterPasswordPolicyDto {
    pub min_complexity: Option<i32>,
    pub min_length: Option<i32>,
    pub require_lower: bool,
    pub require_upper: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub enforce_on_login: bool,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorProviderHintDto {
    pub host: Option<String>,
    pub signature: Option<String>,
    pub auth_url: Option<String>,
    pub nfc: Option<bool>,
    pub email: Option<String>,
    pub challenge: Option<String>,
    pub timeout: Option<i32>,
    pub rp_id: Option<String>,
    pub allow_credentials: Vec<WebauthnAllowCredentialDto>,
    pub user_verification: Option<String>,
    pub extensions: Option<WebauthnRequestExtensionsDto>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebauthnAllowCredentialDto {
    pub r#type: Option<String>,
    pub id: Option<String>,
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WebauthnRequestExtensionsDto {
    pub appid: Option<String>,
}
