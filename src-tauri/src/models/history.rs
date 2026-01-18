// History models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub download_date: i64,
    pub file_path: String,
    pub video_quality: i32,
    pub audio_quality: i32,
    pub video_only: Option<bool>,
    pub audio_only: Option<bool>,
    pub size: i64,
}
