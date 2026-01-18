use crate::models::download::{DownloadConfig, VideoInfo};
use crate::services::download_manager::DownloadManager;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn start_download(
    config: DownloadConfig,
    video_info: Option<VideoInfo>,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<String, String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    manager.start_download(task_id.clone(), config, video_info).await?;
    Ok(task_id)
}

#[tauri::command]
pub async fn pause_download(
    download_id: String,
    _manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // TODO: Implement download pause
    Err("暂停功能暂未实现".to_string())
}

#[tauri::command]
pub async fn resume_download(
    download_id: String,
    _manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // TODO: Implement download resume
    Err("恢复功能暂未实现".to_string())
}

#[tauri::command]
pub async fn cancel_download(
    download_id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.cancel_download(&download_id).await
}

#[tauri::command]
pub async fn get_active_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<String, String> {
    let downloads = manager.active_downloads.lock().await;
    let tasks: Vec<_> = downloads.values().cloned().collect();
    serde_json::to_string(&tasks).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_max_concurrent_downloads(
    max_concurrent: usize,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.set_max_concurrent(max_concurrent);
    manager.start_pending().await;
    Ok(())
}
