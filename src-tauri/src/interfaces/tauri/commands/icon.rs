use tauri::State;

use crate::bootstrap::app_state::AppState;
use crate::support::error::ErrorPayload;

/// Gets an icon for the given hostname.
///
/// # Arguments
/// * `hostname` - The hostname (e.g., "google.com")
///
/// # Returns
/// * `Ok(Some(base64))` - Icon data as base64-encoded PNG
/// * `Ok(None)` - Icon not available
/// * `Err(...)` - Error during operation
#[tauri::command]
#[specta::specta]
pub async fn get_icon(
    state: State<'_, AppState>,
    hostname: String,
) -> Result<Option<String>, ErrorPayload> {
    log::debug!(target: "vanguard::tauri::icon", "get_icon called for '{}'", hostname);
    let result = state.icon_service().get_icon(&hostname).await;
    match &result {
        Ok(Some(_)) => {
            log::debug!(target: "vanguard::tauri::icon", "get_icon succeeded for '{}'", hostname)
        }
        Ok(None) => {
            log::debug!(target: "vanguard::tauri::icon", "get_icon returned None for '{}'", hostname)
        }
        Err(error) => {
            log::error!(target: "vanguard::tauri::icon", "get_icon failed for '{}': {:?}", hostname, error)
        }
    }
    result.map_err(|error| error.to_payload())
}

/// Clears the icon cache.
///
/// # Returns
/// * `Ok(())` - Cache cleared successfully
/// * `Err(...)` - Error during operation
#[tauri::command]
#[specta::specta]
pub fn clear_icon_cache(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    state.icon_service().clear_cache().map_err(|error| {
        log::error!(
            target: "vanguard::tauri::icon",
            "clear_icon_cache failed: {:?}",
            error
        );
        error.to_payload()
    })
}
