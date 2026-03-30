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

    pub fn cipher(base_url: &str, cipher_id: &str) -> String {
        format!("{}/api/ciphers/{}", normalize_base(base_url), cipher_id)
    }

    pub fn cipher_soft_delete(base_url: &str, cipher_id: &str) -> String {
        format!(
            "{}/api/ciphers/{}/delete",
            normalize_base(base_url),
            cipher_id
        )
    }

    pub fn folder(base_url: &str, folder_id: &str) -> String {
        format!("{}/api/folders/{}", normalize_base(base_url), folder_id)
    }

    pub fn folders(base_url: &str) -> String {
        format!("{}/api/folders", normalize_base(base_url))
    }

    pub fn send(base_url: &str, send_id: &str) -> String {
        format!("{}/api/sends/{}", normalize_base(base_url), send_id)
    }

    pub fn send_remove_password(base_url: &str, send_id: &str) -> String {
        format!(
            "{}/api/sends/{}/remove-password",
            normalize_base(base_url),
            send_id
        )
    }

    pub fn sends(base_url: &str) -> String {
        format!("{}/api/sends", normalize_base(base_url))
    }

    pub fn sends_file_v2(base_url: &str) -> String {
        format!("{}/api/sends/file/v2", normalize_base(base_url))
    }

    pub fn send_file_upload(base_url: &str, send_id: &str, file_id: &str) -> String {
        format!(
            "{}/api/sends/{}/file/{}",
            normalize_base(base_url),
            send_id,
            file_id
        )
    }

    pub fn ciphers(base_url: &str) -> String {
        format!("{}/api/ciphers", normalize_base(base_url))
    }

    pub fn cipher_by_id(base_url: &str, cipher_id: &str) -> String {
        format!("{}/api/ciphers/{}", normalize_base(base_url), cipher_id)
    }

    pub fn register(base_url: &str) -> String {
        format!(
            "{}/identity/accounts/register/send-verification-email",
            normalize_base(base_url)
        )
    }

    pub fn register_finish(base_url: &str) -> String {
        format!(
            "{}/identity/accounts/register/finish",
            normalize_base(base_url)
        )
    }
}

fn normalize_base(base_url: &str) -> &str {
    base_url.trim_end_matches('/')
}
