use std::sync::{Mutex, OnceLock};

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::window::Color;
use tauri::{
    ActivationPolicy, LogicalPosition, Manager, Monitor, PhysicalPosition, Runtime, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};
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
static LAST_TRAY_ICON_CLICK_SNAPSHOT: OnceLock<Mutex<Option<TrayClickSnapshot>>> = OnceLock::new();

#[derive(Clone, Copy, Debug)]
struct TrayClickSnapshot {
    position: PhysicalPosition<f64>,
    rect_height: f64,
}

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
        .on_tray_icon_event(move |_tray, event: TrayIconEvent| {
            if let TrayIconEvent::Click { position, rect, .. } = event {
                let (_, rect_height) = rect_size_to_f64(rect.size);
                record_last_tray_icon_click_snapshot(TrayClickSnapshot {
                    position,
                    rect_height,
                });
            }
        })
        .on_menu_event(move |app_handle, event: MenuEvent| {
            handle_tray_menu_event(app_handle, &event);
        });

    if let Some(default_icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(default_icon);
    }

    let _ = tray_builder.build(app)?;
    Ok(())
}

fn rect_size_to_f64(size: tauri::Size) -> (f64, f64) {
    match size {
        tauri::Size::Logical(size) => (size.width, size.height),
        tauri::Size::Physical(size) => (size.width as f64, size.height as f64),
    }
}

fn record_last_tray_icon_click_snapshot(snapshot: TrayClickSnapshot) {
    let locker = LAST_TRAY_ICON_CLICK_SNAPSHOT.get_or_init(|| Mutex::new(None));
    if let Ok(mut last_position) = locker.lock() {
        *last_position = Some(snapshot);
    }
}

fn last_tray_icon_click_snapshot() -> Option<TrayClickSnapshot> {
    let locker = LAST_TRAY_ICON_CLICK_SNAPSHOT.get_or_init(|| Mutex::new(None));
    locker.lock().ok().and_then(|last_position| *last_position)
}

fn handle_tray_menu_event<R: Runtime>(app_handle: &tauri::AppHandle<R>, event: &MenuEvent) {
    match event.id().as_ref() {
        TRAY_MENU_OPEN_VANGUARD_ID => open_main_window_from_tray(app_handle),
        TRAY_MENU_QUIT_ID => quit_app_from_tray(app_handle),
        _ => {}
    }
}

fn quit_app_from_tray<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
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

    move_main_window_to_active_monitor_center(app_handle, &main_window);

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

fn move_main_window_to_active_monitor_center<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    main_window: &WebviewWindow<R>,
) {
    let tray_click_snapshot = last_tray_icon_click_snapshot();
    let cursor_position = app_handle.cursor_position().ok();

    let target_monitor = if let Some(snapshot) = tray_click_snapshot {
        find_monitor_from_tray_click_snapshot(app_handle, snapshot).or_else(|| {
            app_handle
                .monitor_from_point(snapshot.position.x, snapshot.position.y)
                .ok()
                .flatten()
        })
    } else {
        None
    }
    .or_else(|| {
        cursor_position.and_then(|position| find_monitor_from_logical_point(app_handle, position))
    })
    .or_else(|| main_window.current_monitor().ok().flatten())
    .or_else(|| app_handle.primary_monitor().ok().flatten());

    let Some(target_monitor) = target_monitor else {
        log::warn!(
            target: "vanguard::tray",
            "no monitor available, skip repositioning main window"
        );
        return;
    };

    let window_size = match main_window.outer_size() {
        Ok(size) => size,
        Err(error) => {
            log::warn!(
                target: "vanguard::tray",
                "failed to read main window size for reposition: {error}"
            );
            return;
        }
    };
    let window_scale = main_window
        .scale_factor()
        .ok()
        .unwrap_or(target_monitor.scale_factor());
    let window_width_logical = window_size.width as f64 / window_scale;
    let window_height_logical = window_size.height as f64 / window_scale;

    let (work_area_left, work_area_top, work_area_width, work_area_height) =
        monitor_logical_work_area(&target_monitor);
    let max_left = (work_area_left + work_area_width - window_width_logical).max(work_area_left);
    let max_top = (work_area_top + work_area_height - window_height_logical).max(work_area_top);
    let target_x = work_area_left + (work_area_width - window_width_logical) / 2.0;
    let target_y = work_area_top + (work_area_height - window_height_logical) / 2.0;
    let bounded_x = target_x.clamp(work_area_left, max_left);
    let bounded_y = target_y.clamp(work_area_top, max_top);

    if let Err(error) = main_window.set_position(LogicalPosition::new(bounded_x, bounded_y)) {
        log::warn!(
            target: "vanguard::tray",
            "failed to reposition main window to active monitor: {error}"
        );
    }
}

fn find_monitor_from_logical_point<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    point: PhysicalPosition<f64>,
) -> Option<Monitor> {
    let monitors = app_handle.available_monitors().ok()?;
    let mut matches: Vec<Monitor> = monitors
        .into_iter()
        .filter(|monitor| {
            let (left, top, width, height) = monitor_logical_work_area(monitor);
            let right = left + width;
            let bottom = top + height;
            point.x >= left && point.x < right && point.y >= top && point.y < bottom
        })
        .collect();

    if matches.is_empty() {
        return None;
    }
    if matches.len() == 1 {
        return matches.pop();
    }

    // If multiple monitors still overlap after normalization, prefer the smaller logical area.
    matches.sort_by(|left_monitor, right_monitor| {
        let (_, _, left_width, left_height) = monitor_logical_work_area(left_monitor);
        let (_, _, right_width, right_height) = monitor_logical_work_area(right_monitor);
        let left_area = left_width * left_height;
        let right_area = right_width * right_height;
        left_area
            .partial_cmp(&right_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.into_iter().next()
}

fn find_monitor_from_cursor_flexible<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    cursor: PhysicalPosition<f64>,
) -> Option<Monitor> {
    let monitors = app_handle.available_monitors().ok()?;
    let mut scale_factors = vec![1.0_f64];
    for monitor in &monitors {
        let scale = monitor.scale_factor();
        if !scale_factors
            .iter()
            .any(|current| (current - scale).abs() < 0.001)
        {
            scale_factors.push(scale);
        }
    }
    scale_factors
        .sort_by(|left, right| right.partial_cmp(left).unwrap_or(std::cmp::Ordering::Equal));

    for factor in scale_factors {
        if factor <= 0.0 {
            continue;
        }
        let normalized_cursor = PhysicalPosition::new(cursor.x / factor, cursor.y / factor);
        let mut matches: Vec<(Monitor, f64)> = monitors
            .iter()
            .filter_map(|monitor| {
                let position = monitor.position();
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let logical_width = size.width as f64 / scale;
                let logical_height = size.height as f64 / scale;
                let contains = normalized_cursor.x >= position.x as f64
                    && normalized_cursor.x < position.x as f64 + logical_width
                    && normalized_cursor.y >= position.y as f64
                    && normalized_cursor.y < position.y as f64 + logical_height;
                if !contains {
                    return None;
                }

                let center_x = position.x as f64 + logical_width / 2.0;
                let center_y = position.y as f64 + logical_height / 2.0;
                let distance = (normalized_cursor.x - center_x).powi(2)
                    + (normalized_cursor.y - center_y).powi(2);
                Some((monitor.clone(), distance))
            })
            .collect();

        if matches.is_empty() {
            continue;
        }

        matches.sort_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some((monitor, _)) = matches.into_iter().next() {
            return Some(monitor);
        }
    }

    let mut mixed_matches: Vec<(Monitor, f64)> = monitors
        .iter()
        .filter_map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            let contains = cursor.x >= position.x as f64
                && cursor.x < position.x as f64 + size.width as f64
                && cursor.y >= position.y as f64
                && cursor.y < position.y as f64 + size.height as f64;
            if !contains {
                return None;
            }

            let center_x = position.x as f64 + size.width as f64 / 2.0;
            let center_y = position.y as f64 + size.height as f64 / 2.0;
            let distance = (cursor.x - center_x).powi(2) + (cursor.y - center_y).powi(2);
            Some((monitor.clone(), distance))
        })
        .collect();

    mixed_matches.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    mixed_matches.into_iter().next().map(|entry| entry.0)
}

fn find_monitor_from_tray_click_snapshot<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    snapshot: TrayClickSnapshot,
) -> Option<Monitor> {
    let monitors = app_handle.available_monitors().ok()?;
    let mut candidates: Vec<(Monitor, f64)> = Vec::new();

    for monitor in monitors {
        let scale = monitor.scale_factor();
        let logical_x = snapshot.position.x / scale;
        let logical_y = snapshot.position.y / scale;
        let (left, top, width, height) = monitor_logical_work_area(&monitor);
        let right = left + width;
        let bottom = top + height;
        let contains =
            logical_x >= left && logical_x < right && logical_y >= top && logical_y < bottom;

        if !contains {
            continue;
        }

        let inferred_icon_height = snapshot.rect_height / scale;
        let icon_height_score = (inferred_icon_height - 30.0).abs();
        candidates.push((monitor, icon_height_score));
    }

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.into_iter().next().map(|entry| entry.0)
}

fn monitor_logical_work_area(monitor: &Monitor) -> (f64, f64, f64, f64) {
    let work_area = monitor.work_area();
    let scale = monitor.scale_factor();
    let left = work_area.position.x as f64;
    let top = work_area.position.y as f64;
    let width = work_area.size.width as f64 / scale;
    let height = work_area.size.height as f64 / scale;
    (left, top, width, height)
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
                                place_spotlight_window(app, &spotlight_window);
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
            place_spotlight_window(app, &spotlight_window);
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

    place_spotlight_window(app, &spotlight_window);
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

fn place_spotlight_window<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    spotlight_window: &WebviewWindow<R>,
) {
    let cursor_position = app_handle.cursor_position().ok();
    let target_monitor = if let Some(cursor) = cursor_position {
        if let Some(monitor) = find_monitor_from_cursor_flexible(app_handle, cursor) {
            Some(monitor)
        } else if let Some(monitor) = app_handle
            .monitor_from_point(cursor.x, cursor.y)
            .ok()
            .flatten()
        {
            Some(monitor)
        } else if let Some(monitor) = find_monitor_from_logical_point(app_handle, cursor) {
            Some(monitor)
        } else {
            None
        }
    } else {
        None
    }
    .or_else(|| spotlight_window.current_monitor().ok().flatten())
    .or_else(|| app_handle.primary_monitor().ok().flatten());

    if let Some(target_monitor) = target_monitor {
        let (work_area_left, work_area_top, work_area_width, _) =
            monitor_logical_work_area(&target_monitor);
        let target_x = work_area_left + (work_area_width - SPOTLIGHT_WIDTH) / 2.0;

        if let Err(error) =
            spotlight_window.set_position(LogicalPosition::new(target_x, work_area_top))
        {
            log::warn!(
                target: "vanguard::spotlight",
                "failed to place spotlight window by active monitor: {error}"
            );
        }
        return;
    }

    if let Err(error) = spotlight_window
        .as_ref()
        .window()
        .move_window(Position::TopCenter)
    {
        log::warn!(
            target: "vanguard::spotlight",
            "failed to set spotlight window position with positioner fallback: {error}"
        );
    }
}
