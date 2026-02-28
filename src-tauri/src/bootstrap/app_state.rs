use std::sync::Arc;

use crate::application::services::auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
}

impl AppState {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        Arc::clone(&self.auth_service)
    }
}
