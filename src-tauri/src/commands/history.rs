use crate::models::history::HistoryEntry;
use crate::services::storage::Storage;
use tauri::State;

#[tauri::command]
pub async fn add_to_history(
    entry: HistoryEntry,
    storage: State<'_, Storage>,
) -> Result<(), String> {
    storage
        .add_history_entry(&entry)
        .map_err(|e| format!("Failed to add history entry: {}", e))
}

#[tauri::command]
pub async fn get_history(
    page: Option<u32>,
    page_size: Option<u32>,
    storage: State<'_, Storage>,
) -> Result<Vec<HistoryEntry>, String> {
    // If pagination parameters are provided, use them; otherwise get all
    match (page, page_size) {
        (Some(p), Some(ps)) => storage
            .get_history(p, ps)
            .map_err(|e| format!("Failed to get history: {}", e)),
        _ => storage
            .get_all_history()
            .map_err(|e| format!("Failed to get history: {}", e)),
    }
}

#[tauri::command]
pub async fn delete_history_entry(
    entry_id: String,
    storage: State<'_, Storage>,
) -> Result<(), String> {
    storage
        .delete_history_entry(&entry_id)
        .map_err(|e| format!("Failed to delete history entry: {}", e))
}

#[tauri::command]
pub async fn clear_history(storage: State<'_, Storage>) -> Result<(), String> {
    storage
        .clear_history()
        .map_err(|e| format!("Failed to clear history: {}", e))
}
