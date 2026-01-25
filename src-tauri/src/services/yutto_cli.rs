use crate::models::video::{VideoInfo, Owner};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::process::Command;

pub struct YuttoCli;

impl YuttoCli {
    pub async fn fetch_video_info(url: &str, sessdata: Option<&str>) -> Result<VideoInfo, String> {
        // 创建临时目录用于存放元数据
        let temp_dir = std::env::temp_dir().join(format!("yutto_metadata_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

        // 构建 yutto 命令
        let mut cmd = Command::new("yutto");
        cmd.arg(url)
            .arg("--metadata-only")
            .arg("--dir")
            .arg(&temp_dir)
            .arg("--no-color")
            .arg("--no-progress");

        // 如果提供了 SESSDATA，添加到命令中
        if let Some(sessdata) = sessdata {
            cmd.arg("-c").arg(sessdata);
        }

        // 执行命令
        let output = cmd.output().map_err(|e| format!("执行 yutto 命令失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("yutto 命令执行失败: {}", stderr));
        }

        // 查找生成的 .nfo 文件
        let nfo_files: Vec<_> = std::fs::read_dir(&temp_dir)
            .map_err(|e| format!("读取临时目录失败: {}", e))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension().and_then(|s| s.to_str()) == Some("nfo")
            })
            .collect();

        if nfo_files.is_empty() {
            // 清理临时目录
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err("未找到元数据文件".to_string());
        }

        // 读取第一个 .nfo 文件
        let nfo_path = nfo_files[0].path();
        let xml_content = std::fs::read_to_string(&nfo_path)
            .map_err(|e| format!("读取元数据文件失败: {}", e))?;

        // 解析 XML
        let video_info = Self::parse_xml(&xml_content)?;

        // 清理临时目录
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(video_info)
    }

    fn parse_xml(xml: &str) -> Result<VideoInfo, String> {
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
        let (bvid, aid) = Self::extract_video_ids(&website)?;

        // 从 profile URL 中提取 uid
        let uid = Self::extract_uid(&owner_profile);

        Ok(VideoInfo {
            title,
            bvid,
            aid,
            thumbnail,
            duration: 0, // XML 中没有 duration 信息
            description,
            owner: Owner {
                uid,
                name: owner_name,
                face: owner_face,
            },
            episodes: None,
            available_qualities: None,
            available_audio_qualities: None,
            comment_count: None, // XML 中没有评论数量信息
        })
    }

    fn extract_video_ids(url: &str) -> Result<(String, i64), String> {
        // 从 URL 中提取 BV 号
        let re = regex::Regex::new(r"(BV[a-zA-Z0-9]+)").unwrap();
        if let Some(caps) = re.captures(url) {
            let bvid = caps[1].to_string();
            // 暂时使用 0 作为 aid，因为 XML 中没有提供
            return Ok((bvid, 0));
        }

        Err("无法从 URL 中提取视频 ID".to_string())
    }

    fn extract_uid(profile_url: &str) -> i64 {
        // 从 profile URL 中提取 UID
        let re = regex::Regex::new(r"space\.bilibili\.com/(\d+)").unwrap();
        if let Some(caps) = re.captures(profile_url) {
            if let Ok(uid) = caps[1].parse::<i64>() {
                return uid;
            }
        }
        0
    }
}
