use crate::models::video::{VideoInfo, Owner, QualityOption, AudioQualityOption};
use regex::Regex;
use serde_json::Value;

pub struct BilibiliApi;

impl BilibiliApi {
    /// 从B站API获取视频信息
    pub async fn fetch_video_info_from_html(url: &str, sessdata: Option<&str>) -> Result<VideoInfo, String> {
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
        Self::parse_api_response(data, sessdata).await
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
    async fn parse_api_response(data: &Value, sessdata: Option<&str>) -> Result<VideoInfo, String> {
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
        let (available_qualities, available_audio_qualities) = Self::fetch_video_qualities(&bvid, aid, cid, sessdata).await
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
    async fn fetch_video_qualities(bvid: &str, aid: i64, cid: i64, sessdata: Option<&str>) -> Result<(Vec<QualityOption>, Vec<AudioQualityOption>), String> {
        // 构建播放信息 API URL
        let api_url = format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=127&fnval=4048&fourk=1",
            bvid, cid
        );

        eprintln!("调用播放信息 API: {}", api_url);

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

        // 从 support_formats 中提取可用清晰度
        let mut qualities = Vec::new();

        if let Some(support_formats) = data.get("support_formats").and_then(|v| v.as_array()) {
            for format in support_formats {
                if let Some(quality) = format.get("quality").and_then(|q| q.as_i64()) {
                    let description = format.get("new_description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();

                    let description = if description.is_empty() {
                        Self::quality_to_description(quality as i32)
                    } else {
                        description
                    };

                    qualities.push(QualityOption {
                        quality: quality as i32,
                        description,
                    });
                }
            }
        }

        // 从 dash.audio 中提取可用音频质量
        let mut audio_qualities = Vec::new();

        if let Some(dash) = data.get("dash") {
            if let Some(audio_array) = dash.get("audio").and_then(|v| v.as_array()) {
                for audio in audio_array {
                    if let Some(quality) = audio.get("id").and_then(|q| q.as_i64()) {
                        let description = Self::audio_quality_to_description(quality as i32);

                        // 避免重复添加相同质量
                        if !audio_qualities.iter().any(|aq: &AudioQualityOption| aq.quality == quality as i32) {
                            audio_qualities.push(AudioQualityOption {
                                quality: quality as i32,
                                description,
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
        if audio_qualities.is_empty() {
            audio_qualities = Self::get_default_audio_quality_options();
        }

        eprintln!("成功获取 {} 个视频质量选项和 {} 个音频质量选项", qualities.len(), audio_qualities.len());

        Ok((qualities, audio_qualities))
    }

    /// 获取默认的视频质量选项
    fn get_default_quality_options() -> Vec<QualityOption> {
        vec![
            QualityOption { quality: 127, description: "8K 超高清".to_string() },
            QualityOption { quality: 126, description: "杜比视界".to_string() },
            QualityOption { quality: 125, description: "HDR 真彩".to_string() },
            QualityOption { quality: 120, description: "4K 超清".to_string() },
            QualityOption { quality: 116, description: "1080P 60帧".to_string() },
            QualityOption { quality: 112, description: "1080P 高码率".to_string() },
            QualityOption { quality: 80, description: "1080P 高清".to_string() },
            QualityOption { quality: 64, description: "720P 高清".to_string() },
            QualityOption { quality: 32, description: "480P 清晰".to_string() },
            QualityOption { quality: 16, description: "360P 流畅".to_string() },
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
            AudioQualityOption { quality: 30280, description: "Hi-Res无损".to_string() },
            AudioQualityOption { quality: 30232, description: "杜比全景声".to_string() },
            AudioQualityOption { quality: 30216, description: "64K".to_string() },
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
