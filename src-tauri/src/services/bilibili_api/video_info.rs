use crate::models::video::{VideoInfo, Owner};
use crate::utils::http_client::create_bilibili_client;
use regex::Regex;
use serde_json::Value;

/// 从B站API获取视频信息
pub async fn fetch_video_info_from_html(url: &str, sessdata: Option<&str>, is_vip: bool) -> Result<VideoInfo, String> {
    let url_lower = url.to_lowercase();

    // 检查是否是番剧/课程链接
    let is_bangumi = url_lower.contains("/bangumi/")
        || url_lower.contains("/cheese/")
        || url_lower.starts_with("ep")
        || url_lower.starts_with("ss")
        || url_lower.starts_with("md");

    // 检查是否应该跳过 API 提取，直接使用 yutto CLI
    // 根据 yutto 文档，以下类型只能用 yutto CLI：
    // 1. 番剧链接（/bangumi/）
    // 2. 课程链接（/cheese/）
    // 3. 收藏夹、空间、列表等批量链接
    // 4. 短链接（需要重定向）
    if is_bangumi
        || url_lower.contains("space.bilibili.com/")
        || url_lower.contains("watchlater")
        || url_lower.contains("/list/")
        || url_lower.contains("b23.tv/")
    {
        eprintln!("[fetch_video_info_from_html] 检测到非普通视频链接，跳过 API 提取，直接使用 yutto CLI");
        return Err("该链接类型不支持 API 提取，需要使用 yutto CLI".to_string());
    }

    // 从 URL 中提取 BV 号或 AV 号（仅用于普通投稿视频）
    let (bvid, aid) = extract_video_id(url)?;

    // 调用 B站 API
    let api_url = if let Some(bv) = bvid {
        format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bv)
    } else if let Some(av) = aid {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", av)
    } else {
        return Err("无法从URL中提取视频ID".to_string());
    };

    eprintln!("调用 B站 API: {}", api_url);

    // 构建 HTTP 客户端
    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    // 如果提供了 SESSDATA，添加 Cookie
    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    // 发送请求
    let response = request
        .send()
        .await
        .map_err(|e| format!("API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    // 解析 JSON
    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;

    // 检查返回码
    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("API返回错误: {} (code: {})", message, code));
    }

    // 提取 data 字段
    let data = json.get("data")
        .ok_or("API响应中未找到data字段")?;

    // 解析视频信息
    parse_api_response(data, sessdata, is_vip).await
}

/// 从 URL 中提取 BV 号或 AV 号
pub fn extract_video_id(url: &str) -> Result<(Option<String>, Option<i64>), String> {
    eprintln!("[extract_video_id] 尝试从 URL 提取视频 ID: {}", url);

    // 提取 BV 号
    let bv_re = Regex::new(r"(BV[a-zA-Z0-9]+)").unwrap();
    if let Some(caps) = bv_re.captures(url) {
        let bvid = caps[1].to_string();
        eprintln!("[extract_video_id] 提取到 BV 号: {}", bvid);
        return Ok((Some(bvid), None));
    }

    // 提取 AV 号（支持大小写）
    let av_re = Regex::new(r"(?i)av(\d+)").unwrap();
    if let Some(caps) = av_re.captures(url) {
        if let Ok(aid) = caps[1].parse::<i64>() {
            eprintln!("[extract_video_id] 提取到 AV 号: {}", aid);
            return Ok((None, Some(aid)));
        }
    }

    eprintln!("[extract_video_id] 无法从 URL 中提取 BV 号或 AV 号");
    Err(format!("无法从URL中提取BV号或AV号。输入的URL: {}", url))
}

/// 解析 API 响应数据
async fn parse_api_response(data: &Value, sessdata: Option<&str>, is_vip: bool) -> Result<VideoInfo, String> {
    let title = data.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let bvid = data.get("bvid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let aid = data.get("aid")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let thumbnail = data.get("pic")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let duration = data.get("duration")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let description = data.get("desc")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // 提取 UP 主信息
    let owner = if let Some(owner_data) = data.get("owner") {
        Owner {
            uid: owner_data.get("mid")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            name: owner_data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            face: owner_data.get("face")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sign: None,
            level: None,
            location: None,
        }
    } else {
        Owner {
            uid: 0,
            name: "未知".to_string(),
            face: "".to_string(),
            sign: None,
            level: None,
            location: None,
        }
    };

    // 获取第一个分P的cid
    let cid = data.get("cid")
        .and_then(|v| v.as_i64())
        .or_else(|| {
            data.get("pages")
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|page| page.get("cid"))
                .and_then(|v| v.as_i64())
        })
        .unwrap_or(0);

    // 获取视频质量选项和音频质量选项
    let (available_qualities, available_audio_qualities) = super::quality::fetch_video_qualities(&bvid, aid, cid, sessdata, is_vip).await
        .unwrap_or_else(|e| {
            eprintln!("获取视频质量失败: {}, 使用默认选项", e);
            (super::quality::get_default_quality_options(), super::quality::get_default_audio_quality_options())
        });

    // 获取评论数量
    let comment_count = data.get("stat")
        .and_then(|stat| stat.get("reply"))
        .and_then(|v| v.as_i64());

    eprintln!("成功解析视频信息: {}", title);

    Ok(VideoInfo {
        title,
        bvid,
        aid,
        thumbnail,
        duration,
        description,
        owner,
        is_favorite: false,
        episodes: None,
        available_qualities: Some(available_qualities),
        available_audio_qualities: Some(available_audio_qualities),
        comment_count,
    })
}
