use serde_json::Value;
use tauri::{Manager, Runtime};
use tauri_plugin_store::StoreExt;

pub const APP_STORE_PATH: &str = "app.store.json";

pub fn get_app_store_value<R: Runtime, M: Manager<R>>(
    manager: &M,
    key: impl AsRef<str>,
) -> tauri_plugin_store::Result<Option<Value>> {
    let store = manager.store(APP_STORE_PATH)?;
    Ok(store.get(key))
}

pub fn set_app_store_value<R: Runtime, M: Manager<R>>(
    manager: &M,
    key: impl Into<String>,
    value: impl Into<Value>,
) -> tauri_plugin_store::Result<()> {
    let store = manager.store(APP_STORE_PATH)?;
    store.set(key, value);
    store.save()
}

pub fn delete_app_store_value<R: Runtime, M: Manager<R>>(
    manager: &M,
    key: impl AsRef<str>,
) -> tauri_plugin_store::Result<bool> {
    let store = manager.store(APP_STORE_PATH)?;
    let removed = store.delete(key);
    store.save()?;
    Ok(removed)
}
