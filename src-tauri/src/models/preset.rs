// Preset models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: PresetConfig,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub video_quality: i32,
    pub audio_quality: i32,
    pub with_danmaku: bool,
    pub with_subtitle: bool,
    pub with_cover: bool,
}
