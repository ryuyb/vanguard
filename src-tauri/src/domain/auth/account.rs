#[derive(Debug, Clone)]
pub struct AccountServer {
    pub account_id: String,
    pub base_url: String,
}

impl AccountServer {
    pub fn new(account_id: impl Into<String>, base_url: impl Into<String>) -> Option<Self> {
        let account_id = account_id.into();
        let base_url = base_url.into();

        if account_id.trim().is_empty() || base_url.trim().is_empty() {
            return None;
        }

        Some(Self {
            account_id,
            base_url,
        })
    }
}
