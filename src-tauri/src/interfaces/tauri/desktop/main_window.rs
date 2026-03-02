use tauri::{ActivationPolicy, Manager, Runtime};

use crate::interfaces::tauri::desktop::constants::MAIN_WINDOW_LABEL;
use crate::interfaces::tauri::desktop::window_placement::WindowPlacementPolicy;

pub(super) struct MainWindowFeature;

impl MainWindowFeature {
    pub(super) fn bind_close_to_tray<R: Runtime>(app: &tauri::App<R>) {
        let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            log::warn!(
                target: "vanguard::tray",
                "main window not found, skip close-to-tray binding"
            );
            return;
        };

        let main_window_for_events = main_window.clone();
        #[cfg(target_os = "macos")]
        let app_handle_for_events = app.handle().clone();

        main_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();

                #[cfg(target_os = "macos")]
                if let Err(error) =
                    app_handle_for_events.set_activation_policy(ActivationPolicy::Accessory)
                {
                    log::warn!(
                        target: "vanguard::tray",
                        "failed to set activation policy to accessory on hide: {error}"
                    );
                }

                if let Err(error) = main_window_for_events.hide() {
                    log::warn!(
                        target: "vanguard::tray",
                        "failed to hide main window on close request: {error}"
                    );
                }
            }
        });
    }

    pub(super) fn open_from_tray<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
        #[cfg(target_os = "macos")]
        if let Err(error) = app_handle.set_activation_policy(ActivationPolicy::Regular) {
            log::warn!(
                target: "vanguard::tray",
                "failed to set activation policy to regular on restore: {error}"
            );
        }

        let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL) else {
            log::warn!(
                target: "vanguard::tray",
                "main window not found, cannot restore from tray"
            );
            return;
        };

        WindowPlacementPolicy::recenter_main_window_on_active_monitor(app_handle, &main_window);

        if let Err(error) = main_window.unminimize() {
            log::warn!(
                target: "vanguard::tray",
                "failed to unminimize main window from tray: {error}"
            );
        }
        if let Err(error) = main_window.show() {
            log::warn!(
                target: "vanguard::tray",
                "failed to show main window from tray: {error}"
            );
        }
        if let Err(error) = main_window.set_focus() {
            log::warn!(
                target: "vanguard::tray",
                "failed to focus main window from tray: {error}"
            );
        }
    }
}
