use tauri::window::Color;
use tauri::{Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_positioner::{Position, WindowExt};

const SPOTLIGHT_WINDOW_LABEL: &str = "spotlight";
const SPOTLIGHT_PAGE_PATH: &str = "spotlight.html";
const SPOTLIGHT_WIDTH: f64 = 980.0;
const SPOTLIGHT_HEIGHT: f64 = 620.0;

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

    WebviewWindowBuilder::new(
        app,
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
            .with_handler(move |app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                toggle_spotlight_window(app);
            })
            .build(),
    )?;

    Ok(())
}

fn toggle_spotlight_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(spotlight_window) = app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) else {
        log::warn!(
            target: "vanguard::shortcut",
            "spotlight window not found"
        );
        return;
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
