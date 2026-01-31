mod quality_fetcher;
mod episode_handler;
mod xml_parser;

use crate::models::video::{VideoInfo, Episode};
use crate::utils::bundled_binaries;
use std::process::Command;

#[cfg(windows)]
use encoding_rs::GBK;

pub struct YuttoCli;

/// 解码命令输出（现在统一使用 UTF-8）
fn decode_output(bytes: &[u8]) -> String {
    // 由于我们设置了 PYTHONIOENCODING=utf-8，输出应该是 UTF-8 编码
    // 如果解码失败，尝试使用 GBK（向后兼容）
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            #[cfg(windows)]
            {
                let (cow, _, _) = GBK.decode(bytes);
                cow.into_owned()
            }
            #[cfg(not(windows))]
            {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }
    }
}

impl YuttoCli {
    pub async fn fetch_video_info(
        app_handle: &tauri::AppHandle,
        url: &str,
        sessdata: Option<&str>,
        is_vip: bool
    ) -> Result<VideoInfo, String> {
        // 创建临时目录用于存放元数据
        let temp_dir = std::env::temp_dir().join(format!("yutto_metadata_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

        let url_lower = url.to_lowercase();

        // 根据 yutto 文档判断是否需要批量模式
        // 需要批量模式的情况：
        // 1. 番剧 SS/MD 号（EP 号单话不需要）
        // 2. 课程 SS 号（EP 号单话不需要）
        // 3. 收藏夹、空间、列表等（这些本身就是批量）
        let needs_batch =
            // 番剧 SS/MD 号
            url_lower.contains("/bangumi/play/ss")
            || url_lower.contains("/bangumi/media/md")
            || url_lower.starts_with("ss")
            || url_lower.starts_with("md")
            // 课程 SS 号
            || url_lower.contains("/cheese/play/ss")
            // 收藏夹、空间、列表（这些必须批量）
            || url_lower.contains("space.bilibili.com/")
            || url_lower.contains("watchlater")
            || url_lower.contains("/list/");

        // 构建 yutto 命令（使用打包的二进制文件或系统版本）
        let yutto_path = bundled_binaries::get_yutto_command_path(app_handle, None);
        let mut cmd = Command::new(&yutto_path);
        cmd.arg(url)
            .arg("--metadata-only")
            .arg("--dir")
            .arg(&temp_dir)
            .arg("--no-color")
            .arg("--no-progress");

        // 如果需要批量模式，添加 -b 参数
        if needs_batch {
            eprintln!("[yutto_cli] 检测到需要批量模式的链接，添加 -b 参数");
            cmd.arg("-b");
        }

        // 如果提供了 SESSDATA，添加到命令中
        if let Some(sessdata) = sessdata {
            cmd.arg("-c").arg(sessdata);
        }

        // 设置环境变量以强制使用 UTF-8 编码
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUTF8", "1");

        // Additional encoding fixes for Windows
        #[cfg(windows)]
        {
            cmd.env("PYTHONLEGACYWINDOWSSTDIO", "0");
            cmd.env("PYTHONLEGACYWINDOWSFSENCODING", "0");
            // Force UTF-8 for stdout/stderr
            cmd.env("PYTHONSTDOUTENCODING", "utf-8");
            cmd.env("PYTHONSTDERRENCODING", "utf-8");
            // Set console code page to UTF-8
            cmd.env("CHCP", "65001");
            // Disable colorama auto-init which may cause encoding issues
            cmd.env("COLORAMA_AUTORESET", "0");
            cmd.env("COLORAMA_STRIP", "1");
        }

        // 如果有打包的 ffmpeg，添加到 PATH
        if let Some(path_with_ffmpeg) = bundled_binaries::get_path_with_ffmpeg(app_handle) {
            cmd.env("PATH", path_with_ffmpeg);
        }

        // 执行命令
        eprintln!("[yutto_cli] 执行命令: {:?}", cmd);
        let output = cmd.output().map_err(|e| {
            eprintln!("[yutto_cli] 命令执行错误: {}", e);
            format!("执行 yutto 命令失败: {}. 请确保已安装 yutto (pip install yutto)", e)
        })?;

        if !output.status.success() {
            let stderr = decode_output(&output.stderr);
            let stdout = decode_output(&output.stdout);
            eprintln!("[yutto_cli] 命令执行失败");
            eprintln!("[yutto_cli] stdout: {}", stdout);
            eprintln!("[yutto_cli] stderr: {}", stderr);
            return Err(format!("yutto 命令执行失败: {}\n{}", stdout, stderr));
        }

        eprintln!("[yutto_cli] 命令执行成功，查找 .nfo 文件");

        // 查找生成的 .nfo 文件（可能在子目录中）
        let nfo_files = Self::find_nfo_files(&temp_dir)?;

        eprintln!("[yutto_cli] 找到 {} 个 .nfo 文件", nfo_files.len());

        if nfo_files.is_empty() {
            // 列出临时目录中的所有文件以便调试
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                eprintln!("[yutto_cli] 临时目录中的文件:");
                for entry in entries.filter_map(|e| e.ok()) {
                    eprintln!("[yutto_cli]   - {:?}", entry.path());
                }
            }
            // 清理临时目录
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err("未找到元数据文件 (.nfo)。yutto 可能没有正确生成元数据文件。".to_string());
        }

        // 列出所有找到的 .nfo 文件
        eprintln!("[yutto_cli] 找到的 .nfo 文件列表:");
        for (i, nfo_file) in nfo_files.iter().enumerate() {
            eprintln!("[yutto_cli]   [{}] {:?}", i, nfo_file);
        }

        // 从第一个文件路径中提取番剧名称（父目录名）
        let series_name = nfo_files[0]
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        eprintln!("[yutto_cli] 提取到系列名称: {}", series_name);

        // 解析所有 .nfo 文件以获取剧集列表
        let mut episodes = Vec::new();
        let mut base_video_info = None;
        let mut parse_errors = Vec::new();

        for (idx, nfo_path) in nfo_files.iter().enumerate() {
            match std::fs::read_to_string(nfo_path) {
                Ok(xml_content) => {
                    // 从文件名提取剧集序号
                    let episode_index = episode_handler::extract_episode_index(nfo_path, idx);

                    // 解析剧集信息
                    match episode_handler::parse_episode_xml(&xml_content, episode_index) {
                        Ok(episode) => {
                            eprintln!("[yutto_cli] 解析剧集 {}: {} (index: {})", idx + 1, episode.title, episode.index);
                            episodes.push(episode);
                        }
                        Err(e) => {
                            eprintln!("[yutto_cli] 解析剧集 {} 失败: {}", idx + 1, e);
                            parse_errors.push(format!("剧集 {}: {}", idx + 1, e));
                        }
                    }

                    // 第一个文件作为基础视频信息
                    if idx == 0 {
                        // 输出 XML 内容的前 500 个字符用于调试
                        let preview = if xml_content.len() > 500 {
                            xml_content.chars().take(500).collect::<String>()
                        } else {
                            xml_content.clone()
                        };
                        eprintln!("[yutto_cli] XML 内容预览:\n{}", preview);

                        base_video_info = Some(xml_parser::parse_xml(&xml_content, url, &series_name, sessdata).await?);
                    }
                }
                Err(e) => {
                    eprintln!("[yutto_cli] 读取文件 {} 失败: {}", idx + 1, e);
                    parse_errors.push(format!("剧集 {}: 读取失败", idx + 1));
                }
            }
        }

        // 如果所有剧集都解析失败，返回错误
        if episodes.is_empty() && !parse_errors.is_empty() {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(format!("所有剧集解析失败:\n{}", parse_errors.join("\n")));
        }

        // 按序号排序
        episodes.sort_by_key(|e| e.index);

        // 附加剧集列表到视频信息
        let mut video_info = base_video_info.unwrap();
        if episodes.len() > 1 {
            eprintln!("[yutto_cli] 找到 {} 个剧集，添加到视频信息中", episodes.len());
            video_info.episodes = Some(episodes.clone());
            // 对于多剧集番剧，使用系列名称作为标题
            if !series_name.is_empty() {
                eprintln!("[yutto_cli] 使用系列名称作为番剧标题: {}", series_name);
                video_info.title = series_name;
            }

            // 判断是否是番剧：没有 bvid 且 aid 为 0
            let is_bangumi = video_info.bvid.is_empty() && video_info.aid == 0;

            if is_bangumi {
                // 番剧：使用番剧 API 获取质量选项
                if let Some(first_episode) = episodes.first() {
                    eprintln!("[yutto_cli] 检测到番剧，尝试获取第一个剧集的质量选项，ep_id: {}", first_episode.id);

                    // 如果 ep_id 看起来不对（太小），尝试从 URL 中提取 ss_id 并获取剧集列表
                    let ep_id_to_use = if first_episode.id < 100 {
                        eprintln!("[yutto_cli] ep_id 看起来不正确 ({}), 尝试从 URL 获取 ss_id", first_episode.id);
                        match quality_fetcher::get_first_episode_id_from_season(url).await {
                            Ok(real_ep_id) => {
                                eprintln!("[yutto_cli] 从番剧 API 获取到第一个剧集的 ep_id: {}", real_ep_id);
                                real_ep_id
                            }
                            Err(e) => {
                                eprintln!("[yutto_cli] 从番剧 API 获取 ep_id 失败: {}, 使用原始 ep_id", e);
                                first_episode.id
                            }
                        }
                    } else {
                        first_episode.id
                    };

                    match quality_fetcher::fetch_bangumi_qualities(ep_id_to_use, sessdata, is_vip).await {
                        Ok((qualities, audio_qualities)) => {
                            eprintln!("[yutto_cli] 成功获取番剧质量选项");
                            video_info.available_qualities = Some(qualities);
                            video_info.available_audio_qualities = Some(audio_qualities);
                        }
                        Err(e) => {
                            eprintln!("[yutto_cli] 获取番剧质量选项失败: {}", e);
                        }
                    }
                }
            } else {
                // 普通视频（收藏夹等）：使用普通视频 API 获取质量选项
                eprintln!("[yutto_cli] 检测到普通视频（BV/AV），使用普通视频 API 获取质量选项");
                eprintln!("[yutto_cli] bvid: {}, aid: {}", video_info.bvid, video_info.aid);

                // 获取第一个视频的 cid（需要调用 API）
                match quality_fetcher::fetch_video_cid(&video_info.bvid, video_info.aid, sessdata).await {
                    Ok(cid) => {
                        eprintln!("[yutto_cli] 获取到 cid: {}", cid);
                        match quality_fetcher::fetch_video_qualities(&video_info.bvid, video_info.aid, cid, sessdata, is_vip).await {
                            Ok((qualities, audio_qualities)) => {
                                eprintln!("[yutto_cli] 成功获取普通视频质量选项");
                                video_info.available_qualities = Some(qualities);
                                video_info.available_audio_qualities = Some(audio_qualities);
                            }
                            Err(e) => {
                                eprintln!("[yutto_cli] 获取普通视频质量选项失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[yutto_cli] 获取 cid 失败: {}", e);
                    }
                }
            }
        } else {
            eprintln!("[yutto_cli] 只有 1 个剧集，不添加剧集列表");

            // 判断是否是番剧：没有 bvid 且 aid 为 0
            let is_bangumi = video_info.bvid.is_empty() && video_info.aid == 0;

            if is_bangumi {
                // 番剧：使用番剧 API 获取质量选项
                if let Some(first_episode) = episodes.first() {
                    eprintln!("[yutto_cli] 检测到番剧，尝试获取第一个剧集的质量选项，ep_id: {}", first_episode.id);

                    // 优先从 URL 提取 ep_id（适配单集番剧链接）
                    let ep_id_from_url = episode_handler::extract_episode_id(url, first_episode.id);
                    if ep_id_from_url != first_episode.id {
                        eprintln!("[yutto_cli] 从 URL 提取到 ep_id: {}", ep_id_from_url);
                    }

                    // 如果 ep_id 看起来不对（太小），尝试从 URL 中提取 ss_id 并获取剧集列表
                    let ep_id_to_use = if ep_id_from_url < 100 {
                        eprintln!("[yutto_cli] ep_id 看起来不正确 ({}), 尝试从 URL 获取 ss_id", ep_id_from_url);
                        match quality_fetcher::get_first_episode_id_from_season(url).await {
                            Ok(real_ep_id) => {
                                eprintln!("[yutto_cli] 从番剧 API 获取到第一个剧集的 ep_id: {}", real_ep_id);
                                real_ep_id
                            }
                            Err(e) => {
                                eprintln!("[yutto_cli] 从番剧 API 获取 ep_id 失败: {}, 使用原始 ep_id", e);
                                ep_id_from_url
                            }
                        }
                    } else {
                        ep_id_from_url
                    };

                    match quality_fetcher::fetch_bangumi_qualities(ep_id_to_use, sessdata, is_vip).await {
                        Ok((qualities, audio_qualities)) => {
                            eprintln!("[yutto_cli] 成功获取番剧质量选项");
                            video_info.available_qualities = Some(qualities);
                            video_info.available_audio_qualities = Some(audio_qualities);
                        }
                        Err(e) => {
                            eprintln!("[yutto_cli] 获取番剧质量选项失败: {}", e);
                        }
                    }
                }
            } else {
                // 普通视频（收藏夹等）：使用普通视频 API 获取质量选项
                eprintln!("[yutto_cli] 检测到普通视频（BV/AV），使用普通视频 API 获取质量选项");
                eprintln!("[yutto_cli] bvid: {}, aid: {}", video_info.bvid, video_info.aid);

                // 获取第一个视频的 cid（需要调用 API）
                match quality_fetcher::fetch_video_cid(&video_info.bvid, video_info.aid, sessdata).await {
                    Ok(cid) => {
                        eprintln!("[yutto_cli] 获取到 cid: {}", cid);
                        match quality_fetcher::fetch_video_qualities(&video_info.bvid, video_info.aid, cid, sessdata, is_vip).await {
                            Ok((qualities, audio_qualities)) => {
                                eprintln!("[yutto_cli] 成功获取普通视频质量选项");
                                video_info.available_qualities = Some(qualities);
                                video_info.available_audio_qualities = Some(audio_qualities);
                            }
                            Err(e) => {
                                eprintln!("[yutto_cli] 获取普通视频质量选项失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[yutto_cli] 获取 cid 失败: {}", e);
                    }
                }
            }
        }

        // 清理临时目录
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(video_info)
    }

    /// 递归查找目录中的所有 .nfo 文件
    fn find_nfo_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
        let mut nfo_files = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("nfo") {
                nfo_files.push(path);
            } else if path.is_dir() {
                // 递归查找子目录
                if let Ok(mut sub_files) = Self::find_nfo_files(&path) {
                    nfo_files.append(&mut sub_files);
                }
            }
        }

        Ok(nfo_files)
    }
}
