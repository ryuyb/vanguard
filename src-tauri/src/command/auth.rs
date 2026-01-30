use tauri::{AppHandle, State};

use crate::service::auth;
use crate::state::AppState;

#[tauri::command]
pub async fn login_and_sync(
    app: AppHandle,
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<String, String> {
    auth::login(&app, state.inner(), email, password).await?;
    Ok("ok".into())
}
