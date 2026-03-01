use std::sync::Arc;

use crate::application::services::auth_service::AuthService;
use crate::application::services::realtime_sync_service::RealtimeSyncService;
use crate::application::services::sync_service::SyncService;

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService>,
    sync_service: Arc<SyncService>,
    realtime_sync_service: Arc<RealtimeSyncService>,
}

impl AppState {
    pub fn new(
        auth_service: Arc<AuthService>,
        sync_service: Arc<SyncService>,
        realtime_sync_service: Arc<RealtimeSyncService>,
    ) -> Self {
        Self {
            auth_service,
            sync_service,
            realtime_sync_service,
        }
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        Arc::clone(&self.auth_service)
    }

    pub fn sync_service(&self) -> Arc<SyncService> {
        Arc::clone(&self.sync_service)
    }

    pub fn realtime_sync_service(&self) -> Arc<RealtimeSyncService> {
        Arc::clone(&self.realtime_sync_service)
    }
}
