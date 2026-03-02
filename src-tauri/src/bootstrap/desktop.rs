use tauri::window::Color;
use tauri::{Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_positioner::{Position, WindowExt};
#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt as PanelManagerExt, PanelHandle, PanelLevel,
    StyleMask, WebviewWindowExt as WebviewPanelExt,
};

const SPOTLIGHT_WINDOW_LABEL: &str = "spotlight";
const SPOTLIGHT_PAGE_PATH: &str = "spotlight.html";
const SPOTLIGHT_WIDTH: f64 = 980.0;
const SPOTLIGHT_HEIGHT: f64 = 620.0;

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

pub fn install_desktop_features<R: Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let spotlight_window = ensure_spotlight_window(app)?;
    bind_spotlight_blur_hide(spotlight_window);
    register_spotlight_shortcut(app)?;
    Ok(())
}

fn ensure_spotlight_window<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<WebviewWindow<R>> {
    if let Some(window) = app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
        return Ok(window);
    }

    let spotlight_window = build_spotlight_window(app)?;

    Ok(spotlight_window)
}

fn build_spotlight_window<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<WebviewWindow<R>> {
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

fn bind_spotlight_blur_hide<R: Runtime>(spotlight_window: WebviewWindow<R>) {
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

fn register_spotlight_shortcut<R: Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let spotlight_shortcut =
        Shortcut::new(Some(Modifiers::SHIFT | Modifiers::CONTROL), Code::Space);

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcut(spotlight_shortcut)?
            .with_handler(move |app, shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }

                #[cfg(target_os = "macos")]
                if shortcut.matches(Modifiers::SHIFT | Modifiers::CONTROL, Code::Space) {
                    let spotlight_window = match app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
                        Some(window) => window,
                        None => match build_spotlight_window(app) {
                            Ok(window) => {
                                bind_spotlight_blur_hide(window.clone());
                                window
                            }
                            Err(error) => {
                                log::warn!(
                                    target: "vanguard::spotlight",
                                    "failed to create spotlight window for panel conversion: {error}"
                                );
                                return;
                            }
                        },
                    };

                    match app
                        .get_webview_panel(SPOTLIGHT_WINDOW_LABEL)
                        .or_else(|_| spotlight_window.to_spotlight_panel())
                    {
                        Ok(panel) => {
                            if panel.is_visible() {
                                panel.hide();
                            } else {
                                place_spotlight_window(&spotlight_window);
                                panel.show_and_make_key();
                            }
                        }
                        Err(error) => {
                            log::warn!(
                                target: "vanguard::spotlight",
                                "failed to get/create spotlight panel: {error}"
                            );
                        }
                    }
                    return;
                }

                toggle_spotlight_window(app);
            })
            .build(),
    )?;

    Ok(())
}

fn toggle_spotlight_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    #[cfg(target_os = "macos")]
    if toggle_spotlight_panel(app) {
        return;
    }

    let spotlight_window = match app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
        Some(window) => window,
        None => match build_spotlight_window(app) {
            Ok(window) => {
                bind_spotlight_blur_hide(window.clone());
                window
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::shortcut",
                    "failed to create spotlight window: {error}"
                );
                return;
            }
        },
    };

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
            place_spotlight_window(&spotlight_window);
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

#[cfg(target_os = "macos")]
fn toggle_spotlight_panel<R: Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let spotlight_window = match app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
        Some(window) => window,
        None => match build_spotlight_window(app) {
            Ok(window) => {
                bind_spotlight_blur_hide(window.clone());
                window
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::spotlight",
                    "failed to create spotlight window for panel conversion: {error}"
                );
                return false;
            }
        },
    };

    let spotlight_panel = match app
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

    place_spotlight_window(&spotlight_window);
    spotlight_panel.show_and_make_key();
    true
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
        handler.window_did_become_key(|_| {
            log::debug!(target: "vanguard::spotlight", "spotlight panel became key");
        });

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

fn place_spotlight_window<R: Runtime>(spotlight_window: &WebviewWindow<R>) {
    if let Err(error) = spotlight_window
        .as_ref()
        .window()
        .move_window(Position::TopCenter)
    {
        log::warn!(
            target: "vanguard::spotlight",
            "failed to set spotlight window position with positioner: {error}"
        );
    }
}
