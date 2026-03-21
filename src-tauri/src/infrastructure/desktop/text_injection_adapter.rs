use enigo::{Enigo, Keyboard, Settings};

use crate::application::ports::text_injection_port::TextInjectionPort;
use crate::support::error::AppError;
use crate::support::result::AppResult;

/// Adapter for text injection using the enigo crate
///
/// Note: Enigo is not Sync, so we don't store it in the struct.
/// Instead, we create a new instance for each operation.
#[derive(Default)]
pub struct EnigoTextInjectionAdapter {
    has_permission: bool,
}

impl EnigoTextInjectionAdapter {
    pub fn new() -> AppResult<Self> {
        // Try to create Enigo instance to check permission
        let settings = Settings::default();
        let has_permission = match Enigo::new(&settings) {
            Ok(_) => {
                log::info!(
                    target: "vanguard::text_injection",
                    "Enigo initialized successfully, text injection is available"
                );
                true
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::text_injection",
                    "Enigo initialization failed (likely missing Accessibility permission): {}",
                    error
                );
                false
            }
        };

        Ok(Self { has_permission })
    }
}

impl TextInjectionPort for EnigoTextInjectionAdapter {
    fn type_text(&self, text: &str) -> AppResult<()> {
        // Check if we have permission first
        if !self.has_permission {
            return Err(AppError::InternalUnexpected {
                message: "Text injection not available: missing Accessibility permission"
                    .to_string(),
            });
        }

        // Create a new Enigo instance for each typing operation
        // This avoids issues with Sync/Send
        let settings = Settings::default();
        let mut enigo = Enigo::new(&settings).map_err(|e| AppError::InternalUnexpected {
            message: format!("Failed to create Enigo instance: {}", e),
        })?;

        enigo.text(text).map_err(|e| AppError::InternalUnexpected {
            message: format!("Failed to type text: {}", e),
        })?;

        Ok(())
    }

    fn is_available(&self) -> bool {
        // Only available on macOS and when we have permission
        cfg!(target_os = "macos") && self.has_permission
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = EnigoTextInjectionAdapter::new();
        // Should always succeed now, even without permission
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_is_available_without_permission() {
        let adapter = EnigoTextInjectionAdapter::default();
        // Should not be available without permission
        assert!(!adapter.is_available());
    }
}
