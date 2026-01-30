use bitwarden::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub(crate) bw_client: Arc<Mutex<Option<Client>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bw_client: Arc::new(Mutex::new(None)),
        }
    }
}
