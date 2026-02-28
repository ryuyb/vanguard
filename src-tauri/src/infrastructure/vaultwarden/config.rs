use super::error::{VaultwardenError, VaultwardenResult};

#[derive(Debug, Clone)]
pub struct VaultwardenConfig {
    pub client_id: String,
    pub scope: String,
    pub device_identifier: String,
    pub device_name: String,
    pub device_type: String,
    pub allow_invalid_certs: bool,
}

impl VaultwardenConfig {
    pub fn new() -> Self {
        Self {
            client_id: String::from("desktop"),
            scope: String::from("api offline_access"),
            device_identifier: uuid::Uuid::new_v4().to_string(),
            device_name: String::from("vanguard-desktop"),
            device_type: String::from("23"),
            allow_invalid_certs: false,
        }
    }

    pub fn validate_base_url(base_url: &str) -> VaultwardenResult<()> {
        let base_url = base_url.trim();

        if base_url.is_empty() {
            return Err(VaultwardenError::MissingBaseUrl);
        }

        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(VaultwardenError::InvalidEndpoint(
                "base_url must start with http:// or https://",
            ));
        }

        Ok(())
    }
}

impl Default for VaultwardenConfig {
    fn default() -> Self {
        Self::new()
    }
}
