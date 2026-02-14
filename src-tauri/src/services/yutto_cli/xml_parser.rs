use crate::models::video::{Owner, VideoInfo};
use crate::services::bilibili_api::BilibiliApi;
use quick_xml::events::Event;
use quick_xml::Reader;

/// 解析 XML 元数据文件
pub async fn parse_xml(
    xml: &str,
    original_url: &str,
    series_name: &str,
    sessdata: Option<&str>,
) -> Result<VideoInfo, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut title = String::new();
    let mut thumbnail = String::new();
    let mut description = String::new();
    let mut website = String::new();
    let mut owner_name = String::new();
    let mut owner_face = String::new();
    let mut owner_profile = String::new();

    let mut current_tag = String::new();
    let mut in_actor = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = tag_name.clone();
                if tag_name == "actor" {
                    in_actor = true;
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();

                if in_actor {
                    match current_tag.as_str() {
                        "name" => owner_name = text,
                        "thumb" => owner_face = text,
                        "profile" => owner_profile = text,
                        _ => {}
                    }
                } else {
                    match current_tag.as_str() {
                        "title" => title = text,
                        "thumb" => thumbnail = text,
                        "plot" => description = text,
                        "website" => website = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "actor" {
                    in_actor = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML 解析错误: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    // 从 website URL 中提取 bvid 和 aid
    // 如果 website 为空，使用原始 URL
    let url_to_parse = if website.is_empty() {
        eprintln!(
            "[yutto_cli] website 字段为空，使用原始 URL: {}",
            original_url
        );
        original_url
    } else {
        eprintln!("[yutto_cli] website URL: {}", website);
        &website
    };

    let (bvid, aid) = extract_video_ids(url_to_parse)?;

    // 从 profile URL 中提取 uid
    let uid = extract_uid(&owner_profile);

    // 判断是否是番剧/课程（没有 bvid 且 aid 为 0）
    let is_bangumi = bvid.is_empty() && aid == 0;

    // 检查是否是收藏夹链接
    let is_favorite = original_url.contains("/favlist?fid=") || original_url.contains("/fav/");

    // 如果是收藏夹，尝试从 URL 中提取 fid 并获取用户信息
    let owner = if is_favorite {
        eprintln!("[yutto_cli] 检测到收藏夹链接，尝试获取用户信息");
        if let Some(fid) = extract_favorite_id(original_url) {
            eprintln!("[yutto_cli] 提取到收藏夹 ID: {}", fid);
            match BilibiliApi::fetch_favorite_owner_info(fid, sessdata).await {
                Ok(owner_info) => {
                    eprintln!("[yutto_cli] 成功获取收藏夹用户信息: {}", owner_info.name);
                    owner_info
                }
                Err(e) => {
                    eprintln!("[yutto_cli] 获取收藏夹用户信息失败: {}, 使用默认信息", e);
                    Owner {
                        uid,
                        name: owner_name.clone(),
                        face: owner_face.clone(),
                        sign: None,
                        level: None,
                        location: None,
                    }
                }
            }
        } else {
            eprintln!("[yutto_cli] 无法从 URL 提取收藏夹 ID");
            Owner {
                uid,
                name: owner_name.clone(),
                face: owner_face.clone(),
                sign: None,
                level: None,
                location: None,
            }
        }
    } else {
        // 如果是番剧且没有 owner_name，使用系列名称
        let final_owner_name = if is_bangumi && owner_name.is_empty() {
            eprintln!(
                "[yutto_cli] 番剧没有 owner_name，使用系列名称: {}",
                series_name
            );
            series_name.to_string()
        } else {
            owner_name
        };

        Owner {
            uid,
            name: final_owner_name,
            face: owner_face,
            sign: None,
            level: None,
            location: None,
        }
    };

    Ok(VideoInfo {
        title,
        bvid,
        aid,
        thumbnail,
        duration: 0, // XML 中没有 duration 信息
        description,
        owner,
        is_favorite,
        episodes: None,
        available_qualities: None,
        available_audio_qualities: None,
        comment_count: None, // XML 中没有评论数量信息
    })
}

/// 从 URL 中提取 BV 号或 AV 号
pub fn extract_video_ids(url: &str) -> Result<(String, i64), String> {
    // 检查是否是番剧链接
    if url.contains("/bangumi/") || url.contains("/cheese/") {
        eprintln!("[extract_video_ids] 检测到番剧/课程链接，返回空的视频 ID");
        // 番剧和课程不需要 BV/AV 号，返回空值
        return Ok((String::new(), 0));
    }

    // 从 URL 中提取 BV 号
    let re = regex::Regex::new(r"(BV[a-zA-Z0-9]+)").unwrap();
    if let Some(caps) = re.captures(url) {
        let bvid = caps[1].to_string();
        eprintln!("[extract_video_ids] 提取到 BV 号: {}", bvid);
        // 暂时使用 0 作为 aid，因为 XML 中没有提供
        return Ok((bvid, 0));
    }

    // 尝试提取 AV 号
    let av_re = regex::Regex::new(r"(?i)av(\d+)").unwrap();
    if let Some(caps) = av_re.captures(url) {
        if let Ok(aid) = caps[1].parse::<i64>() {
            eprintln!("[extract_video_ids] 提取到 AV 号: {}", aid);
            return Ok((String::new(), aid));
        }
    }

    eprintln!("[extract_video_ids] 无法从 URL 中提取视频 ID: {}", url);
    Err(format!("无法从 URL 中提取视频 ID: {}", url))
}

/// 从 profile URL 中提取 UID
pub fn extract_uid(profile_url: &str) -> i64 {
    // 从 profile URL 中提取 UID
    let re = regex::Regex::new(r"space\.bilibili\.com/(\d+)").unwrap();
    if let Some(caps) = re.captures(profile_url) {
        if let Ok(uid) = caps[1].parse::<i64>() {
            return uid;
        }
    }
    0
}

/// 从收藏夹 URL 中提取 fid
pub fn extract_favorite_id(url: &str) -> Option<i64> {
    // 尝试从 URL 中提取 fid
    // 格式1: /favlist?fid=123456
    let fid_re = regex::Regex::new(r"[?&]fid=(\d+)").unwrap();
    if let Some(caps) = fid_re.captures(url) {
        if let Ok(fid) = caps[1].parse::<i64>() {
            return Some(fid);
        }
    }

    // 格式2: /fav/123456
    let fav_re = regex::Regex::new(r"/fav/(\d+)").unwrap();
    if let Some(caps) = fav_re.captures(url) {
        if let Ok(fid) = caps[1].parse::<i64>() {
            return Some(fid);
        }
    }

    None
}
