// Video models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub bvid: String,
    pub aid: i64,
    pub thumbnail: String,
    pub duration: i64,
    pub description: String,
    pub owner: Owner,
    pub is_favorite: bool,
    pub episodes: Option<Vec<Episode>>,
    pub available_qualities: Option<Vec<QualityOption>>,
    pub available_audio_qualities: Option<Vec<AudioQualityOption>>,
    pub comment_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOption {
    pub quality: i32,
    pub description: String,
    pub available: bool,
    pub vip_only: bool,
    pub login_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityOption {
    pub quality: i32,
    pub description: String,
    pub available: bool,
    pub vip_only: bool,
    pub login_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub uid: i64,
    pub name: String,
    pub face: String,
    pub sign: Option<String>,
    pub level: Option<i32>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub index: i64,
    pub available_qualities: Option<Vec<QualityOption>>,
    pub available_audio_qualities: Option<Vec<AudioQualityOption>>,
}
