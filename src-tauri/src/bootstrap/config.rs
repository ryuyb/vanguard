use serde_json::json;
use tauri::{Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::support::error::AppError;
use crate::support::result::AppResult;

const STORE_PATH: &str = "app-config.json";
const KEY_DEVICE_IDENTIFIER: &str = "device_identifier";
const KEY_ALLOW_INVALID_CERTS: &str = "allow_invalid_certs";
const KEY_SYNC_POLL_INTERVAL_SECONDS: &str = "sync_poll_interval_seconds";
const KEY_LOCALE: &str = "locale";
const KEY_LAUNCH_ON_LOGIN: &str = "launch_on_login";
const KEY_SHOW_WEBSITE_ICON: &str = "show_website_icon";
const KEY_QUICK_ACCESS_SHORTCUT: &str = "quick_access_shortcut";
const KEY_LOCK_SHORTCUT: &str = "lock_shortcut";
const KEY_REQUIRE_MASTER_PASSWORD_INTERVAL: &str = "require_master_password_interval";
const KEY_LOCK_ON_SLEEP: &str = "lock_on_sleep";
const KEY_IDLE_AUTO_LOCK_DELAY: &str = "idle_auto_lock_delay";
const KEY_CLIPBOARD_CLEAR_DELAY: &str = "clipboard_clear_delay";
const DEFAULT_SYNC_POLL_INTERVAL_SECONDS: u64 = 60;
const MIN_SYNC_POLL_INTERVAL_SECONDS: u64 = 30;
const MAX_SYNC_POLL_INTERVAL_SECONDS: u64 = 120;
const DEFAULT_LOCALE: &str = "zh";
const SUPPORTED_LOCALES: &[&str] = &["zh", "en"];
const DEFAULT_REQUIRE_MASTER_PASSWORD_INTERVAL: &str = "never";
const SUPPORTED_REQUIRE_MASTER_PASSWORD_INTERVALS: &[&str] = &["1d", "7d", "14d", "30d", "never"];
const DEFAULT_IDLE_AUTO_LOCK_DELAY: &str = "never";
const SUPPORTED_IDLE_AUTO_LOCK_DELAYS: &[&str] = &[
    "1m", "2m", "5m", "10m", "15m", "30m", "1h", "4h", "8h", "never",
];
const DEFAULT_CLIPBOARD_CLEAR_DELAY: &str = "never";
const SUPPORTED_CLIPBOARD_CLEAR_DELAYS: &[&str] = &["10s", "20s", "30s", "1m", "2m", "5m", "never"];

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub device_identifier: String,
    pub allow_invalid_certs: bool,
    pub sync_poll_interval_seconds: u64,
    pub locale: String,
    pub launch_on_login: bool,
    pub show_website_icon: bool,
    pub quick_access_shortcut: String,
    pub lock_shortcut: String,
    pub require_master_password_interval: String,
    pub lock_on_sleep: bool,
    pub idle_auto_lock_delay: String,
    pub clipboard_clear_delay: String,
}

impl AppConfig {
    pub fn load<R: Runtime, M: Manager<R>>(manager: &M) -> AppResult<Self> {
        let store = manager
            .store(STORE_PATH)
            .map_err(|error| AppError::InternalUnexpected {
                message: format!("failed to open config store: {error}"),
            })?;

        let device_identifier = read_store_string(&store, KEY_DEVICE_IDENTIFIER)
            .filter(|value| uuid::Uuid::parse_str(value).is_ok())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let allow_invalid_certs = read_store_bool(&store, KEY_ALLOW_INVALID_CERTS).unwrap_or(false);
        let sync_poll_interval_seconds = clamp_sync_poll_interval(
            read_store_u64(&store, KEY_SYNC_POLL_INTERVAL_SECONDS)
                .unwrap_or(DEFAULT_SYNC_POLL_INTERVAL_SECONDS),
        );
        let locale = validate_locale(
            read_store_string(&store, KEY_LOCALE)
                .as_deref()
                .unwrap_or(DEFAULT_LOCALE),
        );
        let launch_on_login = read_store_bool(&store, KEY_LAUNCH_ON_LOGIN).unwrap_or(false);
        let show_website_icon = read_store_bool(&store, KEY_SHOW_WEBSITE_ICON).unwrap_or(true);
        let quick_access_shortcut =
            read_store_string(&store, KEY_QUICK_ACCESS_SHORTCUT).unwrap_or_default();
        let lock_shortcut = read_store_string(&store, KEY_LOCK_SHORTCUT).unwrap_or_default();
        let require_master_password_interval = validate_require_master_password_interval(
            read_store_string(&store, KEY_REQUIRE_MASTER_PASSWORD_INTERVAL)
                .as_deref()
                .unwrap_or(DEFAULT_REQUIRE_MASTER_PASSWORD_INTERVAL),
        );
        let lock_on_sleep = read_store_bool(&store, KEY_LOCK_ON_SLEEP).unwrap_or(false);
        let idle_auto_lock_delay = validate_idle_auto_lock_delay(
            read_store_string(&store, KEY_IDLE_AUTO_LOCK_DELAY)
                .as_deref()
                .unwrap_or(DEFAULT_IDLE_AUTO_LOCK_DELAY),
        );
        let clipboard_clear_delay = validate_clipboard_clear_delay(
            read_store_string(&store, KEY_CLIPBOARD_CLEAR_DELAY)
                .as_deref()
                .unwrap_or(DEFAULT_CLIPBOARD_CLEAR_DELAY),
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
        store.set(KEY_LOCALE.to_string(), json!(locale));
        store.set(KEY_LAUNCH_ON_LOGIN.to_string(), json!(launch_on_login));
        store.set(KEY_SHOW_WEBSITE_ICON.to_string(), json!(show_website_icon));
        store.set(
            KEY_QUICK_ACCESS_SHORTCUT.to_string(),
            json!(quick_access_shortcut),
        );
        store.set(KEY_LOCK_SHORTCUT.to_string(), json!(lock_shortcut));
        store.set(
            KEY_REQUIRE_MASTER_PASSWORD_INTERVAL.to_string(),
            json!(require_master_password_interval),
        );
        store.set(KEY_LOCK_ON_SLEEP.to_string(), json!(lock_on_sleep));
        store.set(
            KEY_IDLE_AUTO_LOCK_DELAY.to_string(),
            json!(idle_auto_lock_delay),
        );
        store.set(
            KEY_CLIPBOARD_CLEAR_DELAY.to_string(),
            json!(clipboard_clear_delay),
        );
        store.save().map_err(|error| AppError::InternalUnexpected {
            message: format!("failed to save config store: {error}"),
        })?;

        Ok(Self {
            device_identifier,
            allow_invalid_certs,
            sync_poll_interval_seconds,
            locale,
            launch_on_login,
            show_website_icon,
            quick_access_shortcut,
            lock_shortcut,
            require_master_password_interval,
            lock_on_sleep,
            idle_auto_lock_delay,
            clipboard_clear_delay,
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

fn validate_locale(value: &str) -> String {
    if SUPPORTED_LOCALES.contains(&value) {
        value.to_string()
    } else {
        DEFAULT_LOCALE.to_string()
    }
}

fn validate_require_master_password_interval(value: &str) -> String {
    if SUPPORTED_REQUIRE_MASTER_PASSWORD_INTERVALS.contains(&value) {
        value.to_string()
    } else {
        DEFAULT_REQUIRE_MASTER_PASSWORD_INTERVAL.to_string()
    }
}

fn validate_idle_auto_lock_delay(value: &str) -> String {
    if SUPPORTED_IDLE_AUTO_LOCK_DELAYS.contains(&value) {
        value.to_string()
    } else {
        DEFAULT_IDLE_AUTO_LOCK_DELAY.to_string()
    }
}

fn validate_clipboard_clear_delay(value: &str) -> String {
    if SUPPORTED_CLIPBOARD_CLEAR_DELAYS.contains(&value) {
        value.to_string()
    } else {
        DEFAULT_CLIPBOARD_CLEAR_DELAY.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_locale_accepts_supported_values() {
        assert_eq!(validate_locale("zh"), "zh");
        assert_eq!(validate_locale("en"), "en");
    }

    #[test]
    fn validate_locale_rejects_unsupported_values() {
        assert_eq!(validate_locale("fr"), DEFAULT_LOCALE);
        assert_eq!(validate_locale("invalid"), DEFAULT_LOCALE);
        assert_eq!(validate_locale(""), DEFAULT_LOCALE);
    }

    #[test]
    fn validate_require_master_password_interval_accepts_supported_values() {
        assert_eq!(validate_require_master_password_interval("1d"), "1d");
        assert_eq!(validate_require_master_password_interval("7d"), "7d");
        assert_eq!(validate_require_master_password_interval("14d"), "14d");
        assert_eq!(validate_require_master_password_interval("30d"), "30d");
        assert_eq!(validate_require_master_password_interval("never"), "never");
    }

    #[test]
    fn validate_require_master_password_interval_rejects_unsupported_values() {
        assert_eq!(
            validate_require_master_password_interval("2d"),
            DEFAULT_REQUIRE_MASTER_PASSWORD_INTERVAL
        );
        assert_eq!(
            validate_require_master_password_interval("invalid"),
            DEFAULT_REQUIRE_MASTER_PASSWORD_INTERVAL
        );
    }

    #[test]
    fn validate_idle_auto_lock_delay_accepts_supported_values() {
        assert_eq!(validate_idle_auto_lock_delay("1m"), "1m");
        assert_eq!(validate_idle_auto_lock_delay("5m"), "5m");
        assert_eq!(validate_idle_auto_lock_delay("1h"), "1h");
        assert_eq!(validate_idle_auto_lock_delay("never"), "never");
    }

    #[test]
    fn validate_idle_auto_lock_delay_rejects_unsupported_values() {
        assert_eq!(
            validate_idle_auto_lock_delay("3m"),
            DEFAULT_IDLE_AUTO_LOCK_DELAY
        );
        assert_eq!(
            validate_idle_auto_lock_delay("invalid"),
            DEFAULT_IDLE_AUTO_LOCK_DELAY
        );
    }

    #[test]
    fn validate_clipboard_clear_delay_accepts_supported_values() {
        assert_eq!(validate_clipboard_clear_delay("10s"), "10s");
        assert_eq!(validate_clipboard_clear_delay("1m"), "1m");
        assert_eq!(validate_clipboard_clear_delay("never"), "never");
    }

    #[test]
    fn validate_clipboard_clear_delay_rejects_unsupported_values() {
        assert_eq!(
            validate_clipboard_clear_delay("15s"),
            DEFAULT_CLIPBOARD_CLEAR_DELAY
        );
        assert_eq!(
            validate_clipboard_clear_delay("invalid"),
            DEFAULT_CLIPBOARD_CLEAR_DELAY
        );
    }

    #[test]
    fn clamp_sync_poll_interval_keeps_valid_values() {
        assert_eq!(clamp_sync_poll_interval(60), 60);
        assert_eq!(clamp_sync_poll_interval(30), 30);
        assert_eq!(clamp_sync_poll_interval(120), 120);
    }

    #[test]
    fn clamp_sync_poll_interval_clamps_out_of_range_values() {
        assert_eq!(clamp_sync_poll_interval(10), MIN_SYNC_POLL_INTERVAL_SECONDS);
        assert_eq!(
            clamp_sync_poll_interval(200),
            MAX_SYNC_POLL_INTERVAL_SECONDS
        );
    }
}
