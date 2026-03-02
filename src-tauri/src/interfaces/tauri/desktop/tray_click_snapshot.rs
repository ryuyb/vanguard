use std::sync::{Mutex, OnceLock};

use tauri::{PhysicalPosition, Size};

static LAST_TRAY_ICON_CLICK_SNAPSHOT: OnceLock<Mutex<Option<TrayClickSnapshot>>> = OnceLock::new();

#[derive(Clone, Copy, Debug)]
pub(super) struct TrayClickSnapshot {
    pub(super) position: PhysicalPosition<f64>,
    pub(super) rect_height: f64,
}

impl TrayClickSnapshot {
    pub(super) fn from_click(position: PhysicalPosition<f64>, rect_size: Size) -> Self {
        let (_, rect_height) = rect_size_to_f64(rect_size);
        Self {
            position,
            rect_height,
        }
    }
}

pub(super) struct TrayClickSnapshotStore;

impl TrayClickSnapshotStore {
    pub(super) fn record(snapshot: TrayClickSnapshot) {
        let locker = LAST_TRAY_ICON_CLICK_SNAPSHOT.get_or_init(|| Mutex::new(None));
        if let Ok(mut last_position) = locker.lock() {
            *last_position = Some(snapshot);
        }
    }

    pub(super) fn latest() -> Option<TrayClickSnapshot> {
        let locker = LAST_TRAY_ICON_CLICK_SNAPSHOT.get_or_init(|| Mutex::new(None));
        locker.lock().ok().and_then(|last_position| *last_position)
    }
}

fn rect_size_to_f64(size: Size) -> (f64, f64) {
    match size {
        Size::Logical(size) => (size.width, size.height),
        Size::Physical(size) => (size.width as f64, size.height as f64),
    }
}
