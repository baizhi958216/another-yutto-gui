mod commands;
mod services;
mod models;

use commands::*;
use services::download_manager::DownloadManager;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize download manager
    let download_manager = Arc::new(DownloadManager::new(3));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(download_manager)
        .invoke_handler(tauri::generate_handler![
            // Video commands
            video::fetch_video_info,
            video::validate_url,
            // Download commands
            download::start_download,
            download::pause_download,
            download::resume_download,
            download::cancel_download,
            download::get_active_downloads,
            // Auth commands
            auth::save_sessdata,
            auth::get_sessdata,
            auth::validate_sessdata,
            auth::clear_auth,
            // Preset commands
            preset::save_preset,
            preset::get_presets,
            preset::delete_preset,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
