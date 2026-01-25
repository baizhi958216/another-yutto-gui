use crate::models::video::{VideoInfo, Owner, QualityOption, AudioQualityOption};
use crate::models::comment::{Comment, CommentResponse};
use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// WBI签名相关常量
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49,
    33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40,
    61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11,
    36, 20, 34, 44, 52,
];

// WBI密钥缓存
static WBI_KEYS_CACHE: Mutex<Option<(String, String, u64)>> = Mutex::new(None);

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
            episodes: None,
            available_qualities: Some(available_qualities),
            available_audio_qualities: Some(available_audio_qualities),
            comment_count,
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

    /// 获取视频评论（使用WBI签名API）
    pub async fn fetch_comments(aid: i64, pagination_str: &str, sessdata: Option<&str>) -> Result<(Vec<Comment>, Option<String>, bool), String> {
        // 构建参数
        let mut params = vec![
            ("oid".to_string(), aid.to_string()),
            ("type".to_string(), "1".to_string()),
            ("pagination_str".to_string(), pagination_str.to_string()),
            ("plat".to_string(), "1".to_string()),
            ("web_location".to_string(), "1315875".to_string()),
        ];

        // 对参数进行WBI签名
        Self::sign_wbi_params(&mut params).await?;

        // 构建URL
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let api_url = format!("https://api.bilibili.com/x/v2/reply/wbi/main?{}", query_string);

        eprintln!("调用评论 API: {}", api_url);

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
            .map_err(|e| format!("评论API请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("评论API请求失败: {}", response.status()));
        }

        let json_text = response
            .text()
            .await
            .map_err(|e| format!("读取评论响应失败: {}", e))?;

        let json: CommentResponse = serde_json::from_str(&json_text)
            .map_err(|e| format!("解析评论JSON失败: {}", e))?;

        if json.code != 0 {
            return Err(format!("评论API返回错误: {} (code: {})", json.message, json.code));
        }

        let data = json.data.ok_or("评论API响应中未找到data字段")?;

        // 获取下一页的offset
        let next_offset = data.cursor.pagination_reply
            .as_ref()
            .map(|p| p.next_offset.clone());

        let is_end = data.cursor.is_end;

        let replies = data.replies.unwrap_or_default();

        // 转换为简化的Comment结构
        let comments: Vec<Comment> = replies.iter().map(|reply| {
            Comment {
                rpid: reply.rpid,
                oid: reply.oid,
                mid: reply.mid,
                uname: reply.member.uname.clone(),
                avatar: reply.member.avatar.clone(),
                sex: reply.member.sex.clone(),
                content: reply.content.message.clone(),
                ctime: reply.ctime,
                like: reply.like,
                current_level: reply.member.level_info.current_level,
                location: reply.reply_control.location.clone().unwrap_or_default(),
                parent: reply.parent,
                pictures: reply.content.pictures.clone().unwrap_or_default(),
            }
        }).collect();

        eprintln!("成功获取 {} 条评论", comments.len());

        Ok((comments, next_offset, is_end))
    }

    /// 下载所有评论到本地文件
    pub async fn download_all_comments(
        aid: i64,
        bvid: &str,
        save_path: &str,
        download_avatars: bool,
        delay_seconds: u64,
        sessdata: Option<&str>,
    ) -> Result<String, String> {
        use std::fs::{create_dir_all, File, OpenOptions};
        use std::io::Write;
        use std::path::Path;
        use tokio::time::{sleep, Duration};

        // 创建保存目录
        let video_dir = Path::new(save_path).join(bvid);
        create_dir_all(&video_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        let csv_path = video_dir.join(format!("{}.csv", bvid));
        let avatars_dir = video_dir.join("avatars");

        if download_avatars {
            create_dir_all(&avatars_dir)
                .map_err(|e| format!("创建头像目录失败: {}", e))?;
        }

        // 检查CSV文件是否存在，如果不存在则创建并写入表头
        let file_exists = csv_path.exists();
        let mut csv_file = if file_exists {
            OpenOptions::new()
                .append(true)
                .open(&csv_path)
                .map_err(|e| format!("打开CSV文件失败: {}", e))?
        } else {
            let mut file = File::create(&csv_path)
                .map_err(|e| format!("创建CSV文件失败: {}", e))?;
            // 写入CSV表头
            writeln!(file, "bvid,upname,sex,content,avatar,rpid,oid,mid,parent,ctime,like,level,location")
                .map_err(|e| format!("写入CSV表头失败: {}", e))?;
            file
        };

        let mut total_downloaded = 0;
        let mut pagination_str = r#"{"offset":""}"#.to_string();

        eprintln!("开始下载评论，视频: {}, aid: {}", bvid, aid);

        loop {
            eprintln!("正在获取评论，pagination_str: {}", pagination_str);

            // 获取评论
            let (comments, next_offset, is_end) = Self::fetch_comments(aid, &pagination_str, sessdata).await?;

            if comments.is_empty() {
                eprintln!("没有更多评论，下载完成");
                break;
            }

            // 保存评论到CSV
            for comment in &comments {
                let content_escaped = comment.content.replace("\"", "\"\"").replace("\n", " ");
                let location_escaped = comment.location.replace("\"", "\"\"");
                let uname_escaped = comment.uname.replace("\"", "\"\"");

                let line = format!(
                    "{},\"{}\",{},\"{}\",{},{},{},{},{},{},{},{},\"{}\"\n",
                    bvid,
                    uname_escaped,
                    comment.sex,
                    content_escaped,
                    comment.avatar,
                    comment.rpid,
                    comment.oid,
                    comment.mid,
                    comment.parent,
                    comment.ctime,
                    comment.like,
                    comment.current_level,
                    location_escaped
                );

                csv_file.write_all(line.as_bytes())
                    .map_err(|e| format!("写入CSV失败: {}", e))?;

                // 下载头像
                if download_avatars && !comment.avatar.is_empty() {
                    let avatar_filename = format!("{}_{}.jpg", comment.mid, comment.uname.replace("/", "_"));
                    let avatar_path = avatars_dir.join(&avatar_filename);

                    // 如果头像文件不存在，则下载
                    if !avatar_path.exists() {
                        if let Err(e) = Self::download_avatar(&comment.avatar, &avatar_path).await {
                            eprintln!("下载头像失败 {}: {}", comment.uname, e);
                        }
                    }
                }
            }

            csv_file.flush()
                .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

            total_downloaded += comments.len();
            eprintln!("已下载 {} 条评论", total_downloaded);

            // 检查是否结束
            if is_end {
                eprintln!("已到达最后一页，下载完成");
                break;
            }

            // 更新pagination_str
            if let Some(offset) = next_offset {
                pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
            } else {
                eprintln!("未找到next_offset，下载完成");
                break;
            }

            // 延迟，避免请求过快
            if delay_seconds > 0 {
                eprintln!("等待 {} 秒后继续...", delay_seconds);
                sleep(Duration::from_secs(delay_seconds)).await;
            }
        }

        let result_msg = format!("评论下载完成！共下载 {} 条评论，保存至: {}", total_downloaded, csv_path.display());
        eprintln!("{}", result_msg);

        Ok(result_msg)
    }

    /// 下载所有评论到指定文件路径
    pub async fn download_comments_to_file(
        aid: i64,
        csv_file_path: &str,
        delay_seconds: u64,
        sessdata: Option<&str>,
    ) -> Result<String, String> {
        Self::download_comments_to_file_with_progress(
            aid,
            csv_file_path,
            delay_seconds,
            sessdata,
            |_, _| {},
        ).await
    }

    pub async fn download_comments_to_file_with_progress<F>(
        aid: i64,
        csv_file_path: &str,
        delay_seconds: u64,
        sessdata: Option<&str>,
        progress_callback: F,
    ) -> Result<String, String>
    where
        F: Fn(usize, Option<usize>) + Send + 'static,
    {
        use std::fs::{create_dir_all, File};
        use std::io::Write;
        use std::path::Path;
        use tokio::time::{sleep, Duration};

        let csv_path = Path::new(csv_file_path);

        // 创建父目录
        if let Some(parent) = csv_path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // 创建CSV文件并写入表头
        let mut csv_file = File::create(&csv_path)
            .map_err(|e| format!("创建CSV文件失败: {}", e))?;
        writeln!(csv_file, "rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent")
            .map_err(|e| format!("写入CSV表头失败: {}", e))?;

        let mut total_downloaded = 0;
        let mut pagination_str = r#"{"offset":""}"#.to_string();
        let mut estimated_total: Option<usize> = None;

        eprintln!("开始下载评论，aid: {}", aid);

        loop {
            eprintln!("正在获取评论，pagination_str: {}", pagination_str);

            // 获取评论
            let result = Self::fetch_comments(aid, &pagination_str, sessdata).await;

            let (comments, next_offset, is_end) = match result {
                Ok(data) => data,
                Err(e) => {
                    // If we have downloaded some comments, treat as incomplete
                    if total_downloaded > 0 {
                        eprintln!("评论下载中断（已下载 {} 条）: {}", total_downloaded, e);
                        return Err(format!("评论API请求失败: {}", e));
                    } else {
                        return Err(e);
                    }
                }
            };

            if comments.is_empty() {
                eprintln!("没有更多评论，下载完成");
                break;
            }

            // 保存评论到CSV
            for comment in &comments {
                let content_escaped = comment.content.replace("\"", "\"\"").replace("\n", " ");
                let location_escaped = comment.location.replace("\"", "\"\"");
                let uname_escaped = comment.uname.replace("\"", "\"\"");

                let line = format!(
                    "{},{},{},\"{}\",{},\"{}\",{},{},{},{},\"{}\",{}\n",
                    comment.rpid,
                    comment.oid,
                    comment.mid,
                    uname_escaped,
                    comment.sex,
                    content_escaped,
                    comment.avatar,
                    comment.ctime,
                    comment.like,
                    comment.current_level,
                    location_escaped,
                    comment.parent
                );

                csv_file.write_all(line.as_bytes())
                    .map_err(|e| format!("写入CSV失败: {}", e))?;
            }

            csv_file.flush()
                .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

            total_downloaded += comments.len();
            eprintln!("已下载 {} 条评论", total_downloaded);

            // Call progress callback
            progress_callback(total_downloaded, estimated_total);

            // 检查是否结束
            if is_end {
                eprintln!("已到达最后一页，下载完成");
                break;
            }

            // 更新pagination_str
            if let Some(offset) = next_offset {
                pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
            } else {
                eprintln!("未找到next_offset，下载完成");
                break;
            }

            // 延迟，避免请求过快
            if delay_seconds > 0 {
                eprintln!("等待 {} 秒后继续...", delay_seconds);
                sleep(Duration::from_secs(delay_seconds)).await;
            }
        }

        let result_msg = format!("评论下载完成！共下载 {} 条评论，保存至: {}", total_downloaded, csv_path.display());
        eprintln!("{}", result_msg);

        Ok(result_msg)
    }

    pub async fn download_comments_to_file_old(
        aid: i64,
        csv_file_path: &str,
        delay_seconds: u64,
        sessdata: Option<&str>,
    ) -> Result<String, String> {
        use std::fs::{create_dir_all, File};
        use std::io::Write;
        use std::path::Path;
        use tokio::time::{sleep, Duration};

        let csv_path = Path::new(csv_file_path);

        // 创建父目录
        if let Some(parent) = csv_path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // 创建CSV文件并写入表头
        let mut csv_file = File::create(&csv_path)
            .map_err(|e| format!("创建CSV文件失败: {}", e))?;
        writeln!(csv_file, "rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent")
            .map_err(|e| format!("写入CSV表头失败: {}", e))?;

        let mut total_downloaded = 0;
        let mut pagination_str = r#"{"offset":""}"#.to_string();

        eprintln!("开始下载评论，aid: {}", aid);

        loop {
            eprintln!("正在获取评论，pagination_str: {}", pagination_str);

            // 获取评论
            let (comments, next_offset, is_end) = Self::fetch_comments(aid, &pagination_str, sessdata).await?;

            if comments.is_empty() {
                eprintln!("没有更多评论，下载完成");
                break;
            }

            // 保存评论到CSV
            for comment in &comments {
                let content_escaped = comment.content.replace("\"", "\"\"").replace("\n", " ");
                let location_escaped = comment.location.replace("\"", "\"\"");
                let uname_escaped = comment.uname.replace("\"", "\"\"");

                let line = format!(
                    "{},{},{},\"{}\",{},\"{}\",{},{},{},{},\"{}\",{}\n",
                    comment.rpid,
                    comment.oid,
                    comment.mid,
                    uname_escaped,
                    comment.sex,
                    content_escaped,
                    comment.avatar,
                    comment.ctime,
                    comment.like,
                    comment.current_level,
                    location_escaped,
                    comment.parent
                );

                csv_file.write_all(line.as_bytes())
                    .map_err(|e| format!("写入CSV失败: {}", e))?;
            }

            csv_file.flush()
                .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

            total_downloaded += comments.len();
            eprintln!("已下载 {} 条评论", total_downloaded);

            // 检查是否结束
            if is_end {
                eprintln!("已到达最后一页，下载完成");
                break;
            }

            // 更新pagination_str
            if let Some(offset) = next_offset {
                pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
            } else {
                eprintln!("未找到next_offset，下载完成");
                break;
            }

            // 延迟，避免请求过快
            if delay_seconds > 0 {
                eprintln!("等待 {} 秒后继续...", delay_seconds);
                sleep(Duration::from_secs(delay_seconds)).await;
            }
        }

        let result_msg = format!("评论下载完成！共下载 {} 条评论，保存至: {}", total_downloaded, csv_path.display());
        eprintln!("{}", result_msg);

        Ok(result_msg)
    }

    /// 下载头像
    async fn download_avatar(avatar_url: &str, save_path: &Path) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let response = client
            .get(avatar_url)
            .send()
            .await
            .map_err(|e| format!("下载头像请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("下载头像失败: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("读取头像数据失败: {}", e))?;

        std::fs::write(save_path, bytes)
            .map_err(|e| format!("保存头像文件失败: {}", e))?;

        Ok(())
    }

    /// 获取WBI密钥（带缓存）
    async fn get_wbi_keys() -> Result<(String, String), String> {
        // 检查缓存
        {
            let cache = WBI_KEYS_CACHE.lock().unwrap();
            if let Some((img_key, sub_key, timestamp)) = cache.as_ref() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // 缓存10分钟
                if now - timestamp < 600 {
                    return Ok((img_key.clone(), sub_key.clone()));
                }
            }
        }

        // 获取新的密钥
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let response = client
            .get("https://api.bilibili.com/x/web-interface/nav")
            .send()
            .await
            .map_err(|e| format!("获取WBI密钥失败: {}", e))?;

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("解析WBI密钥响应失败: {}", e))?;

        let wbi_img = json
            .get("data")
            .and_then(|d| d.get("wbi_img"))
            .ok_or("未找到wbi_img字段")?;

        let img_url = wbi_img
            .get("img_url")
            .and_then(|v| v.as_str())
            .ok_or("未找到img_url")?;

        let sub_url = wbi_img
            .get("sub_url")
            .and_then(|v| v.as_str())
            .ok_or("未找到sub_url")?;

        // 从URL中提取文件名（去除扩展名）
        let img_key = img_url
            .split('/')
            .last()
            .and_then(|s| s.split('.').next())
            .ok_or("无法提取img_key")?
            .to_string();

        let sub_key = sub_url
            .split('/')
            .last()
            .and_then(|s| s.split('.').next())
            .ok_or("无法提取sub_key")?
            .to_string();

        // 更新缓存
        {
            let mut cache = WBI_KEYS_CACHE.lock().unwrap();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            *cache = Some((img_key.clone(), sub_key.clone(), now));
        }

        Ok((img_key, sub_key))
    }

    /// 生成混合密钥
    fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
        let combined = format!("{}{}", img_key, sub_key);
        let mut mixin_key = String::new();

        for &index in MIXIN_KEY_ENC_TAB.iter() {
            if index < combined.len() {
                if let Some(ch) = combined.chars().nth(index) {
                    mixin_key.push(ch);
                }
            }
        }

        // 取前32个字符
        mixin_key.chars().take(32).collect()
    }

    /// 对参数进行WBI签名
    async fn sign_wbi_params(params: &mut Vec<(String, String)>) -> Result<(), String> {
        // 获取WBI密钥
        let (img_key, sub_key) = Self::get_wbi_keys().await?;
        let mixin_key = Self::get_mixin_key(&img_key, &sub_key);

        // 添加时��戳
        let wts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        params.push(("wts".to_string(), wts.to_string()));

        // 按key排序
        params.sort_by(|a, b| a.0.cmp(&b.0));

        // 构建查询字符串
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // 计算MD5
        let to_hash = format!("{}{}", query_string, mixin_key);
        let digest = md5::compute(to_hash.as_bytes());
        let w_rid = format!("{:x}", digest);

        // 添加w_rid参数
        params.push(("w_rid".to_string(), w_rid));

        Ok(())
    }
}
