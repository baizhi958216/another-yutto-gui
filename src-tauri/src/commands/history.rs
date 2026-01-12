// History command stubs - to be implemented
#[tauri::command]
pub async fn add_to_history(entry: String) -> Result<String, String> {
    // TODO: Implement add to history
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn get_history(page: u32, page_size: u32) -> Result<String, String> {
    // TODO: Implement get history
    Ok("[]".to_string())
}

#[tauri::command]
pub async fn delete_history_entry(entry_id: String) -> Result<(), String> {
    // TODO: Implement delete history entry
    Ok(())
}

#[tauri::command]
pub async fn clear_history() -> Result<(), String> {
    // TODO: Implement clear history
    Ok(())
}
