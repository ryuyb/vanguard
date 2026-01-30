mod command;
mod database;
mod error;
mod model;
mod service;
mod state;

use bitwarden::auth::login::PasswordLoginRequest;
use bitwarden::{Client, ClientSettings, DeviceType};
use tauri::State;

use crate::state::AppState;

// Learn more about Tauri command at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn login_and_sync(
    state: State<'_, AppState>,
    server_url: String,
    email: String,
    password: String,
) -> Result<String, String> {
    let settings = ClientSettings {
        identity_url: format!("{}/identity", server_url),
        api_url: format!("{}/api", server_url),
        user_agent: "Vanguard".into(),
        device_type: DeviceType::MacOsDesktop,
    };

    let mut client = Client::new(Some(settings));

    let kdf = client
        .auth()
        .prelogin(email.clone())
        .await
        .expect("Pre-login failed");
    let login_response = client
        .auth()
        .login_password(&PasswordLoginRequest {
            email,
            password,
            two_factor: None,
            kdf,
        })
        .await
        .expect("Login failed");
    println!("login_response: {:?}", login_response);

    Ok("123".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![login_and_sync])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
