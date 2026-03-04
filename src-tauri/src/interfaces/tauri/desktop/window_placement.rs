use tauri::{LogicalPosition, Manager, Monitor, Runtime, WebviewWindow};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::interfaces::tauri::desktop::constants::{SPOTLIGHT_WIDTH, SPOTLIGHT_WINDOW_LABEL};
use crate::interfaces::tauri::desktop::monitor::{
    find_monitor_from_cursor_flexible, find_monitor_from_logical_point,
    find_monitor_from_tray_click_snapshot, monitor_logical_work_area,
};
use crate::interfaces::tauri::desktop::tray_click_snapshot::TrayClickSnapshotStore;

pub(super) struct WindowPlacementPolicy;

impl WindowPlacementPolicy {
    pub(super) fn recenter_main_window_on_active_monitor<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        main_window: &WebviewWindow<R>,
        prefer_tray_snapshot: bool,
    ) {
        let Some(target_monitor) =
            Self::resolve_monitor_for_main_window(app_handle, main_window, prefer_tray_snapshot)
        else {
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
        let max_left =
            (work_area_left + work_area_width - window_width_logical).max(work_area_left);
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

    pub(super) fn place_spotlight_window<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        spotlight_window: &WebviewWindow<R>,
    ) {
        if let Some(target_monitor) =
            Self::resolve_monitor_for_spotlight(app_handle, spotlight_window)
        {
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

    fn resolve_monitor_for_main_window<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        main_window: &WebviewWindow<R>,
        prefer_tray_snapshot: bool,
    ) -> Option<Monitor> {
        if prefer_tray_snapshot {
            if let Some(snapshot) = TrayClickSnapshotStore::latest() {
                if let Some(monitor) = find_monitor_from_tray_click_snapshot(app_handle, snapshot)
                    .or_else(|| {
                        app_handle
                            .monitor_from_point(snapshot.position.x, snapshot.position.y)
                            .ok()
                            .flatten()
                    })
                {
                    return Some(monitor);
                }
            }
        }

        if !prefer_tray_snapshot {
            if let Some(spotlight_window) = app_handle.get_webview_window(SPOTLIGHT_WINDOW_LABEL) {
                if let Some(monitor) = spotlight_window.current_monitor().ok().flatten() {
                    return Some(monitor);
                }
            }
        }

        if let Ok(cursor_position) = app_handle.cursor_position() {
            if let Some(monitor) = find_monitor_from_cursor_flexible(app_handle, cursor_position) {
                return Some(monitor);
            }
            if let Some(monitor) = app_handle
                .monitor_from_point(cursor_position.x, cursor_position.y)
                .ok()
                .flatten()
            {
                return Some(monitor);
            }
            if let Some(monitor) = find_monitor_from_logical_point(app_handle, cursor_position) {
                return Some(monitor);
            }
        }

        main_window
            .current_monitor()
            .ok()
            .flatten()
            .or_else(|| app_handle.primary_monitor().ok().flatten())
    }

    fn resolve_monitor_for_spotlight<R: Runtime>(
        app_handle: &tauri::AppHandle<R>,
        spotlight_window: &WebviewWindow<R>,
    ) -> Option<Monitor> {
        if let Ok(cursor_position) = app_handle.cursor_position() {
            if let Some(monitor) = find_monitor_from_cursor_flexible(app_handle, cursor_position) {
                return Some(monitor);
            }
            if let Some(monitor) = app_handle
                .monitor_from_point(cursor_position.x, cursor_position.y)
                .ok()
                .flatten()
            {
                return Some(monitor);
            }
            if let Some(monitor) = find_monitor_from_logical_point(app_handle, cursor_position) {
                return Some(monitor);
            }
        }

        spotlight_window
            .current_monitor()
            .ok()
            .flatten()
            .or_else(|| app_handle.primary_monitor().ok().flatten())
    }
}
