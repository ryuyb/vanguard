use std::cmp::Ordering;

use tauri::{Monitor, PhysicalPosition, Runtime};

use crate::interfaces::tauri::desktop::tray_click_snapshot::TrayClickSnapshot;

pub(super) fn find_monitor_from_logical_point<R: Runtime>(
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
            .unwrap_or(Ordering::Equal)
    });
    matches.into_iter().next()
}

pub(super) fn find_monitor_from_cursor_flexible<R: Runtime>(
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
    scale_factors.sort_by(|left, right| right.partial_cmp(left).unwrap_or(Ordering::Equal));

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

        matches.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(Ordering::Equal));
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

    mixed_matches.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(Ordering::Equal));
    mixed_matches.into_iter().next().map(|entry| entry.0)
}

pub(super) fn find_monitor_from_tray_click_snapshot<R: Runtime>(
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

    candidates.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(Ordering::Equal));
    candidates.into_iter().next().map(|entry| entry.0)
}

pub(super) fn monitor_logical_work_area(monitor: &Monitor) -> (f64, f64, f64, f64) {
    let work_area = monitor.work_area();
    let scale = monitor.scale_factor();
    let left = work_area.position.x as f64;
    let top = work_area.position.y as f64;
    let width = work_area.size.width as f64 / scale;
    let height = work_area.size.height as f64 / scale;
    (left, top, width, height)
}
