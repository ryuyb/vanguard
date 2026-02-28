pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod support;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            greet,
            interfaces::tauri::commands::auth::auth_prelogin,
            interfaces::tauri::commands::auth::auth_login_with_password,
            interfaces::tauri::commands::auth::auth_refresh_token,
            interfaces::tauri::commands::auth::auth_send_email_login,
            interfaces::tauri::commands::auth::auth_verify_email_token
        ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("failed to export specta bindings");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = bootstrap::wiring::build_app_state(app).map_err(
                |error| -> Box<dyn std::error::Error> {
                    log::error!("failed to wire application state: {error}");
                    Box::new(error)
                },
            )?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
