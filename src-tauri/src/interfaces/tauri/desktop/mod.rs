mod constants;
mod main_window;
mod monitor;
mod shortcut_utils;
pub mod spotlight;
pub mod tray;
mod tray_click_snapshot;
mod tray_i18n;
mod window_placement;

use main_window::MainWindowFeature;
use spotlight::SpotlightFeature;
use tauri::Runtime;
use tray::TrayFeature;

pub fn install_desktop_features<R: Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    DesktopBootstrap::new(app).install()
}

pub fn open_main_window<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    MainWindowFeature::open_from_shortcut(app_handle);
}

struct DesktopBootstrap<'a, R: Runtime> {
    app: &'a tauri::App<R>,
}

impl<'a, R: Runtime> DesktopBootstrap<'a, R> {
    fn new(app: &'a tauri::App<R>) -> Self {
        Self { app }
    }

    fn install(self) -> Result<(), Box<dyn std::error::Error>> {
        TrayFeature::install(self.app)?;
        MainWindowFeature::bind_close_to_tray(self.app);
        SpotlightFeature::ensure_window(self.app)?;
        SpotlightFeature::register_shortcut(self.app)?;
        Ok(())
    }
}
