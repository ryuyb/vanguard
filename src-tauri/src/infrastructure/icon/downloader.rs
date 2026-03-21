use std::time::Duration;

use reqwest::Client;

use crate::support::error::AppError;

/// HTTP timeout for icon downloads
const DOWNLOAD_TIMEOUT_SECONDS: u64 = 10;

/// Downloads icons from icon servers.
pub struct IconDownloader {
    client: Client,
    icon_server_url: String,
    is_official: bool,
}

impl IconDownloader {
    /// Creates a new IconDownloader.
    ///
    /// # Arguments
    /// * `base_url` - The vault server base URL (e.g., "https://vault.bitwarden.com")
    pub fn new(base_url: &str) -> Self {
        let normalized_url = base_url.trim().to_lowercase();
        let is_official =
            normalized_url.contains("bitwarden.com") || normalized_url.contains("bitwarden.eu");

        let icon_server_url = if is_official {
            String::from("https://icons.bitwarden.net")
        } else {
            base_url.trim().trim_end_matches('/').to_string()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECONDS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            icon_server_url,
            is_official,
        }
    }

    /// Downloads an icon for the given hostname.
    ///
    /// # Arguments
    /// * `hostname` - The hostname (e.g., "google.com")
    ///
    /// # Returns
    /// * `Ok(Some(bytes))` - Icon downloaded successfully
    /// * `Ok(None)` - Icon not found (404) or invalid
    /// * `Err(...)` - Network or other error
    pub async fn download(&self, hostname: &str) -> Result<Option<Vec<u8>>, IconDownloadError> {
        let url = self.build_icon_url(hostname);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| IconDownloadError::NetworkError(e.to_string()))?;

        // Handle 404 - icon not available
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // Handle other errors
        if !response.status().is_success() {
            return Err(IconDownloadError::HttpError(
                response.status().as_u16(),
                response.status().to_string(),
            ));
        }

        // Read response body
        let bytes = response
            .bytes()
            .await
            .map_err(|e| IconDownloadError::NetworkError(e.to_string()))?;

        // Return the bytes directly without validation
        // The icon server may return different image formats
        Ok(Some(bytes.to_vec()))
    }

    /// Builds the icon URL based on the server type.
    fn build_icon_url(&self, hostname: &str) -> String {
        // Sanitize hostname - remove any path components or query strings
        let sanitized = hostname
            .trim()
            .to_lowercase()
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();

        if self.is_official {
            format!("{}/{}/icon.png", self.icon_server_url, sanitized)
        } else {
            format!("{}/icons/{}/icon.png", self.icon_server_url, sanitized)
        }
    }

    /// Returns the icon server URL.
    pub fn icon_server_url(&self) -> &str {
        &self.icon_server_url
    }

    /// Returns whether this is an official Bitwarden server.
    pub fn is_official(&self) -> bool {
        self.is_official
    }
}

/// Errors that can occur during icon download.
#[derive(Debug)]
pub enum IconDownloadError {
    NetworkError(String),
    HttpError(u16, String),
}

impl std::fmt::Display for IconDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconDownloadError::NetworkError(e) => write!(f, "Network error: {}", e),
            IconDownloadError::HttpError(code, msg) => {
                write!(f, "HTTP error {}: {}", code, msg)
            }
        }
    }
}

impl std::error::Error for IconDownloadError {}

impl From<IconDownloadError> for AppError {
    fn from(error: IconDownloadError) -> Self {
        AppError::InternalUnexpected {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_icon_url_official() {
        let downloader = IconDownloader::new("https://vault.bitwarden.com");
        assert!(downloader.is_official());
        assert_eq!(downloader.icon_server_url(), "https://icons.bitwarden.net");

        let url = downloader.build_icon_url("google.com");
        assert_eq!(url, "https://icons.bitwarden.net/google.com/icon.png");
    }

    #[test]
    fn test_build_icon_url_custom() {
        let downloader = IconDownloader::new("https://vault.example.com");
        assert!(!downloader.is_official());
        assert_eq!(downloader.icon_server_url(), "https://vault.example.com");

        let url = downloader.build_icon_url("google.com");
        assert_eq!(url, "https://vault.example.com/icons/google.com/icon.png");
    }

    #[test]
    fn test_build_icon_url_sanitization() {
        let downloader = IconDownloader::new("https://vault.bitwarden.com");

        // Should strip paths
        let url = downloader.build_icon_url("google.com/path/to/something");
        assert_eq!(url, "https://icons.bitwarden.net/google.com/icon.png");

        // Should strip ports
        let url = downloader.build_icon_url("google.com:8080");
        assert_eq!(url, "https://icons.bitwarden.net/google.com/icon.png");
    }
}
