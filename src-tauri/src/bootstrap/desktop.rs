use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::window::Color;
use tauri::{ActivationPolicy, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt as PanelManagerExt, PanelHandle, PanelLevel,
    StyleMask, WebviewWindowExt as WebviewPanelExt,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_positioner::{Position, WindowExt};

const SPOTLIGHT_WINDOW_LABEL: &str = "spotlight";
const MAIN_WINDOW_LABEL: &str = "main";
const SPOTLIGHT_PAGE_PATH: &str = "spotlight.html";
const SPOTLIGHT_WIDTH: f64 = 980.0;
const SPOTLIGHT_HEIGHT: f64 = 620.0;
const TRAY_ICON_ID: &str = "vanguard-tray";
const TRAY_MENU_OPEN_VANGUARD_ID: &str = "tray-open-vanguard";
const TRAY_MENU_OPEN_QUICK_ACCESS_ID: &str = "tray-open-quick-access";
const TRAY_MENU_LOCK_ID: &str = "tray-lock";
const TRAY_MENU_SETTINGS_ID: &str = "tray-settings";
const TRAY_MENU_QUIT_ID: &str = "tray-quit";

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
    install_tray_icon(app)?;
    bind_main_window_close_to_tray(app);
    let spotlight_window = ensure_spotlight_window(app)?;
    bind_spotlight_blur_hide(spotlight_window);
    register_spotlight_shortcut(app)?;
    Ok(())
}

fn install_tray_icon<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let open_vanguard = MenuItem::with_id(
        app,
        TRAY_MENU_OPEN_VANGUARD_ID,
        "打开 Vanguard",
        true,
        None::<&str>,
    )?;
    let open_quick_access = MenuItem::with_id(
        app,
        TRAY_MENU_OPEN_QUICK_ACCESS_ID,
        "打开快速访问",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let lock = MenuItem::with_id(app, TRAY_MENU_LOCK_ID, "锁定", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, TRAY_MENU_SETTINGS_ID, "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, TRAY_MENU_QUIT_ID, "退出", true, None::<&str>)?;

    let tray_menu = Menu::with_items(
        app,
        &[
            &open_vanguard,
            &open_quick_access,
            &separator,
            &lock,
            &settings,
            &quit,
        ],
    )?;

    let mut tray_builder = TrayIconBuilder::with_id(TRAY_ICON_ID)
        .menu(&tray_menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app_handle, event: MenuEvent| {
            handle_tray_menu_event(app_handle, &event);
        });

    if let Some(default_icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(default_icon);
    }

    let _ = tray_builder.build(app)?;
    Ok(())
}

fn handle_tray_menu_event<R: Runtime>(app_handle: &tauri::AppHandle<R>, event: &MenuEvent) {
    match event.id().as_ref() {
        TRAY_MENU_OPEN_VANGUARD_ID => open_main_window_from_tray(app_handle),
        TRAY_MENU_QUIT_ID => quit_app_from_tray(app_handle),
        _ => {
            log::info!(
                target: "vanguard::tray",
                "tray menu clicked (not implemented): {:?}",
                event.id()
            );
        }
    }
}

fn quit_app_from_tray<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    log::info!(target: "vanguard::tray", "exiting app from tray menu");
    app_handle.exit(0);
}

fn open_main_window_from_tray<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
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

fn ensure_spotlight_window<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<WebviewWindow<R>> {
    if let Some(window) = app.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
        return Ok(window);
    }

    let spotlight_window = build_spotlight_window(app)?;

    Ok(spotlight_window)
}

fn bind_main_window_close_to_tray<R: Runtime>(app: &tauri::App<R>) {
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

fn build_spotlight_window<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> tauri::Result<WebviewWindow<R>> {
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
