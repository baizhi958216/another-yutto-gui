use crate::services::yutto_cli::YuttoCli;
use crate::services::bilibili_api::BilibiliApi;
use crate::models::video::VideoInfo;

#[tauri::command]
pub async fn fetch_video_info(url: String, sessdata: Option<String>, is_vip: bool) -> Result<VideoInfo, String> {
    eprintln!("[fetch_video_info] Received parameters - url: {}, sessdata present: {}, is_vip: {}",
        url, sessdata.is_some(), is_vip);

    // 首先尝试从B站HTML页面获取详细信息（包括可用清晰度）
    match BilibiliApi::fetch_video_info_from_html(&url, sessdata.as_deref(), is_vip).await {
        Ok(info) => Ok(info),
        Err(e) => {
            // 如果HTML解析失败，回退到使用yutto CLI获取基本信息
            eprintln!("从HTML获取视频信息失败: {}, 回退到yutto CLI", e);
            YuttoCli::fetch_video_info(&url, sessdata.as_deref()).await
        }
    }
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
