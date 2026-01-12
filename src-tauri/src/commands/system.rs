// System command stubs - to be implemented
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|f| f.to_string()))
}

#[tauri::command]
pub async fn open_file_location(path: String) -> Result<(), String> {
    // TODO: Implement open file location
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(std::path::Path::new(&path).parent().unwrap())
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_settings() -> Result<String, String> {
    // TODO: Implement get settings
    Ok("{}".to_string())
}

#[tauri::command]
pub async fn save_settings(settings: String) -> Result<(), String> {
    // TODO: Implement save settings
    Ok(())
}
