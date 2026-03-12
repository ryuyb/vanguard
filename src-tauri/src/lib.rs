#[cfg(debug_assertions)]
use std::path::{Path, PathBuf};

pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod support;

#[cfg(debug_assertions)]
use specta_typescript::BigIntExportBehavior;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            interfaces::tauri::commands::auth::auth_login_with_password,
            interfaces::tauri::commands::auth::auth_send_email_login,
            interfaces::tauri::commands::auth::auth_verify_email_token,
            interfaces::tauri::commands::auth::auth_restore_state,
            interfaces::tauri::commands::auth::auth_logout,
            interfaces::tauri::commands::desktop::desktop_open_main_window,
            interfaces::tauri::commands::folder::list_folders,
            interfaces::tauri::commands::folder::create_folder,
            interfaces::tauri::commands::folder::rename_folder,
            interfaces::tauri::commands::folder::delete_folder,
            interfaces::tauri::commands::sync::vault_sync_now,
            interfaces::tauri::commands::sync::vault_sync_status,
            interfaces::tauri::commands::sync::vault_sync_check_revision,
            interfaces::tauri::commands::vault::vault_can_unlock,
            interfaces::tauri::commands::vault::vault_is_unlocked,
            interfaces::tauri::commands::vault::vault_get_biometric_status,
            interfaces::tauri::commands::vault::vault_get_pin_status,
            interfaces::tauri::commands::vault::vault_can_unlock_with_biometric,
            interfaces::tauri::commands::vault::vault_unlock,
            interfaces::tauri::commands::vault::vault_enable_pin_unlock,
            interfaces::tauri::commands::vault::vault_disable_pin_unlock,
            interfaces::tauri::commands::vault::vault_enable_biometric_unlock,
            interfaces::tauri::commands::vault::vault_disable_biometric_unlock,
            interfaces::tauri::commands::vault::vault_lock,
            interfaces::tauri::commands::vault::vault_get_view_data,
            interfaces::tauri::commands::vault::vault_list_ciphers,
            interfaces::tauri::commands::vault::vault_get_cipher_detail,
            interfaces::tauri::commands::vault::vault_copy_cipher_field,
            interfaces::tauri::commands::vault::vault_get_cipher_totp_code,
            interfaces::tauri::commands::vault::vault_get_icon_server,
            interfaces::tauri::commands::vault::create_cipher,
            interfaces::tauri::commands::vault::update_cipher,
            interfaces::tauri::commands::vault::delete_cipher,
            interfaces::tauri::commands::vault::soft_delete_cipher
        ])
        .events(tauri_specta::collect_events![
            interfaces::tauri::events::sync::VaultSyncStarted,
            interfaces::tauri::events::sync::VaultSyncSucceeded,
            interfaces::tauri::events::sync::VaultSyncFailed,
            interfaces::tauri::events::sync::VaultSyncAuthRequired,
            interfaces::tauri::events::sync::VaultSyncLoggedOut,
            interfaces::tauri::events::sync::VaultFoldersSynced,
            interfaces::tauri::events::cipher::CipherCreated,
            interfaces::tauri::events::cipher::CipherUpdated,
            interfaces::tauri::events::cipher::CipherDeleted
        ]);

    let invoke_handler = specta_builder.invoke_handler();

    #[cfg(debug_assertions)]
    export_specta_bindings(&specta_builder).expect("failed to export specta bindings");

    let mut app_builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            interfaces::tauri::desktop::open_main_window(app);
        }));
    }

    app_builder
        .setup(move |app| {
            specta_builder.mount_events(app);

            #[cfg(target_os = "macos")]
            let _ = app.manage(tauri_nspanel::WebviewPanelManager::<tauri::Wry>::default());

            #[cfg(desktop)]
            {
                bootstrap::desktop::install_desktop_features(app)?;
            }

            match app.path().app_data_dir() {
                Ok(path) => {
                    log::info!(
                        target: "vanguard::bootstrap",
                        "app_data_dir={}",
                        path.display()
                    );
                }
                Err(error) => {
                    log::warn!(
                        target: "vanguard::bootstrap",
                        "failed to resolve app_data_dir: {}",
                        error
                    );
                }
            }
            let app_state = bootstrap::wiring::build_app_state(app).map_err(
                |error| -> Box<dyn std::error::Error> {
                    log::error!("failed to wire application state: {error}");
                    Box::new(error)
                },
            )?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(debug_assertions)]
fn export_specta_bindings(builder: &tauri_specta::Builder<tauri::Wry>) -> std::io::Result<()> {
    let output = builder
        .export_str(
            specta_typescript::Typescript::default()
                .bigint(BigIntExportBehavior::Number)
                .header("// @ts-nocheck"),
        )
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let target_path = bindings_output_path();
    let temp_path = temporary_bindings_path(&target_path);
    std::fs::write(&temp_path, output)?;
    std::fs::rename(&temp_path, &target_path)?;
    Ok(())
}

#[cfg(debug_assertions)]
fn bindings_output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts")
}

#[cfg(debug_assertions)]
fn temporary_bindings_path(target_path: &Path) -> PathBuf {
    let mut path = target_path.to_path_buf();
    path.set_extension("ts.tmp");
    path
}
