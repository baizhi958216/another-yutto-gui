use crate::models::video::{VideoInfo, Owner, QualityOption, AudioQualityOption};
use regex::Regex;
use serde_json::Value;

pub struct BilibiliApi;

impl BilibiliApi {
    /// 从B站API获取视频信息
    pub async fn fetch_video_info_from_html(url: &str, sessdata: Option<&str>, is_vip: bool) -> Result<VideoInfo, String> {
        // 从 URL 中提取 BV 号或 AV 号
        let (bvid, aid) = Self::extract_video_id(url)?;

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
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

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
        Self::parse_api_response(data, sessdata, is_vip).await
    }

    /// 从 URL 中提取 BV 号或 AV 号
    fn extract_video_id(url: &str) -> Result<(Option<String>, Option<i64>), String> {
        // 提取 BV 号
        let bv_re = Regex::new(r"(BV[a-zA-Z0-9]+)").unwrap();
        if let Some(caps) = bv_re.captures(url) {
            return Ok((Some(caps[1].to_string()), None));
        }

        // 提取 AV 号
        let av_re = Regex::new(r"av(\d+)").unwrap();
        if let Some(caps) = av_re.captures(url) {
            if let Ok(aid) = caps[1].parse::<i64>() {
                return Ok((None, Some(aid)));
            }
        }

        Err("无法从URL中提取BV号或AV号".to_string())
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
            }
        } else {
            Owner {
                uid: 0,
                name: "未知".to_string(),
                face: "".to_string(),
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
        let (available_qualities, available_audio_qualities) = Self::fetch_video_qualities(&bvid, aid, cid, sessdata, is_vip).await
            .unwrap_or_else(|e| {
                eprintln!("获取视频质量失败: {}, 使用默认选项", e);
                (Self::get_default_quality_options(), Self::get_default_audio_quality_options())
            });

        eprintln!("成功解析视频信息: {}", title);

        Ok(VideoInfo {
            title,
            bvid,
            aid,
            thumbnail,
            duration,
            description,
            owner,
            episodes: None,
            available_qualities: Some(available_qualities),
            available_audio_qualities: Some(available_audio_qualities),
        })
    }

    /// 获取视频的可用清晰度选项和音频质量选项
    async fn fetch_video_qualities(bvid: &str, aid: i64, cid: i64, sessdata: Option<&str>, is_vip: bool) -> Result<(Vec<QualityOption>, Vec<AudioQualityOption>), String> {
        // 构建播放信息 API URL
        let api_url = format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=127&fnval=4048&fourk=1",
            bvid, cid
        );

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

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
                        Self::quality_to_description(quality_code)
                    } else {
                        description
                    };

                    // 判断权限要求
                    let (vip_only, login_required) = Self::get_quality_requirements(quality_code);

                    // 判断用户是否可以访问这个质量
                    let available = if vip_only {
                        // 需要大会员的质量，检查用户是否是VIP
                        is_vip
                    } else if login_required {
                        // 需要登录的质量，检查是否已登录
                        is_logged_in
                    } else {
                        // 不需要登录的质量（360p及以下），总是可用
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

        // 从 dash.audio 中提取视频支持的音频质量
        let mut audio_qualities = Vec::new();
        if let Some(dash) = data.get("dash") {
            if let Some(audio_array) = dash.get("audio").and_then(|v| v.as_array()) {
                // 使用HashSet去重
                let mut seen_qualities = std::collections::HashSet::new();
                for audio in audio_array {
                    if let Some(quality) = audio.get("id").and_then(|q| q.as_i64()) {
                        let quality_code = quality as i32;
                        if seen_qualities.insert(quality_code) {
                            let description = Self::audio_quality_to_description(quality_code);
                            let (vip_only, login_required) = Self::get_audio_quality_requirements(quality_code);

                            // 音频质量的可用性判断逻辑与视频相同
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
            Self::get_default_audio_quality_options()
        } else {
            audio_qualities
        };

        Ok((qualities, audio_qualities))
    }

    /// 判断视频质量的权限要求
    fn get_quality_requirements(quality: i32) -> (bool, bool) {
        match quality {
            // 大会员专享质量
            127 | 126 | 125 | 120 | 116 | 112 | 100 => (true, false),
            // 需要登录的质量（480p-1080P）
            80 | 74 | 64 | 32 => (false, true),
            // 360p及以下无需登录
            16 => (false, false),
            _ => (false, true), // 默认需要登录
        }
    }

    /// 判断音频质量的权限要求
    fn get_audio_quality_requirements(quality: i32) -> (bool, bool) {
        match quality {
            // 大会员专享音质
            30251 | 30255 | 30250 => (true, false),
            // 普通音质需要登录
            30280 | 30232 => (false, true),
            // 基础音质无需登录
            30216 => (false, false),
            _ => (false, true), // 默认需要登录
        }
    }

    /// 获取默认的视频质量选项
    fn get_default_quality_options() -> Vec<QualityOption> {
        vec![
            QualityOption { quality: 127, description: "8K 超高清".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 126, description: "杜比视界".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 125, description: "HDR 真彩".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 120, description: "4K 超清".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 116, description: "1080P 60帧".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 112, description: "1080P 高码率".to_string(), available: false, vip_only: true, login_required: false },
            QualityOption { quality: 80, description: "1080P 高清".to_string(), available: false, vip_only: false, login_required: true },
            QualityOption { quality: 64, description: "720P 高清".to_string(), available: false, vip_only: false, login_required: true },
            QualityOption { quality: 32, description: "480P 清晰".to_string(), available: false, vip_only: false, login_required: true },
            QualityOption { quality: 16, description: "360P 流畅".to_string(), available: false, vip_only: false, login_required: true },
        ]
    }

    /// 将质量代码转换为描述文字
    fn quality_to_description(quality: i32) -> String {
        match quality {
            127 => "8K 超高清".to_string(),
            126 => "杜比视界".to_string(),
            125 => "HDR 真彩".to_string(),
            120 => "4K 超清".to_string(),
            116 => "1080P 60帧".to_string(),
            112 => "1080P 高码率".to_string(),
            80 => "1080P 高清".to_string(),
            64 => "720P 高清".to_string(),
            32 => "480P 清晰".to_string(),
            16 => "360P 流畅".to_string(),
            _ => format!("质量 {}", quality),
        }
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

    /// 将音频质量代码转换为描述文字
    fn audio_quality_to_description(quality: i32) -> String {
        match quality {
            30280 => "Hi-Res无损".to_string(),
            30251 => "无损".to_string(),
            30250 => "杜比全景声".to_string(),
            30232 => "132K".to_string(),
            30216 => "64K".to_string(),
            _ => format!("音频质量 {}", quality),
        }
    }
}
