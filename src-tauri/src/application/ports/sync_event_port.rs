use crate::domain::sync::SyncContext;

pub trait SyncEventPort: Send + Sync {
    fn emit_sync_started(&self, account_id: &str);

    fn emit_sync_succeeded(&self, context: &SyncContext);

    fn emit_sync_failed(&self, account_id: &str, code: &str, message: &str);

    fn emit_auth_required(&self, account_id: &str, status: u16, message: &str);

    fn emit_logged_out(&self, account_id: &str, reason: &str);
}
