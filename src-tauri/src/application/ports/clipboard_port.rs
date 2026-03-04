use crate::support::result::AppResult;

pub trait ClipboardPort: Send + Sync {
    fn write_text(&self, text: &str) -> AppResult<()>;
    fn clear(&self) -> AppResult<()>;
}
