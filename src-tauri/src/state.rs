use bitwarden::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub bw_client: Arc<Mutex<Option<Client>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bw_client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_bw_client(&self, client: Client) {
        let mut bw_client = self.bw_client.lock().await;
        *bw_client = Some(client);
    }
}
