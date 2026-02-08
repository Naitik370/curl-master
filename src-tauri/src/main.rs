// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod http;
mod db;
mod models;
mod commands;
mod sync;



fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database on app startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::init_db(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_request,
            commands::cancel_request,
            commands::get_settings,
            commands::update_setting,
            commands::create_collection,
            commands::create_folder,
            commands::save_request,
            commands::get_collections_with_folders,
            commands::ensure_workspace,
            commands::create_environment,
            commands::get_environments,
            commands::set_active_env,
            commands::get_variables,
            commands::save_variables,
            commands::delete_workspace,
            commands::delete_collection,
            commands::delete_folder,
            commands::delete_request,
            commands::get_workspaces,
            commands::create_workspace,
            commands::rename_workspace,
            commands::get_history,
            commands::clear_history,
            commands::delete_history_entry,
            commands::get_active_variables,
            commands::import_collection,
            commands::update_request,
            commands::clear_all_data,
            commands::get_sync_config,
            commands::set_sync_config,
            commands::sync_login,
            commands::sync_register,
            commands::sync_logout,
            commands::pull_workspace,
            commands::invite_to_workspace,
            commands::clear_sync_error,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
