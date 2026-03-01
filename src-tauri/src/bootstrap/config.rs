use serde_json::json;
use tauri::{Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::support::error::AppError;
use crate::support::result::AppResult;

const STORE_PATH: &str = "app-config.json";
const KEY_DEVICE_IDENTIFIER: &str = "device_identifier";
const KEY_ALLOW_INVALID_CERTS: &str = "allow_invalid_certs";
const KEY_SYNC_POLL_INTERVAL_SECONDS: &str = "sync_poll_interval_seconds";
const DEFAULT_SYNC_POLL_INTERVAL_SECONDS: u64 = 60;
const MIN_SYNC_POLL_INTERVAL_SECONDS: u64 = 30;
const MAX_SYNC_POLL_INTERVAL_SECONDS: u64 = 120;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub device_identifier: String,
    pub allow_invalid_certs: bool,
    pub sync_poll_interval_seconds: u64,
}

impl AppConfig {
    pub fn load<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<Self> {
        let store = manager
            .store(STORE_PATH)
            .map_err(|error| AppError::internal(format!("failed to open config store: {error}")))?;

        let device_identifier = read_store_string(&store, KEY_DEVICE_IDENTIFIER)
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let allow_invalid_certs = read_store_bool(&store, KEY_ALLOW_INVALID_CERTS).unwrap_or(false);
        let sync_poll_interval_seconds = clamp_sync_poll_interval(
            read_store_u64(&store, KEY_SYNC_POLL_INTERVAL_SECONDS)
                .unwrap_or(DEFAULT_SYNC_POLL_INTERVAL_SECONDS),
        );

        store.set(KEY_DEVICE_IDENTIFIER.to_string(), json!(device_identifier));
        store.set(
            KEY_ALLOW_INVALID_CERTS.to_string(),
            json!(allow_invalid_certs),
        );
        store.set(
            KEY_SYNC_POLL_INTERVAL_SECONDS.to_string(),
            json!(sync_poll_interval_seconds),
        );
        store
            .save()
            .map_err(|error| AppError::internal(format!("failed to save config store: {error}")))?;

        Ok(Self {
            device_identifier,
            allow_invalid_certs,
            sync_poll_interval_seconds,
        })
    }
}

fn read_store_string<R: Runtime>(
    store: &tauri_plugin_store::Store<R>,
    key: &str,
) -> Option<String> {
    store
        .get(key)
        .and_then(|value| value.as_str().map(ToString::to_string))
}

fn read_store_bool<R: Runtime>(store: &tauri_plugin_store::Store<R>, key: &str) -> Option<bool> {
    store.get(key).and_then(|value| value.as_bool())
}

fn read_store_u64<R: Runtime>(store: &tauri_plugin_store::Store<R>, key: &str) -> Option<u64> {
    store.get(key).and_then(|value| value.as_u64())
}

fn clamp_sync_poll_interval(value: u64) -> u64 {
    value.clamp(
        MIN_SYNC_POLL_INTERVAL_SECONDS,
        MAX_SYNC_POLL_INTERVAL_SECONDS,
    )
}
