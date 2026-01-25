// Download models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadConfig {
    pub url: String,
    pub video_quality: i32,
    pub audio_quality: i32,
    pub download_path: String,
    pub yutto_cli_path: Option<String>,
    pub with_danmaku: bool,
    pub with_subtitle: bool,
    pub with_cover: bool,
    pub with_comments: bool,
    pub batch: bool,
    pub video_only: Option<bool>,
    pub audio_only: Option<bool>,
    pub episodes: Option<String>,
    pub sessdata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub title: String,
    pub thumbnail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub config: DownloadConfig,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: String,
    pub eta: String,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub video_info: Option<VideoInfo>,
    pub total_size: i64, // Total file size in bytes
    pub saved_file_path: Option<String>, // Actual saved file path parsed from yutto output
    pub comment_file_path: Option<String>, // Comment file path if comments were downloaded
    pub comment_download_progress: Option<String>, // Comment download progress like "121/450"
    pub is_downloading_comments: bool, // Whether currently downloading comments
    pub start_time: i64, // Download start timestamp in milliseconds
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub progress: f64,
    pub speed: String,
    pub eta: String,
}
