use super::config::VaultwardenConfig;
use super::endpoints::VaultwardenEndpoints;
use super::error::{VaultwardenError, VaultwardenResult};
use super::models::{
    PasswordLoginRequest, PreloginRequest, PreloginResponse, RefreshTokenRequest,
    SendEmailLoginRequest, SyncResponse, TokenErrorResponse, TokenRequest, TokenResponse,
    VerifyEmailTokenRequest,
};

#[derive(Debug, Clone)]
pub struct VaultwardenClient {
    config: VaultwardenConfig,
    http_client: reqwest::Client,
}

impl VaultwardenClient {
    pub fn new(config: VaultwardenConfig) -> VaultwardenResult<Self> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.allow_invalid_certs)
            .build()
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    pub fn client_id(&self) -> &str {
        &self.config.client_id
    }

    pub fn scope(&self) -> &str {
        &self.config.scope
    }

    pub fn device_identifier(&self) -> &str {
        &self.config.device_identifier
    }

    pub fn device_name(&self) -> &str {
        &self.config.device_name
    }

    pub fn device_type(&self) -> &str {
        &self.config.device_type
    }

    pub fn allow_invalid_certs(&self) -> bool {
        self.config.allow_invalid_certs
    }

    pub fn prelogin_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::prelogin(base_url))
    }

    pub fn token_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::token(base_url))
    }

    pub fn sync_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::sync(base_url))
    }

    pub fn send_email_login_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::send_email_login(base_url))
    }

    pub fn verify_email_token_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::verify_email_token(base_url))
    }

    pub async fn prelogin(
        &self,
        base_url: &str,
        request: PreloginRequest,
    ) -> VaultwardenResult<PreloginResponse> {
        let endpoint = self.prelogin_endpoint(base_url)?;

        let response = self
            .http_client
            .post(endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        if !(200..300).contains(&status) {
            return Err(Self::api_error(status, body));
        }

        serde_json::from_str::<PreloginResponse>(&body).map_err(|error| {
            VaultwardenError::Decode(format!("invalid prelogin response: {error}"))
        })
    }

    pub async fn login_with_password(
        &self,
        base_url: &str,
        request: PasswordLoginRequest,
    ) -> VaultwardenResult<TokenResponse> {
        self.token(base_url, TokenRequest::from_password(request))
            .await
    }

    pub async fn refresh_token(
        &self,
        base_url: &str,
        request: RefreshTokenRequest,
    ) -> VaultwardenResult<TokenResponse> {
        self.token(base_url, TokenRequest::from_refresh_token(request))
            .await
    }

    pub async fn token(
        &self,
        base_url: &str,
        request: TokenRequest,
    ) -> VaultwardenResult<TokenResponse> {
        let endpoint = self.token_endpoint(base_url)?;
        let form = request.to_form_pairs();

        let response = self
            .http_client
            .post(endpoint)
            .form(&form)
            .send()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        if (200..300).contains(&status) {
            return serde_json::from_str::<TokenResponse>(&body).map_err(|error| {
                VaultwardenError::Decode(format!("invalid token response: {error}"))
            });
        }

        if let Ok(token_error) = serde_json::from_str::<TokenErrorResponse>(&body) {
            return Err(VaultwardenError::TokenRejected {
                status,
                error: token_error,
            });
        }

        Err(Self::api_error(status, body))
    }

    pub async fn send_email_login(
        &self,
        base_url: &str,
        request: SendEmailLoginRequest,
    ) -> VaultwardenResult<()> {
        let endpoint = self.send_email_login_endpoint(base_url)?;

        let response = self
            .http_client
            .post(endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        if (200..300).contains(&status) {
            return Ok(());
        }

        Err(Self::api_error(status, body))
    }

    pub async fn verify_email_token(
        &self,
        base_url: &str,
        request: VerifyEmailTokenRequest,
    ) -> VaultwardenResult<()> {
        let endpoint = self.verify_email_token_endpoint(base_url)?;

        let response = self
            .http_client
            .post(endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| VaultwardenError::Transport(error.to_string()))?;

        if (200..300).contains(&status) {
            return Ok(());
        }

        Err(Self::api_error(status, body))
    }

    pub async fn sync(
        &self,
        base_url: &str,
        _access_token: &str,
    ) -> VaultwardenResult<SyncResponse> {
        self.sync_endpoint(base_url)?;
        Err(VaultwardenError::Transport(String::from(
            "sync is not implemented yet",
        )))
    }

    fn validated_base_url<'a>(base_url: &'a str) -> VaultwardenResult<&'a str> {
        VaultwardenConfig::validate_base_url(base_url)?;
        Ok(base_url.trim_end_matches('/'))
    }

    fn api_error(status: u16, body: String) -> VaultwardenError {
        let message = Self::extract_error_message(&body)
            .unwrap_or_else(|| format!("request failed with status {status}"));

        let body = if body.trim().is_empty() {
            None
        } else {
            Some(body)
        };

        VaultwardenError::ApiError {
            status,
            message,
            body,
        }
    }

    fn extract_error_message(body: &str) -> Option<String> {
        if body.trim().is_empty() {
            return None;
        }

        let value: serde_json::Value = serde_json::from_str(body).ok()?;

        if let Some(description) = value.get("error_description").and_then(|v| v.as_str()) {
            return Some(String::from(description));
        }

        if let Some(error) = value.get("error").and_then(|v| v.as_str()) {
            return Some(String::from(error));
        }

        if let Some(message) = value.get("message").and_then(|v| v.as_str()) {
            return Some(String::from(message));
        }

        None
    }
}
