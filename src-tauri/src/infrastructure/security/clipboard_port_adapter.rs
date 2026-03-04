use tauri::{AppHandle, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::application::ports::clipboard_port::ClipboardPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

#[derive(Clone)]
pub struct TauriClipboardPortAdapter<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> TauriClipboardPortAdapter<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<R: Runtime> ClipboardPort for TauriClipboardPortAdapter<R> {
    fn write_text(&self, text: &str) -> AppResult<()> {
        self.app_handle
            .clipboard()
            .write_text(text)
            .map_err(|error| AppError::internal(format!("failed to write clipboard text: {error}")))
    }

    fn clear(&self) -> AppResult<()> {
        self.app_handle
            .clipboard()
            .clear()
            .map_err(|error| AppError::internal(format!("failed to clear clipboard: {error}")))
    }
}
