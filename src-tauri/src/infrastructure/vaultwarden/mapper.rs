use super::models::{TokenResponse, VaultSession};

pub fn to_session(token: TokenResponse) -> VaultSession {
    VaultSession {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
    }
}
