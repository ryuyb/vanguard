use tauri::Runtime;

pub fn install_desktop_features<R: Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::interfaces::tauri::desktop::install_desktop_features(app)
}
