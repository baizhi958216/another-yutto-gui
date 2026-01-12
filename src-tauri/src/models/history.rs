// History models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub download_date: i64,
    pub file_path: String,
    pub quality: String,
    pub size: i64,
}
