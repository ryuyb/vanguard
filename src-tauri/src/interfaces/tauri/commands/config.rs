use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::bootstrap::config::AppConfig;
use crate::interfaces::tauri::desktop::tray::TrayFeature;
use crate::support::error::{AppError, ErrorPayload};
use crate::support::redaction::redact_sensitive;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
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
    pub spotlight_autofill: bool,
}

impl From<AppConfig> for AppConfigDto {
    fn from(config: AppConfig) -> Self {
        Self {
            device_identifier: config.device_identifier,
            allow_invalid_certs: config.allow_invalid_certs,
            sync_poll_interval_seconds: config.sync_poll_interval_seconds,
            locale: config.locale,
            launch_on_login: config.launch_on_login,
            show_website_icon: config.show_website_icon,
            quick_access_shortcut: config.quick_access_shortcut,
            lock_shortcut: config.lock_shortcut,
            require_master_password_interval: config.require_master_password_interval,
            lock_on_sleep: config.lock_on_sleep,
            idle_auto_lock_delay: config.idle_auto_lock_delay,
            clipboard_clear_delay: config.clipboard_clear_delay,
            spotlight_autofill: config.spotlight_autofill,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_on_login: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_website_icon: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_access_shortcut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_shortcut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_master_password_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_on_sleep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_auto_lock_delay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_clear_delay: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spotlight_autofill: Option<bool>,
}

fn log_command_error(command: &str, error: AppError) -> ErrorPayload {
    let payload = error.to_payload();
    let sanitized = redact_sensitive(&payload.message);
    log::error!(
        target: "vanguard::tauri::config",
        "{command} failed: [{}] {}",
        payload.code,
        sanitized
    );
    payload
}

#[tauri::command]
#[specta::specta]
pub fn config_get_app_config(app_handle: AppHandle) -> Result<AppConfigDto, ErrorPayload> {
    let config = AppConfig::load(&app_handle)
        .map_err(|error| log_command_error("config_get_app_config", error))?;
    Ok(config.into())
}

#[tauri::command]
#[specta::specta]
pub fn config_update_app_config(
    app_handle: AppHandle,
    request: UpdateAppConfigRequest,
) -> Result<AppConfigDto, ErrorPayload> {
    use serde_json::json;
    use tauri_plugin_store::StoreExt;

    let store = app_handle.store("app-config.json").map_err(|error| {
        log_command_error(
            "config_update_app_config",
            AppError::InternalUnexpected {
                message: format!("failed to open config store: {error}"),
            },
        )
    })?;

    let locale_changed = request.locale.is_some();

    if let Some(locale) = request.locale {
        store.set("locale".to_string(), json!(locale));
    }
    if let Some(launch_on_login) = request.launch_on_login {
        store.set("launch_on_login".to_string(), json!(launch_on_login));
    }
    if let Some(show_website_icon) = request.show_website_icon {
        store.set("show_website_icon".to_string(), json!(show_website_icon));
    }
    if let Some(quick_access_shortcut) = request.quick_access_shortcut {
        store.set(
            "quick_access_shortcut".to_string(),
            json!(quick_access_shortcut),
        );
    }
    if let Some(lock_shortcut) = request.lock_shortcut {
        store.set("lock_shortcut".to_string(), json!(lock_shortcut));
    }
    if let Some(require_master_password_interval) = request.require_master_password_interval {
        store.set(
            "require_master_password_interval".to_string(),
            json!(require_master_password_interval),
        );
    }
    if let Some(lock_on_sleep) = request.lock_on_sleep {
        store.set("lock_on_sleep".to_string(), json!(lock_on_sleep));
    }
    if let Some(idle_auto_lock_delay) = request.idle_auto_lock_delay {
        store.set(
            "idle_auto_lock_delay".to_string(),
            json!(idle_auto_lock_delay),
        );
    }
    if let Some(clipboard_clear_delay) = request.clipboard_clear_delay {
        store.set(
            "clipboard_clear_delay".to_string(),
            json!(clipboard_clear_delay),
        );
    }
    if let Some(spotlight_autofill) = request.spotlight_autofill {
        store.set("spotlight_autofill".to_string(), json!(spotlight_autofill));
    }

    store.save().map_err(|error| {
        log_command_error(
            "config_update_app_config",
            AppError::InternalUnexpected {
                message: format!("failed to save config store: {error}"),
            },
        )
    })?;

    // 如果语言设置发生变化,更新托盘菜单
    if locale_changed {
        TrayFeature::update_menu(&app_handle);
    }

    let config = AppConfig::load(&app_handle)
        .map_err(|error| log_command_error("config_update_app_config", error))?;
    Ok(config.into())
}

#[tauri::command]
#[specta::specta]
pub fn config_check_text_injection_permission() -> bool {
    use crate::application::ports::text_injection_port::TextInjectionPort;
    use crate::infrastructure::desktop::EnigoTextInjectionAdapter;

    EnigoTextInjectionAdapter::new()
        .map(|adapter| adapter.is_available())
        .unwrap_or(false)
}
