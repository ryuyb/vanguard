#![allow(clippy::unused_unit)]

use tauri::window::Color;
use tauri::{Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use crate::interfaces::tauri::desktop::constants::{
    SPOTLIGHT_HEIGHT, SPOTLIGHT_PAGE_PATH, SPOTLIGHT_WIDTH, SPOTLIGHT_WINDOW_LABEL,
};
use crate::interfaces::tauri::desktop::window_placement::WindowPlacementPolicy;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt as PanelManagerExt, PanelHandle, PanelLevel,
    StyleMask, WebviewWindowExt as WebviewPanelExt,
};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(SpotlightPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    })

    panel_event!(SpotlightPanelEventHandler {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> (),
    })
}

pub(super) struct SpotlightFeature;

impl SpotlightFeature {
    pub(super) fn ensure_window<R: Runtime, M: Manager<R>>(
        manager: &M,
    ) -> tauri::Result<WebviewWindow<R>> {
        if let Some(window) = manager.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
            return Ok(window);
        }

        let spotlight_window = Self::build_window(manager)?;
        Self::bind_blur_hide(spotlight_window.clone());
        Ok(spotlight_window)
    }

    pub(super) fn register_shortcut<R: Runtime>(
        app: &tauri::App<R>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spotlight_shortcut =
            Shortcut::new(Some(Modifiers::SHIFT | Modifiers::CONTROL), Code::Space);

        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(spotlight_shortcut)?
                .with_handler(move |app_handle, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    Self::toggle(app_handle);
                })
                .build(),
        )?;

        Ok(())
    }

    pub(super) fn toggle<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
        let spotlight_window = match Self::ensure_window(app_handle) {
            Ok(window) => window,
            Err(error) => {
                log::warn!(
                    target: "vanguard::spotlight",
                    "failed to ensure spotlight window: {error}"
                );
                return;
            }
        };

        #[cfg(target_os = "macos")]
        if Self::toggle_panel(app_handle, &spotlight_window) {
            return;
        }

        Self::toggle_window(app_handle, &spotlight_window);
    }

    fn toggle_window<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        spotlight_window: &WebviewWindow<R>,
    ) {
        match spotlight_window.is_visible() {
            Ok(true) => {
                if let Err(error) = spotlight_window.hide() {
                    log::warn!(
                        target: "vanguard::shortcut",
                        "failed to hide spotlight window: {error}"
                    );
                }
            }
            Ok(false) => {
                WindowPlacementPolicy::place_spotlight_window(app_handle, spotlight_window);
                if let Err(error) = spotlight_window.show() {
                    log::warn!(
                        target: "vanguard::shortcut",
                        "failed to show spotlight window: {error}"
                    );
                    return;
                }
                if let Err(error) = spotlight_window.set_focus() {
                    log::warn!(
                        target: "vanguard::shortcut",
                        "failed to focus spotlight window: {error}"
                    );
                }
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::shortcut",
                    "failed to read spotlight visibility: {error}"
                );
            }
        }
    }

    fn build_window<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<WebviewWindow<R>> {
        WebviewWindowBuilder::new(
            manager,
            SPOTLIGHT_WINDOW_LABEL,
            WebviewUrl::App(SPOTLIGHT_PAGE_PATH.into()),
        )
        .title("Spotlight")
        .visible(false)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .resizable(false)
        .skip_taskbar(true)
        .inner_size(SPOTLIGHT_WIDTH, SPOTLIGHT_HEIGHT)
        .build()
    }

    fn bind_blur_hide<R: Runtime>(spotlight_window: WebviewWindow<R>) {
        let spotlight_window_for_events = spotlight_window.clone();
        spotlight_window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if let Err(error) = spotlight_window_for_events.hide() {
                    log::warn!(
                        target: "vanguard::spotlight",
                        "failed to hide spotlight window on blur: {error}"
                    );
                }
            }
        });
    }

    #[cfg(target_os = "macos")]
    fn toggle_panel<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        spotlight_window: &WebviewWindow<R>,
    ) -> bool {
        let spotlight_panel = match app_handle
            .get_webview_panel(SPOTLIGHT_WINDOW_LABEL)
            .or_else(|_| spotlight_window.to_spotlight_panel())
        {
            Ok(panel) => panel,
            Err(error) => {
                log::warn!(
                    target: "vanguard::spotlight",
                    "failed to get/create spotlight panel: {error}"
                );
                return false;
            }
        };

        if spotlight_panel.is_visible() {
            spotlight_panel.hide();
            return true;
        }

        WindowPlacementPolicy::place_spotlight_window(app_handle, spotlight_window);
        spotlight_panel.show_and_make_key();
        true
    }
}

#[cfg(target_os = "macos")]
trait SpotlightWebviewWindowExt<R: Runtime> {
    fn to_spotlight_panel(&self) -> tauri::Result<PanelHandle<R>>;
}

#[cfg(target_os = "macos")]
impl<R: Runtime> SpotlightWebviewWindowExt<R> for WebviewWindow<R> {
    fn to_spotlight_panel(&self) -> tauri::Result<PanelHandle<R>> {
        let spotlight_panel = self.to_panel::<SpotlightPanel<R>>()?;

        spotlight_panel.set_level(PanelLevel::Floating.value());
        spotlight_panel.set_collection_behavior(
            CollectionBehavior::new()
                .full_screen_auxiliary()
                .move_to_active_space()
                .value(),
        );
        spotlight_panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

        let handler = SpotlightPanelEventHandler::new();
        let app_handle = self.app_handle().clone();
        handler.window_did_resign_key(move |_| {
            if let Ok(panel) = app_handle.get_webview_panel(SPOTLIGHT_WINDOW_LABEL) {
                if panel.is_visible() {
                    panel.hide();
                }
            }
        });
        spotlight_panel.set_event_handler(Some(handler.as_ref()));

        Ok(spotlight_panel)
    }
}
