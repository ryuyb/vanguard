use std::fs::{self, Metadata};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};

/// Default cache size limit: 100MB
const DEFAULT_CACHE_SIZE_LIMIT_MB: u64 = 100;
/// Default TTL: 30 days
const DEFAULT_CACHE_TTL_DAYS: u64 = 30;

/// Manages disk-based icon caching with LRU eviction and TTL expiration.
pub struct IconCache {
    cache_dir: PathBuf,
    size_limit_bytes: u64,
    ttl: Duration,
}

impl IconCache {
    /// Creates a new IconCache instance.
    ///
    /// # Arguments
    /// * `app_name` - The application name used for the cache directory
    pub fn new(app_name: &str) -> Result<Self, IconCacheError> {
        let cache_dir = dirs::cache_dir()
            .map(|dir| dir.join(app_name).join("icons"))
            .ok_or(IconCacheError::CacheDirNotFound)?;

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).map_err(IconCacheError::IoError)?;

        Ok(Self {
            cache_dir,
            size_limit_bytes: DEFAULT_CACHE_SIZE_LIMIT_MB * 1024 * 1024,
            ttl: Duration::from_secs(DEFAULT_CACHE_TTL_DAYS * 24 * 60 * 60),
        })
    }

    /// Returns the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Gets an icon from cache if it exists and is not expired.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to look up
    ///
    /// # Returns
    /// * `Ok(Some(bytes))` - Icon found and valid
    /// * `Ok(None)` - Icon not found or expired
    /// * `Err(...)` - IO error
    pub fn get(&self, hostname: &str) -> Result<Option<Vec<u8>>, IconCacheError> {
        let cache_path = self.cache_path(hostname);

        if !cache_path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&cache_path).map_err(IconCacheError::IoError)?;

        // Check if expired
        if self.is_expired(&metadata)? {
            // Remove expired entry
            fs::remove_file(&cache_path).map_err(IconCacheError::IoError)?;
            return Ok(None);
        }

        // Update access time to mark as recently used
        self.touch(&cache_path)?;

        // Read and return the icon data
        let data = fs::read(&cache_path).map_err(IconCacheError::IoError)?;
        Ok(Some(data))
    }

    /// Stores an icon in the cache.
    ///
    /// # Arguments
    /// * `hostname` - The hostname for the icon
    /// * `data` - The icon bytes (PNG)
    pub fn put(&self, hostname: &str, data: &[u8]) -> Result<(), IconCacheError> {
        // Ensure cache size limit before adding new entry
        self.ensure_size_limit()?;

        let cache_path = self.cache_path(hostname);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cache_path)
            .map_err(IconCacheError::IoError)?;

        file.write_all(data).map_err(IconCacheError::IoError)?;
        file.flush().map_err(IconCacheError::IoError)?;

        Ok(())
    }

    /// Returns the cache file path for a given hostname.
    fn cache_path(&self, hostname: &str) -> PathBuf {
        let hash = Self::hash_hostname(hostname);
        self.cache_dir.join(format!("{}.png", hash))
    }

    /// Generates a SHA256 hash of the hostname for use as cache key.
    fn hash_hostname(hostname: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(hostname.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Checks if a cache entry is expired based on TTL.
    fn is_expired(&self, metadata: &Metadata) -> Result<bool, IconCacheError> {
        let modified = metadata
            .modified()
            .map_err(|e| IconCacheError::MetadataError(e.to_string()))?;

        let age = SystemTime::now()
            .duration_since(modified)
            .map_err(|e| IconCacheError::MetadataError(e.to_string()))?;

        Ok(age > self.ttl)
    }

    /// Updates the file's modification time to mark it as recently used.
    fn touch(&self, path: &Path) -> Result<(), IconCacheError> {
        // Open file to update access time
        let _ = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(IconCacheError::IoError)?;

        Ok(())
    }

    /// Ensures the cache size is under the limit by evicting oldest entries.
    fn ensure_size_limit(&self) -> Result<(), IconCacheError> {
        let entries = self.list_entries()?;
        let total_size: u64 = entries.iter().map(|(_, meta)| meta.len()).sum();

        if total_size <= self.size_limit_bytes {
            return Ok(());
        }

        // Sort by modification time (oldest first)
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| {
            let time_a = a.1.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let time_b = b.1.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            time_a.cmp(&time_b)
        });

        // Remove oldest entries until under limit
        let mut current_size = total_size;
        for (path, metadata) in sorted_entries {
            if current_size <= self.size_limit_bytes {
                break;
            }

            fs::remove_file(&path).map_err(IconCacheError::IoError)?;
            current_size -= metadata.len();
        }

        Ok(())
    }

    /// Lists all cache entries with their metadata.
    fn list_entries(&self) -> Result<Vec<(PathBuf, Metadata)>, IconCacheError> {
        let mut entries = Vec::new();

        for entry in fs::read_dir(&self.cache_dir).map_err(IconCacheError::IoError)? {
            let entry = entry.map_err(IconCacheError::IoError)?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("png") {
                let metadata = entry.metadata().map_err(IconCacheError::IoError)?;
                entries.push((path, metadata));
            }
        }

        Ok(entries)
    }

    /// Clears all cached icons.
    pub fn clear(&self) -> Result<(), IconCacheError> {
        for entry in fs::read_dir(&self.cache_dir).map_err(IconCacheError::IoError)? {
            let entry = entry.map_err(IconCacheError::IoError)?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("png") {
                fs::remove_file(&path).map_err(IconCacheError::IoError)?;
            }
        }

        Ok(())
    }
}

/// Errors that can occur during cache operations.
#[derive(Debug)]
pub enum IconCacheError {
    CacheDirNotFound,
    IoError(std::io::Error),
    MetadataError(String),
}

impl std::fmt::Display for IconCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconCacheError::CacheDirNotFound => write!(f, "System cache directory not found"),
            IconCacheError::IoError(e) => write!(f, "IO error: {}", e),
            IconCacheError::MetadataError(e) => write!(f, "Metadata error: {}", e),
        }
    }
}

impl std::error::Error for IconCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IconCacheError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache() -> (IconCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache = IconCache {
            cache_dir: temp_dir.path().join("icons"),
            size_limit_bytes: 1024 * 1024, // 1MB for testing
            ttl: Duration::from_secs(60),  // 1 minute for testing
        };
        fs::create_dir_all(&cache.cache_dir).unwrap();
        (cache, temp_dir)
    }

    #[test]
    fn test_hash_hostname() {
        let hash1 = IconCache::hash_hostname("google.com");
        let hash2 = IconCache::hash_hostname("google.com");
        let hash3 = IconCache::hash_hostname("GOOGLE.COM");

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3); // Case insensitive
        assert_eq!(hash1.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_put_and_get() {
        let (cache, _temp) = create_test_cache();
        let data = vec![1, 2, 3, 4, 5];

        // Put
        cache.put("example.com", &data).unwrap();

        // Get
        let result = cache.get("example.com").unwrap();
        assert_eq!(result, Some(data));
    }

    #[test]
    fn test_get_nonexistent() {
        let (cache, _temp) = create_test_cache();

        let result = cache.get("nonexistent.com").unwrap();
        assert_eq!(result, None);
    }
}
