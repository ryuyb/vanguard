pub struct VaultwardenEndpoints;

impl VaultwardenEndpoints {
    pub fn prelogin(base_url: &str) -> String {
        format!("{}/identity/accounts/prelogin", normalize_base(base_url))
    }

    pub fn token(base_url: &str) -> String {
        format!("{}/identity/connect/token", normalize_base(base_url))
    }

    pub fn send_email_login(base_url: &str) -> String {
        format!(
            "{}/api/two-factor/send-email-login",
            normalize_base(base_url)
        )
    }

    pub fn verify_email_token(base_url: &str) -> String {
        format!(
            "{}/api/accounts/verify-email-token",
            normalize_base(base_url)
        )
    }

    pub fn sync(base_url: &str) -> String {
        format!("{}/api/sync", normalize_base(base_url))
    }

    pub fn revision_date(base_url: &str) -> String {
        format!("{}/api/accounts/revision-date", normalize_base(base_url))
    }
}

fn normalize_base(base_url: &str) -> &str {
    base_url.trim_end_matches('/')
}
