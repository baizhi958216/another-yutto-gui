use crate::services::yutto_cli::YuttoCli;
use crate::services::bilibili_api::BilibiliApi;
use crate::models::video::VideoInfo;
use crate::models::comment::Comment;

#[tauri::command]
pub async fn fetch_video_info(
    app_handle: tauri::AppHandle,
    url: String,
    sessdata: Option<String>,
    is_vip: bool
) -> Result<VideoInfo, String> {
    eprintln!("[fetch_video_info] Received parameters - url: {}, sessdata present: {}, is_vip: {}",
        url, sessdata.is_some(), is_vip);

    // 首先尝试从B站HTML页面获取详细信息（包括可用清晰度）
    match BilibiliApi::fetch_video_info_from_html(&url, sessdata.as_deref(), is_vip).await {
        Ok(info) => Ok(info),
        Err(e) => {
            // 如果HTML解析失败，回退到使用yutto CLI获取基本信息
            eprintln!("从HTML获取视频信息失败: {}, 回退到yutto CLI", e);
            YuttoCli::fetch_video_info(&app_handle, &url, sessdata.as_deref(), is_vip).await
        }
    }
}

#[tauri::command]
pub fn validate_url(url: String) -> Result<bool, String> {
    // 验证是否为有效的 B 站链接或视频 ID
    let url_lower = url.to_lowercase();
    let valid_patterns = [
        // 投稿视频
        "bilibili.com/video/",
        "bv",
        "av",
        // 番剧
        "bilibili.com/bangumi/",
        "ep",
        "ss",
        "md",
        // 课程
        "bilibili.com/cheese/",
        // 短链接
        "b23.tv/",
        // 收藏夹和空间
        "space.bilibili.com/",
        // 稍后再看
        "watchlater",
        // 列表
        "bilibili.com/list/",
    ];

    Ok(valid_patterns.iter().any(|pattern| url_lower.contains(pattern)))
}

#[tauri::command]
pub async fn fetch_video_comments(aid: i64, pagination_str: String, sessdata: Option<String>) -> Result<(Vec<Comment>, Option<String>, bool), String> {
    eprintln!("[fetch_video_comments] Received parameters - aid: {}, pagination_str: {}, sessdata present: {}",
        aid, pagination_str, sessdata.is_some());

    BilibiliApi::fetch_comments(aid, &pagination_str, sessdata.as_deref()).await
}

#[tauri::command]
pub async fn download_video_comments(
    aid: i64,
    bvid: String,
    save_path: String,
    download_avatars: bool,
    delay_seconds: u64,
    sessdata: Option<String>,
) -> Result<String, String> {
    eprintln!("[download_video_comments] Starting download - aid: {}, bvid: {}, save_path: {}, download_avatars: {}, delay: {}s",
        aid, bvid, save_path, download_avatars, delay_seconds);

    BilibiliApi::download_all_comments(aid, &bvid, &save_path, download_avatars, delay_seconds, sessdata.as_deref()).await
}

#[tauri::command]
pub async fn download_comments_to_file(
    aid: i64,
    csv_file_path: String,
    delay_seconds: u64,
    sessdata: Option<String>,
) -> Result<String, String> {
    eprintln!("[download_comments_to_file] Starting download - aid: {}, csv_file_path: {}, delay: {}s",
        aid, csv_file_path, delay_seconds);

    BilibiliApi::download_comments_to_file(aid, &csv_file_path, delay_seconds, sessdata.as_deref()).await
}
