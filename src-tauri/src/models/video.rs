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
    pub episodes: Option<Vec<Episode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub uid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub index: i64,
}
