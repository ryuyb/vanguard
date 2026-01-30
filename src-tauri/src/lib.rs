mod command;
mod database;
mod error;
mod model;
mod service;
mod state;
mod util;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![command::auth::login_and_sync])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
