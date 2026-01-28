use crate::models::video::{QualityOption, AudioQualityOption};
use crate::utils::http_client::create_bilibili_client;
use crate::utils::quality_mapper::{
    video_quality_to_description, audio_quality_to_description,
    get_video_quality_requirements, get_audio_quality_requirements
};
use serde_json::Value;

/// 获取番剧的质量选项（通过第一个剧集的ep_id）
pub async fn fetch_bangumi_qualities(ep_id: i64, sessdata: Option<&str>, is_vip: bool) -> Result<(Vec<QualityOption>, Vec<AudioQualityOption>), String> {
    // 构建番剧播放信息 API URL
    let api_url = format!(
        "https://api.bilibili.com/pgc/player/web/playurl?ep_id={}&qn=127&fnval=4048&fourk=1",
        ep_id
    );

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("番剧播放信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("番剧播放信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取番剧播放信息响应失败: {}", e))?;

    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析番剧播放信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("番剧播放信息API返回错误: {} (code: {})", message, code));
    }

    let result = json.get("result")
        .ok_or("番剧播放信息API响应中未找到result字段")?;

    // 检查用户是否登录
    let is_logged_in = sessdata.is_some() && !sessdata.unwrap().is_empty();

    // 从 support_formats 中提取视频支持的所有清晰度
    let mut qualities = Vec::new();
    if let Some(support_formats) = result.get("support_formats").and_then(|v| v.as_array()) {
        for format in support_formats {
            if let Some(quality) = format.get("quality").and_then(|q| q.as_i64()) {
                let quality_code = quality as i32;
                let description = format.get("new_description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();

                let description = if description.is_empty() {
                    video_quality_to_description(quality_code)
                } else {
                    description
                };

                // 判断权限要求
                let (vip_only, login_required) = get_video_quality_requirements(quality_code);

                // 判断用户是否可以访问这个质量
                let available = if vip_only {
                    is_vip
                } else if login_required {
                    is_logged_in
                } else {
                    true
                };

                qualities.push(QualityOption {
                    quality: quality_code,
                    description,
                    available,
                    vip_only,
                    login_required,
                });
            }
        }
    }

    // 从 dash.audio 中提取音频质量
    let mut audio_qualities = Vec::new();
    if let Some(dash) = result.get("dash") {
        if let Some(audio_array) = dash.get("audio").and_then(|v| v.as_array()) {
            let mut seen_qualities = std::collections::HashSet::new();
            for audio in audio_array {
                if let Some(quality) = audio.get("id").and_then(|q| q.as_i64()) {
                    let quality_code = quality as i32;
                    if seen_qualities.insert(quality_code) {
                        let description = audio_quality_to_description(quality_code);
                        let (vip_only, login_required) = get_audio_quality_requirements(quality_code);

                        let available = if vip_only {
                            is_vip
                        } else if login_required {
                            is_logged_in
                        } else {
                            true
                        };

                        audio_qualities.push(AudioQualityOption {
                            quality: quality_code,
                            description,
                            available,
                            vip_only,
                            login_required,
                        });
                    }
                }
            }
        }
    }

    if qualities.is_empty() {
        return Err("未找到可用的视频质量选项".to_string());
    }

    // 如果没有找到音频质量，使用默认值
    let audio_qualities = if audio_qualities.is_empty() {
        get_default_audio_quality_options()
    } else {
        audio_qualities
    };

    Ok((qualities, audio_qualities))
}

/// 获取视频的 cid（通过 BV 号或 AV 号）
pub async fn fetch_video_cid(bvid: &str, aid: i64, sessdata: Option<&str>) -> Result<i64, String> {
    // 构建 API URL
    let api_url = if !bvid.is_empty() {
        format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid)
    } else if aid > 0 {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", aid)
    } else {
        return Err("无效的视频 ID".to_string());
    };

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("视频信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("视频信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取视频信息响应失败: {}", e))?;

    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析视频信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("视频信息API返回错误: {} (code: {})", message, code));
    }

    let data = json.get("data")
        .ok_or("视频信息API响应中未找到data字段")?;

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
        .ok_or("未找到 cid")?;

    Ok(cid)
}

/// 获取视频的质量选项（通过 BV 号或 AV 号）
pub async fn fetch_video_qualities(bvid: &str, aid: i64, cid: i64, sessdata: Option<&str>, is_vip: bool) -> Result<(Vec<QualityOption>, Vec<AudioQualityOption>), String> {
    // 构建播放信息 API URL
    let api_url = if !bvid.is_empty() {
        format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=127&fnval=4048&fourk=1",
            bvid, cid
        )
    } else if aid > 0 {
        format!(
            "https://api.bilibili.com/x/player/playurl?aid={}&cid={}&qn=127&fnval=4048&fourk=1",
            aid, cid
        )
    } else {
        return Err("无效的视频 ID".to_string());
    };

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("播放信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("播放信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取播放信息响应失败: {}", e))?;

    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析播放信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("播放信息API返回错误: {} (code: {})", message, code));
    }

    let data = json.get("data")
        .ok_or("播放信息API响应中未找到data字段")?;

    // 检查用户是否登录
    let is_logged_in = sessdata.is_some() && !sessdata.unwrap().is_empty();

    // 从 support_formats 中提取视频支持的所有清晰度
    let mut qualities = Vec::new();
    if let Some(support_formats) = data.get("support_formats").and_then(|v| v.as_array()) {
        for format in support_formats {
            if let Some(quality) = format.get("quality").and_then(|q| q.as_i64()) {
                let quality_code = quality as i32;
                let description = format.get("new_description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();

                let description = if description.is_empty() {
                    video_quality_to_description(quality_code)
                } else {
                    description
                };

                // 判断权限要求
                let (vip_only, login_required) = get_video_quality_requirements(quality_code);

                // 判断用户是否可以访问这个质量
                let available = if vip_only {
                    is_vip
                } else if login_required {
                    is_logged_in
                } else {
                    true
                };

                qualities.push(QualityOption {
                    quality: quality_code,
                    description,
                    available,
                    vip_only,
                    login_required,
                });
            }
        }
    }

    // 从 dash.audio 中提取音频质量
    let mut audio_qualities = Vec::new();
    if let Some(dash) = data.get("dash") {
        if let Some(audio_array) = dash.get("audio").and_then(|v| v.as_array()) {
            let mut seen_qualities = std::collections::HashSet::new();
            for audio in audio_array {
                if let Some(quality) = audio.get("id").and_then(|q| q.as_i64()) {
                    let quality_code = quality as i32;
                    if seen_qualities.insert(quality_code) {
                        let description = audio_quality_to_description(quality_code);
                        let (vip_only, login_required) = get_audio_quality_requirements(quality_code);

                        let available = if vip_only {
                            is_vip
                        } else if login_required {
                            is_logged_in
                        } else {
                            true
                        };

                        audio_qualities.push(AudioQualityOption {
                            quality: quality_code,
                            description,
                            available,
                            vip_only,
                            login_required,
                        });
                    }
                }
            }
        }
    }

    if qualities.is_empty() {
        return Err("未找到可用的视频质量选项".to_string());
    }

    // 如果没有找到音频质量，使用默认值
    let audio_qualities = if audio_qualities.is_empty() {
        get_default_audio_quality_options()
    } else {
        audio_qualities
    };

    Ok((qualities, audio_qualities))
}

/// 从番剧 URL 中提取 ss_id 并获取第一个剧集的 ep_id
pub async fn get_first_episode_id_from_season(url: &str) -> Result<i64, String> {
    // 从 URL 中提取 ss_id
    let ss_re = regex::Regex::new(r"ss(\d+)").unwrap();
    let ss_id = if let Some(caps) = ss_re.captures(url) {
        caps[1].parse::<i64>().map_err(|e| format!("解析 ss_id 失败: {}", e))?
    } else {
        return Err("无法从 URL 中提取 ss_id".to_string());
    };

    eprintln!("[get_first_episode_id_from_season] 从 URL 提取到 ss_id: {}", ss_id);

    // 调用番剧信息 API
    let api_url = format!("https://api.bilibili.com/pgc/view/web/season?season_id={}", ss_id);

    let client = create_bilibili_client()?;

    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("番剧信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("番剧信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取番剧信息响应失败: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析番剧信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("番剧信息API返回错误: {} (code: {})", message, code));
    }

    let result = json.get("result")
        .ok_or("番剧信息API响应中未找到result字段")?;

    // 获取第一个剧集的 ep_id
    let episodes = result.get("episodes")
        .and_then(|v| v.as_array())
        .ok_or("未找到剧集列表")?;

    if episodes.is_empty() {
        return Err("剧集列表为空".to_string());
    }

    let first_ep_id = episodes[0].get("id")
        .and_then(|v| v.as_i64())
        .ok_or("无法获取第一个剧集的 ep_id")?;

    Ok(first_ep_id)
}

/// 获取默认的音频质量选项
fn get_default_audio_quality_options() -> Vec<AudioQualityOption> {
    vec![
        AudioQualityOption { quality: 30251, description: "Hi-Res无损".to_string(), available: false, vip_only: true, login_required: false },
        AudioQualityOption { quality: 30255, description: "杜比音效".to_string(), available: false, vip_only: true, login_required: false },
        AudioQualityOption { quality: 30250, description: "杜比全景声".to_string(), available: false, vip_only: true, login_required: false },
        AudioQualityOption { quality: 30280, description: "320kbps".to_string(), available: false, vip_only: false, login_required: true },
        AudioQualityOption { quality: 30232, description: "132kbps".to_string(), available: false, vip_only: false, login_required: true },
        AudioQualityOption { quality: 30216, description: "64kbps".to_string(), available: false, vip_only: false, login_required: true },
    ]
}
