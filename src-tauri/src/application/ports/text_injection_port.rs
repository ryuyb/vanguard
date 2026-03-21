use crate::support::result::AppResult;

/// Port for injecting text into the focused input field
pub trait TextInjectionPort: Send + Sync {
    /// Type text into the previously focused input field
    ///
    /// # Arguments
    /// * `text` - The text to type
    ///
    /// # Returns
    /// Ok(()) if successful, Err otherwise
    fn type_text(&self, text: &str) -> AppResult<()>;

    /// Check if text injection is available on this platform
    fn is_available(&self) -> bool;
}
