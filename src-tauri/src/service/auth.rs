use bitwarden::auth::login::PasswordLoginRequest;
use bitwarden::{Client, ClientSettings, DeviceType};
use tauri::AppHandle;

use crate::state::AppState;
use crate::util::tauri_store::get_app_store_value;

const SERVER_HOST_KEY: &str = "serverHost";
const SELF_HOSTED_VALUE: &str = "self-hosted";

pub async fn login(
    app: &AppHandle,
    state: &AppState,
    email: String,
    password: String,
) -> Result<(), String> {
    let server_url = read_server_url(app)?;
    let settings = ClientSettings {
        identity_url: format!("{}/identity", server_url),
        api_url: format!("{}/api", server_url),
        user_agent: "Vanguard".into(),
        device_type: DeviceType::MacOsDesktop,
    };

    let client = Client::new(Some(settings));

    let kdf = client
        .auth()
        .prelogin(email.clone())
        .await
        .map_err(|err| err.to_string())?;

    client
        .auth()
        .login_password(&PasswordLoginRequest {
            email,
            password,
            two_factor: None,
            kdf,
        })
        .await
        .map_err(|err| err.to_string())?;

    state.set_bw_client(client).await;

    Ok(())
}

fn read_server_url(app: &AppHandle) -> Result<String, String> {
    let host = read_server_host(app)?
        .ok_or_else(|| "Missing serverHost in app store.".to_string())?;

    if host == SELF_HOSTED_VALUE {
        return read_self_hosted_server_url(app)?
            .ok_or_else(|| "Missing self-hosted server URL in app store.".to_string());
    }

    Ok(host)
}

fn read_self_hosted_server_url(app: &AppHandle) -> Result<Option<String>, String> {
    let value = get_app_store_value(app, "selfHosted").map_err(|err| err.to_string())?;
    let Some(value) = value else {
        return Ok(None);
    };

    let url = value
        .get("serverUrl")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Ok(url)
}

fn read_server_host(app: &AppHandle) -> Result<Option<String>, String> {
    let value = get_app_store_value(app, SERVER_HOST_KEY).map_err(|err| err.to_string())?;
    let Some(value) = value else {
        return Ok(None);
    };

    let host = value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Ok(host)
}
