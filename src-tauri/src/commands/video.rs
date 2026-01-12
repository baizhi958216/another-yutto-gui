use crate::services::yutto_cli::YuttoCli;
use crate::models::video::VideoInfo;

#[tauri::command]
pub async fn fetch_video_info(url: String, sessdata: Option<String>) -> Result<VideoInfo, String> {
    YuttoCli::fetch_video_info(&url, sessdata.as_deref()).await
}

#[tauri::command]
pub fn validate_url(url: String) -> Result<bool, String> {
    // 验证是否为有效的 B 站链接
    let valid_patterns = [
        "bilibili.com/video/",
        "bilibili.com/bangumi/",
        "b23.tv/",
        "BV",
    ];

    Ok(valid_patterns.iter().any(|pattern| url.contains(pattern)))
}
