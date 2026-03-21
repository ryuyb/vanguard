use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use tokio::sync::Mutex;

use crate::infrastructure::icon::cache::IconCache;
use crate::infrastructure::icon::downloader::{IconDownloadError, IconDownloader};
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Application name for cache directory
const APP_NAME: &str = "vanguard";

/// Type alias for pending icon requests to reduce complexity.
type PendingRequestMap = HashMap<String, Arc<tokio::sync::Mutex<Option<Vec<u8>>>>>;

/// Service for managing icon caching and retrieval.
///
/// This service coordinates between disk cache and HTTP downloads,
/// with request deduplication for concurrent requests.
pub struct IconService {
    cache: IconCache,
    downloader: Arc<Mutex<Option<IconDownloader>>>,
    pending_requests: Arc<Mutex<PendingRequestMap>>,
}

impl IconService {
    /// Creates a new IconService.
    pub fn new() -> AppResult<Self> {
        let cache = IconCache::new(APP_NAME).map_err(|e| AppError::InternalUnexpected {
            message: format!("Failed to initialize icon cache: {}", e),
        })?;

        Ok(Self {
            cache,
            downloader: Arc::new(Mutex::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Sets the icon downloader with the given base URL.
    ///
    /// # Arguments
    /// * `base_url` - The vault server base URL
    pub async fn set_downloader(&self, base_url: &str) {
        let mut downloader = self.downloader.lock().await;
        *downloader = Some(IconDownloader::new(base_url));
    }

    /// Gets an icon for the given hostname.
    ///
    /// This method:
    /// 1. Checks disk cache first
    /// 2. If not cached, checks for pending requests (deduplication)
    /// 3. If no pending request, starts a download
    /// 4. Caches successful downloads
    /// 5. Returns base64-encoded icon data
    ///
    /// # Arguments
    /// * `hostname` - The hostname (e.g., "google.com")
    ///
    /// # Returns
    /// * `Ok(Some(base64))` - Icon found or downloaded successfully
    /// * `Ok(None)` - Icon not available
    /// * `Err(...)` - Error during operation
    pub async fn get_icon(&self, hostname: &str) -> AppResult<Option<String>> {
        // Validate hostname
        if hostname.is_empty() || !hostname.contains('.') {
            return Ok(None);
        }

        // 1. Check disk cache
        match self.cache.get(hostname) {
            Ok(Some(data)) => {
                // Cache hit - return base64 encoded data
                return Ok(Some(base64::engine::general_purpose::STANDARD.encode(data)));
            }
            Ok(None) => {
                // Cache miss - continue to download
            }
            Err(e) => {
                log::warn!("Icon cache read error for {}: {}", hostname, e);
                // Continue to download even if cache fails
            }
        }

        // 2. Check for pending request (deduplication)
        let pending_key = hostname.to_lowercase();
        let pending_mutex = {
            let mut pending = self.pending_requests.lock().await;
            pending
                .entry(pending_key.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None)))
                .clone()
        };

        // 3. Acquire lock on this specific hostname
        let mut result_guard = pending_mutex.lock().await;

        // Check if another task already completed the download
        if let Some(data) = result_guard.as_ref() {
            return Ok(Some(base64::engine::general_purpose::STANDARD.encode(data)));
        }

        // 4. Download the icon
        let data = self.download_icon(hostname).await?;

        // 5. Cache successful downloads
        if let Some(ref bytes) = data {
            if let Err(e) = self.cache.put(hostname, bytes) {
                log::warn!("Failed to cache icon for {}: {}", hostname, e);
            }
        }

        // 6. Store result and return
        let result = data
            .as_ref()
            .map(|d| base64::engine::general_purpose::STANDARD.encode(d));
        *result_guard = data;

        // Clean up pending request
        self.pending_requests.lock().await.remove(&pending_key);

        Ok(result)
    }

    /// Downloads an icon from the remote server.
    async fn download_icon(&self, hostname: &str) -> AppResult<Option<Vec<u8>>> {
        let downloader = self.downloader.lock().await;

        let downloader = match downloader.as_ref() {
            Some(d) => d,
            None => {
                // Downloader not initialized (user not logged in) - return None silently
                log::debug!(
                    "Icon downloader not initialized, skipping download for {}",
                    hostname
                );
                return Ok(None);
            }
        };

        match downloader.download(hostname).await {
            Ok(data) => Ok(data),
            Err(IconDownloadError::HttpError(404, _)) => Ok(None),
            Err(e) => {
                log::warn!("Failed to download icon for {}: {}", hostname, e);
                Ok(None)
            }
        }
    }

    /// Clears the icon cache.
    pub fn clear_cache(&self) -> AppResult<()> {
        self.cache
            .clear()
            .map_err(|e| AppError::InternalUnexpected {
                message: format!("Failed to clear icon cache: {}", e),
            })
    }

    /// Returns the cache directory path.
    pub fn cache_dir(&self) -> &std::path::Path {
        self.cache.cache_dir()
    }
}

impl Default for IconService {
    fn default() -> Self {
        // This will panic if cache initialization fails, but it's only used in tests
        Self::new().expect("Failed to create IconService")
    }
}

#[cfg(test)]
mod tests {
    fn is_valid_hostname(hostname: &str) -> bool {
        !hostname.is_empty() && hostname.contains('.')
    }

    #[test]
    fn test_hostname_validation() {
        // Empty hostname should be rejected
        assert!(!is_valid_hostname(""));

        // Hostname without dot should be rejected
        assert!(!is_valid_hostname("localhost"));

        // Valid hostname with dot should be accepted
        assert!(is_valid_hostname("google.com"));
        assert!(is_valid_hostname("sub.example.com"));
    }
}
