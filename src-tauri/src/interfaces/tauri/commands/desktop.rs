#[tauri::command]
#[specta::specta]
pub fn desktop_open_main_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::interfaces::tauri::desktop::open_main_window(&app_handle);
    Ok(())
}
