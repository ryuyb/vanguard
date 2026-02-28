#[derive(Debug, Clone)]
pub struct PreloginQuery {
    pub base_url: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct PreloginInfo {
    pub kdf: i32,
    pub kdf_iterations: i32,
    pub kdf_memory: Option<i32>,
    pub kdf_parallelism: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct PasswordLoginCommand {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub two_factor_provider: Option<i32>,
    pub two_factor_token: Option<String>,
    pub two_factor_remember: Option<bool>,
    pub authrequest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RefreshTokenCommand {
    pub base_url: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct SendEmailLoginCommand {
    pub base_url: String,
    pub email: Option<String>,
    pub plaintext_password: Option<String>,
    pub auth_request_id: Option<String>,
    pub auth_request_access_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerifyEmailTokenCommand {
    pub base_url: String,
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
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

#[derive(Debug, Clone)]
pub struct TwoFactorChallenge {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub providers: Vec<String>,
    pub providers2: Option<serde_json::Value>,
    pub master_password_policy: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum PasswordLoginOutcome {
    Authenticated(SessionInfo),
    TwoFactorRequired(TwoFactorChallenge),
}
