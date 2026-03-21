//! Icon caching infrastructure module.
//!
//! Provides persistent disk-based caching for website icons with LRU eviction
//! and TTL expiration. Icons are stored as PNG files in the system cache directory.

pub mod cache;
pub mod downloader;
pub mod service;

pub use cache::IconCache;
pub use downloader::IconDownloader;
pub use service::IconService;
