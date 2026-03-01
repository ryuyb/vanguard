use super::config::VaultwardenConfig;
use super::endpoints::VaultwardenEndpoints;
use super::error::{VaultwardenError, VaultwardenResult};
use super::models::{
    PasswordLoginRequest, PreloginRequest, PreloginResponse, RefreshTokenRequest,
    RevisionDateResponse, SendEmailLoginRequest, SyncCipher, SyncFolder, SyncResponse, SyncSend,
    TokenErrorResponse, TokenRequest, TokenResponse, VerifyEmailTokenRequest,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct VaultwardenClient {
    config: VaultwardenConfig,
    http_client: reqwest::Client,
}

impl VaultwardenClient {
    pub fn new(config: VaultwardenConfig) -> VaultwardenResult<Self> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.allow_invalid_certs)
            .connect_timeout(Duration::from_millis(config.http_connect_timeout_ms.max(1)))
            .timeout(Duration::from_millis(config.http_request_timeout_ms.max(1)))
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

    pub fn revision_date_endpoint(&self, base_url: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::revision_date(base_url))
    }

    pub fn cipher_endpoint(&self, base_url: &str, cipher_id: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::cipher(base_url, cipher_id))
    }

    pub fn folder_endpoint(&self, base_url: &str, folder_id: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::folder(base_url, folder_id))
    }

    pub fn send_endpoint(&self, base_url: &str, send_id: &str) -> VaultwardenResult<String> {
        let base_url = Self::validated_base_url(base_url)?;
        Ok(VaultwardenEndpoints::send(base_url, send_id))
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
        access_token: &str,
        exclude_domains: bool,
    ) -> VaultwardenResult<SyncResponse> {
        let endpoint = self.sync_endpoint(base_url)?;

        let response = self
            .http_client
            .get(endpoint.as_str())
            .bearer_auth(access_token)
            .header("Bitwarden-Client-Version", "2024.12.0")
            .query(&[(
                "excludeDomains",
                if exclude_domains { "true" } else { "false" },
            )])
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

        match serde_json::from_str::<SyncResponse>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(error) => {
                let line = error.line();
                let column = error.column();
                let snippet = json_error_snippet_redacted(&body, line, column, 220);
                log::error!(
                    target: "vanguard::vaultwarden",
                    "sync decode failed endpoint={} status={} body_len={} line={} column={} snippet={}",
                    endpoint,
                    status,
                    body.len(),
                    line,
                    column,
                    snippet
                );
                Err(VaultwardenError::Decode(format!(
                    "invalid sync response: {error}"
                )))
            }
        }
    }

    pub async fn revision_date(
        &self,
        base_url: &str,
        access_token: &str,
    ) -> VaultwardenResult<i64> {
        let endpoint = self.revision_date_endpoint(base_url)?;

        let response = self
            .http_client
            .get(endpoint)
            .bearer_auth(access_token)
            .header("Bitwarden-Client-Version", "2024.12.0")
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

        parse_revision_date(&body).ok_or_else(|| {
            VaultwardenError::Decode(String::from(
                "invalid revision-date response: expected integer timestamp in milliseconds",
            ))
        })
    }

    pub async fn get_cipher(
        &self,
        base_url: &str,
        access_token: &str,
        cipher_id: &str,
    ) -> VaultwardenResult<SyncCipher> {
        let endpoint = self.cipher_endpoint(base_url, cipher_id)?;

        let response = self
            .http_client
            .get(endpoint.as_str())
            .bearer_auth(access_token)
            .header("Bitwarden-Client-Version", "2024.12.0")
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

        match serde_json::from_str::<SyncCipher>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(error) => {
                let line = error.line();
                let column = error.column();
                let snippet = json_error_snippet_redacted(&body, line, column, 220);
                log::error!(
                    target: "vanguard::vaultwarden",
                    "cipher decode failed endpoint={} status={} body_len={} line={} column={} snippet={}",
                    endpoint,
                    status,
                    body.len(),
                    line,
                    column,
                    snippet
                );
                Err(VaultwardenError::Decode(format!(
                    "invalid cipher response: {error}"
                )))
            }
        }
    }

    pub async fn get_folder(
        &self,
        base_url: &str,
        access_token: &str,
        folder_id: &str,
    ) -> VaultwardenResult<SyncFolder> {
        let endpoint = self.folder_endpoint(base_url, folder_id)?;

        let response = self
            .http_client
            .get(endpoint.as_str())
            .bearer_auth(access_token)
            .header("Bitwarden-Client-Version", "2024.12.0")
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

        match serde_json::from_str::<SyncFolder>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(error) => {
                let line = error.line();
                let column = error.column();
                let snippet = json_error_snippet_redacted(&body, line, column, 220);
                log::error!(
                    target: "vanguard::vaultwarden",
                    "folder decode failed endpoint={} status={} body_len={} line={} column={} snippet={}",
                    endpoint,
                    status,
                    body.len(),
                    line,
                    column,
                    snippet
                );
                Err(VaultwardenError::Decode(format!(
                    "invalid folder response: {error}"
                )))
            }
        }
    }

    pub async fn get_send(
        &self,
        base_url: &str,
        access_token: &str,
        send_id: &str,
    ) -> VaultwardenResult<SyncSend> {
        let endpoint = self.send_endpoint(base_url, send_id)?;

        let response = self
            .http_client
            .get(endpoint.as_str())
            .bearer_auth(access_token)
            .header("Bitwarden-Client-Version", "2024.12.0")
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

        match serde_json::from_str::<SyncSend>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(error) => {
                let line = error.line();
                let column = error.column();
                let snippet = json_error_snippet_redacted(&body, line, column, 220);
                log::error!(
                    target: "vanguard::vaultwarden",
                    "send decode failed endpoint={} status={} body_len={} line={} column={} snippet={}",
                    endpoint,
                    status,
                    body.len(),
                    line,
                    column,
                    snippet
                );
                Err(VaultwardenError::Decode(format!(
                    "invalid send response: {error}"
                )))
            }
        }
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

        let payload: BasicErrorPayload = serde_json::from_str(body).ok()?;
        first_non_empty(vec![
            payload.error_description,
            payload.error,
            payload.message,
        ])
    }
}

fn first_non_empty(values: Vec<Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
}

fn parse_revision_date(body: &str) -> Option<i64> {
    let parsed: RevisionDateResponse = serde_json::from_str(body).ok()?;
    parsed.to_revision_ms()
}

fn json_error_snippet_redacted(body: &str, line: usize, column: usize, radius: usize) -> String {
    if body.is_empty() {
        return String::from("<empty>");
    }
    let offset = line_col_to_byte_offset(body, line, column).unwrap_or(body.len());
    let start = offset.saturating_sub(radius);
    let end = (offset + radius).min(body.len());
    let snippet = &body[start..end];
    let redacted = redact_json_string_contents(snippet);
    redacted.replace('\n', "\\n").replace('\r', "\\r")
}

fn line_col_to_byte_offset(body: &str, line: usize, column: usize) -> Option<usize> {
    let target_line = line.max(1);
    let target_column = column.max(1);

    let mut current_line = 1usize;
    let mut line_start = 0usize;
    for (idx, ch) in body.char_indices() {
        if current_line == target_line {
            line_start = idx;
            break;
        }
        if ch == '\n' {
            current_line += 1;
            line_start = idx + 1;
        }
    }
    if current_line != target_line {
        return None;
    }

    let line_slice = &body[line_start..];
    let mut current_column = 1usize;
    for (idx, _) in line_slice.char_indices() {
        if current_column == target_column {
            return Some(line_start + idx);
        }
        current_column += 1;
    }

    Some(body.len())
}

fn redact_json_string_contents(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in input.chars() {
        if in_string {
            if escaped {
                escaped = false;
                output.push('*');
                continue;
            }
            match ch {
                '\\' => {
                    escaped = true;
                    output.push('*');
                }
                '"' => {
                    in_string = false;
                    output.push('"');
                }
                _ => output.push('*'),
            }
        } else if ch == '"' {
            in_string = true;
            output.push('"');
        } else if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            output.push(' ');
        } else {
            output.push(ch);
        }
    }

    output
}

#[derive(serde::Deserialize)]
struct BasicErrorPayload {
    error_description: Option<String>,
    error: Option<String>,
    message: Option<String>,
}
