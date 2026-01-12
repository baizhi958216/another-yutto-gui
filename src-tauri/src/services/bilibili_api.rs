use crate::models::video::{VideoInfo, Owner, QualityOption};
use regex::Regex;
use serde_json::Value;

pub struct BilibiliApi;

impl BilibiliApi {
    /// 从B站页面HTML中提取videoData JSON
    pub async fn fetch_video_info_from_html(url: &str, sessdata: Option<&str>) -> Result<VideoInfo, String> {
        // 构建HTTP客户端
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        // 构建请求
        let mut request = client.get(url);

        // 如果提供了SESSDATA，添加Cookie
        if let Some(sessdata) = sessdata {
            request = request.header("Cookie", format!("SESSDATA={}", sessdata));
        }

        // 发送请求
        let response = request
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP请求失败: {}", response.status()));
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("读取响应失败: {}", e))?;

        // 从HTML中提取__INITIAL_STATE__或__playinfo__数据
        Self::parse_video_data_from_html(&html, url)
    }

    /// 从HTML中解析视频数据
    fn parse_video_data_from_html(html: &str, url: &str) -> Result<VideoInfo, String> {
        // 尝试提取 window.__INITIAL_STATE__
        let initial_state_re = Regex::new(r#"window\.__INITIAL_STATE__\s*=\s*(\{.*?\});?\s*\(function"#).unwrap();

        let video_data_json = if let Some(caps) = initial_state_re.captures(html) {
            caps[1].to_string()
        } else {
            // 尝试另一种模式
            let alt_re = Regex::new(r#"window\.__INITIAL_STATE__\s*=\s*(\{.*?\});\s*</script>"#).unwrap();
            if let Some(caps) = alt_re.captures(html) {
                caps[1].to_string()
            } else {
                return Err("无法从HTML中提取视频数据".to_string());
            }
        };

        // 解析JSON
        let data: Value = serde_json::from_str(&video_data_json)
            .map_err(|e| format!("解析JSON失败: {}", e))?;

        // 提取视频信息
        let video_data = data.get("videoData")
            .or_else(|| data.get("videoInfo"))
            .ok_or("未找到videoData字段")?;

        let title = video_data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let bvid = video_data.get("bvid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let aid = video_data.get("aid")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let thumbnail = video_data.get("pic")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let duration = video_data.get("duration")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let description = video_data.get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 提取UP主信息
        let owner_data = video_data.get("owner")
            .ok_or("未找到owner字段")?;

        let owner = Owner {
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
        };

        // 提取可用的视频质量选项
        let available_qualities = Self::extract_quality_options(&data);

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
        })
    }

    /// 从数据中提取可用的视频质量选项
    fn extract_quality_options(data: &Value) -> Vec<QualityOption> {
        let mut qualities = Vec::new();

        // 尝试从 playinfo 中提取
        if let Some(playinfo) = data.get("playInfo")
            .or_else(|| data.get("videoData").and_then(|v| v.get("dash"))) {

            if let Some(dash) = playinfo.get("data").and_then(|d| d.get("dash"))
                .or_else(|| playinfo.get("dash")) {

                if let Some(video_list) = dash.get("video").and_then(|v| v.as_array()) {
                    for video in video_list {
                        if let Some(quality) = video.get("id").and_then(|q| q.as_i64()) {
                            let description = Self::quality_to_description(quality as i32);
                            qualities.push(QualityOption {
                                quality: quality as i32,
                                description,
                            });
                        }
                    }
                }
            }
        }

        // 如果没有找到质量信息，返回默认选项
        if qualities.is_empty() {
            qualities = vec![
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
            ];
        } else {
            // 去重并排序
            qualities.sort_by(|a, b| b.quality.cmp(&a.quality));
            qualities.dedup_by(|a, b| a.quality == b.quality);
        }

        qualities
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
            74 => "720P 60帧".to_string(),
            64 => "720P 高清".to_string(),
            32 => "480P 清晰".to_string(),
            16 => "360P 流畅".to_string(),
            _ => format!("质量 {}", quality),
        }
    }
}
