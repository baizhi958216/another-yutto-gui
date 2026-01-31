mod commands;
mod services;
mod models;
mod utils;
mod error;

use commands::*;
use services::download_manager::DownloadManager;
use services::storage::Storage;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize storage in setup hook where we have access to app handle
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
            let db_path = app_data_dir.join("yutto.db");
            let storage = Storage::new(db_path.to_string_lossy().to_string())
                .expect("Failed to initialize storage");
            storage.init_db().expect("Failed to initialize database");

            // Initialize download manager with app handle
            let download_manager = Arc::new(DownloadManager::new(3, app.handle().clone()));

            app.manage(storage);
            app.manage(download_manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Video commands
            video::fetch_video_info,
            video::validate_url,
            video::fetch_video_comments,
            video::download_video_comments,
            video::download_comments_to_file,
            // Download commands
            download::start_download,
            download::pause_download,
            download::resume_download,
            download::cancel_download,
            download::get_active_downloads,
            download::set_max_concurrent_downloads,
            // Auth commands
            auth::save_sessdata,
            auth::get_sessdata,
            auth::validate_sessdata,
            auth::clear_auth,
            auth::open_login_window,
            auth::check_vip_status,
            // History commands
            history::add_to_history,
            history::get_history,
            history::delete_history_entry,
            history::clear_history,
            // System commands
            system::select_directory,
            system::open_file_location,
            system::get_settings,
            system::save_settings,
            system::get_file_size,
            system::find_newest_file_in_dir,
            system::read_csv_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
