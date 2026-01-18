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

#[tauri::command]
pub async fn get_file_size(path: String) -> Result<i64, String> {
    use std::fs;
    use std::path::Path;

    fn calculate_dir_size(path: &Path) -> Result<i64, String> {
        let mut total_size = 0i64;

        // Check if path exists
        if !path.exists() {
            return Err(format!("路径不存在: {}", path.display()));
        }

        if path.is_file() {
            let metadata = fs::metadata(path).map_err(|e| format!("无法获取文件信息: {}", e))?;
            return Ok(metadata.len() as i64);
        }

        if path.is_dir() {
            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(e) => {
                    // If we can't read the directory (permission denied, etc.), return 0 instead of error
                    eprintln!("警告: 无法读取目录 {}: {}", path.display(), e);
                    return Ok(0);
                }
            };

            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();

                    // Skip if we can't access the entry
                    if let Ok(metadata) = fs::metadata(&entry_path) {
                        if metadata.is_file() {
                            total_size += metadata.len() as i64;
                        } else if metadata.is_dir() {
                            // Recursively calculate subdirectory size
                            // Ignore errors from subdirectories
                            if let Ok(subdir_size) = calculate_dir_size(&entry_path) {
                                total_size += subdir_size;
                            }
                        }
                    }
                }
            }
        }

        Ok(total_size)
    }

    let path_obj = Path::new(&path);
    calculate_dir_size(path_obj)
}

#[tauri::command]
pub async fn find_newest_file_in_dir(
    dir_path: String,
    after_timestamp: Option<i64>,
    extensions: Option<Vec<String>>,
    name_hint: Option<String>,
) -> Result<Option<String>, String> {
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;
    use std::time::SystemTime;

    let dir = Path::new(&dir_path);

    if !dir.exists() {
        return Err(format!("目录不存在: {}", dir_path));
    }

    if !dir.is_dir() {
        return Err(format!("路径不是目录: {}", dir_path));
    }

    let mut newest_file: Option<(String, SystemTime)> = None;
    let threshold_time = after_timestamp.map(|ts| {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(ts as u64)
    });
    let allowed_exts = extensions.map(|exts| {
        exts.into_iter()
            .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
            .collect::<HashSet<String>>()
    });
    let normalized_hint = name_hint
        .map(|hint| normalize_name(&hint))
        .filter(|value| !value.is_empty());

    fn normalize_name(value: &str) -> String {
        value
            .chars()
            .filter(|ch| ch.is_alphanumeric())
            .flat_map(|ch| ch.to_lowercase())
            .collect()
    }

    fn extension_allowed(path: &Path, allowed_exts: &Option<HashSet<String>>) -> bool {
        match allowed_exts {
            None => true,
            Some(allowed) => {
                let ext = path.extension().and_then(|s| s.to_str());
                ext.map(|value| allowed.contains(&value.to_ascii_lowercase())).unwrap_or(false)
            }
        }
    }

    fn name_matches(path: &Path, name_hint: &Option<String>) -> bool {
        match name_hint {
            None => true,
            Some(hint) => {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let normalized_file_name = normalize_name(file_name);
                normalized_file_name.contains(hint)
            }
        }
    }

    fn scan_dir(
        dir: &Path,
        newest: &mut Option<(String, SystemTime)>,
        threshold: Option<SystemTime>,
        allowed_exts: &Option<HashSet<String>>,
        name_hint: &Option<String>,
    ) -> Result<(), String> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("警告: 无法读取目录 {}: {}", dir.display(), e);
                return Ok(());
            }
        };

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_file() {
                        if !extension_allowed(&path, allowed_exts) {
                            continue;
                        }
                        if !name_matches(&path, name_hint) {
                            continue;
                        }
                        if let Ok(modified) = metadata.modified() {
                            // Check if file was modified after threshold
                            if let Some(threshold_time) = threshold {
                                if modified <= threshold_time {
                                    continue;
                                }
                            }

                            // Update newest file if this one is newer
                            match newest {
                                None => {
                                    *newest = Some((path.to_string_lossy().to_string(), modified));
                                }
                                Some((_, ref current_time)) => {
                                    if modified > *current_time {
                                        *newest = Some((path.to_string_lossy().to_string(), modified));
                                    }
                                }
                            }
                        }
                    } else if metadata.is_dir() {
                        // Recursively scan subdirectories
                        let _ = scan_dir(&path, newest, threshold, allowed_exts, name_hint);
                    }
                }
            }
        }

        Ok(())
    }

    scan_dir(dir, &mut newest_file, threshold_time, &allowed_exts, &normalized_hint)?;

    Ok(newest_file.map(|(path, _)| path))
}
