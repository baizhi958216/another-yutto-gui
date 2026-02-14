use crate::models::download::{DownloadConfig, DownloadTask};
use crate::services::download_manager::json_parser::parse_progress_line;
use crate::utils::format::{format_eta, format_speed};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::BufReader;
use tokio::sync::Mutex;

/// Process output from yutto command and update task progress
pub async fn process_output<R: tokio::io::AsyncRead + Unpin>(
    reader: BufReader<R>,
    task_id: String,
    downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    app_handle: tauri::AppHandle,
) {
    use tokio::io::AsyncReadExt;

    let mut reader = reader;
    let mut buffer = vec![0u8; 4096];
    let mut line_buffer = String::new();

    // Regex patterns for parsing progress
    // Example from yutto: "77.86 MiB/110.21 MiB 56.30 MiB/s"
    let progress_re = Regex::new(r"(\d+\.?\d*)\s*MiB/(\d+\.?\d*)\s*MiB").unwrap();
    let speed_re = Regex::new(r"(\d+\.?\d*)\s*([KMGT]?iB/s)").unwrap();
    // Parse total size from progress bar: "/ 75.70 MiB"
    let total_size_re = Regex::new(r"/\s*(\d+\.?\d*)\s*(Bytes|KiB|MiB|GiB|TiB)").unwrap();
    // Parse saved file path: "保存路径：" or "Saved to:" followed by file path
    let saved_path_re = Regex::new(r"(?:保存路径：|Saved to:|保存为：|已保存：)\s*(.+)").unwrap();

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);

                for ch in chunk.chars() {
                    if ch == '\r' || ch == '\n' {
                        if !line_buffer.is_empty() {
                            // Process the line
                            let line = line_buffer.trim();

                            // Try JSON parsing first
                            if let Some(parsed) = parse_progress_line(line) {
                                // JSON parsing succeeded - update task with precise data
                                let mut downloads_guard = downloads.lock().await;
                                if let Some(task) = downloads_guard.get_mut(&task_id) {
                                    task.progress = parsed.progress;
                                    task.downloaded_bytes = parsed.downloaded_bytes;
                                    task.total_bytes = parsed.total_bytes;
                                    task.speed_bytes_per_sec = parsed.speed_bytes_per_sec;
                                    task.eta_seconds = parsed.eta_seconds;
                                    task.files_count = parsed.files_count;

                                    // Update formatted strings for backward compatibility
                                    task.speed = format_speed(parsed.speed_bytes_per_sec);
                                    task.eta = parsed
                                        .eta_seconds
                                        .map(|s| format_eta(s))
                                        .unwrap_or_else(|| "--:--".to_string());
                                    task.total_size = parsed.total_bytes;

                                    // Clone task for event emission
                                    let task_clone = task.clone();

                                    // Release lock before emitting event
                                    drop(downloads_guard);

                                    // Emit real-time event to frontend
                                    if let Err(e) =
                                        app_handle.emit("download-progress", &task_clone)
                                    {
                                        eprintln!("[输出解析器] 发射事件失败: {}", e);
                                    }

                                    println!(
                                        "[进度] {}% - {} - ETA: {}",
                                        task_clone.progress.round(),
                                        task_clone.speed,
                                        task_clone.eta
                                    );
                                }

                                line_buffer.clear();
                                continue;
                            }

                            // Fallback to regex parsing for non-JSON output
                            // Only print non-progress lines to avoid spam
                            if !line.contains("MiB/") && !line.is_empty() {
                                println!("[yutto] {}", line);
                            }

                            // Try to parse progress information using regex
                            let mut updated = false;
                            let mut progress_val = None;
                            let mut speed_val = None;
                            let mut total_size_val = None;
                            let mut saved_path_val = None;

                            // Parse: "77.86 MiB/110.21 MiB"
                            if let Some(caps) = progress_re.captures(line) {
                                if let (Ok(current), Ok(total)) =
                                    (caps[1].parse::<f64>(), caps[2].parse::<f64>())
                                {
                                    if total > 0.0 {
                                        let percentage = (current / total) * 100.0;
                                        progress_val = Some(percentage);
                                        updated = true;
                                    }
                                }
                            }

                            // Parse: "56.30 MiB/s"
                            if let Some(caps) = speed_re.captures(line) {
                                speed_val = Some(format!("{} {}", &caps[1], &caps[2]));
                                updated = true;
                            }

                            // Parse total size: "/ 75.70 MiB"
                            if let Some(caps) = total_size_re.captures(line) {
                                if let Ok(size) = caps[1].parse::<f64>() {
                                    let unit = &caps[2];
                                    let bytes = match unit {
                                        "Bytes" => size as i64,
                                        "KiB" => (size * 1024.0) as i64,
                                        "MiB" => (size * 1024.0 * 1024.0) as i64,
                                        "GiB" => (size * 1024.0 * 1024.0 * 1024.0) as i64,
                                        "TiB" => (size * 1024.0 * 1024.0 * 1024.0 * 1024.0) as i64,
                                        _ => 0,
                                    };
                                    total_size_val = Some(bytes);
                                    updated = true;
                                }
                            }

                            // Parse saved file path
                            if let Some(caps) = saved_path_re.captures(line) {
                                let path = caps[1].trim().to_string();
                                let path_obj = std::path::Path::new(&path);
                                if !path_obj.is_dir() {
                                    saved_path_val = Some(path.clone());
                                    println!("[文件路径] {}", path);
                                    updated = true;
                                }
                            }

                            // Update task if we found any progress info
                            if updated {
                                let mut downloads = downloads.lock().await;
                                if let Some(task) = downloads.get_mut(&task_id) {
                                    if let Some(p) = progress_val {
                                        task.progress = p;
                                        println!("[进度] {}%", p.round());
                                    }
                                    if let Some(s) = speed_val {
                                        task.speed = s;
                                    }
                                    if let Some(size) = total_size_val {
                                        task.total_size = size;
                                        println!(
                                            "[文件大小] {} bytes ({:.2} MiB)",
                                            size,
                                            size as f64 / 1024.0 / 1024.0
                                        );
                                    }
                                    if let Some(path) = saved_path_val {
                                        task.saved_file_path = Some(path);
                                    }
                                    // Calculate simple ETA based on progress
                                    if let Some(p) = progress_val {
                                        if p > 0.0 && p < 100.0 {
                                            task.eta = "计算中...".to_string();
                                        }
                                    }
                                }
                            }

                            line_buffer.clear();
                        }
                    } else {
                        line_buffer.push(ch);
                    }
                }
            }
            Err(e) => {
                println!("[yutto] 读取输出错误: {}", e);
                break;
            }
        }
    }

    // Process any remaining content
    if !line_buffer.is_empty() {
        println!("[yutto] {}", line_buffer.trim());
    }
}

/// Download comments for a completed video task
pub async fn download_comments_for_task(
    task_id: &str,
    config: &DownloadConfig,
    downloads: &Arc<Mutex<HashMap<String, DownloadTask>>>,
) -> Result<String, String> {
    use crate::services::bilibili_api::BilibiliApi;

    // Set is_downloading_comments flag
    {
        let mut downloads_guard = downloads.lock().await;
        if let Some(task) = downloads_guard.get_mut(task_id) {
            task.is_downloading_comments = true;
        }
    }

    // Get video info to extract aid and bvid
    let video_info =
        BilibiliApi::fetch_video_info_from_html(&config.url, config.sessdata.as_deref(), false)
            .await?;

    // Determine comment file path
    let saved_file_path = {
        let downloads_guard = downloads.lock().await;
        downloads_guard
            .get(task_id)
            .and_then(|task| task.saved_file_path.clone())
    };

    let comment_file_path = if let Some(video_path) = saved_file_path {
        // Use the same directory as the video file
        let path = std::path::Path::new(&video_path);
        let parent = path.parent().ok_or("无法获取视频文件目录")?;
        let file_stem = path.file_stem().ok_or("无法获取视频文件名")?;
        let comment_file = parent.join(format!("{}_comments.csv", file_stem.to_string_lossy()));
        comment_file.to_string_lossy().to_string()
    } else {
        // Fallback: use download path with bvid
        let download_path = std::path::Path::new(&config.download_path);
        let comment_file = download_path.join(format!("{}_comments.csv", video_info.bvid));
        comment_file.to_string_lossy().to_string()
    };

    println!("[下载管理器] 评论文件路径: {}", comment_file_path);

    // Download comments with progress callback
    let downloads_clone = downloads.clone();
    let task_id_clone = task_id.to_string();
    let progress_callback = move |current: usize, total: Option<usize>| {
        let downloads = downloads_clone.clone();
        let task_id = task_id_clone.clone();
        tokio::spawn(async move {
            let mut downloads_guard = downloads.lock().await;
            if let Some(task) = downloads_guard.get_mut(&task_id) {
                if let Some(total) = total {
                    task.comment_download_progress = Some(format!("{}/{}", current, total));
                } else {
                    task.comment_download_progress = Some(format!("{}", current));
                }
            }
        });
    };

    let result = BilibiliApi::download_comments_to_file_with_progress(
        video_info.aid,
        &comment_file_path,
        3, // 3 second delay between requests
        config.sessdata.as_deref(),
        progress_callback,
    )
    .await;

    // Clear is_downloading_comments flag
    {
        let mut downloads_guard = downloads.lock().await;
        if let Some(task) = downloads_guard.get_mut(task_id) {
            task.is_downloading_comments = false;
            task.comment_download_progress = None;
        }
    }

    match result {
        Ok(_) => {
            // Update task with comment file path
            let mut downloads_guard = downloads.lock().await;
            if let Some(task) = downloads_guard.get_mut(task_id) {
                task.comment_file_path = Some(comment_file_path.clone());
            }
            Ok(comment_file_path)
        }
        Err(e) => {
            // Check if it's a network error that should be treated as incomplete
            if e.contains("error sending request for url") || e.contains("评论API请求失败") {
                println!("[下载管理器] 评论下载网络错误，标记为不完整: {}", e);
                let mut downloads_guard = downloads.lock().await;
                if let Some(task) = downloads_guard.get_mut(task_id) {
                    task.warning = Some("评论下载不完整（网络错误）".to_string());
                    // Still set the comment file path if some comments were downloaded
                    if std::path::Path::new(&comment_file_path).exists() {
                        task.comment_file_path = Some(comment_file_path.clone());
                    }
                }
                Ok(comment_file_path)
            } else {
                Err(e)
            }
        }
    }
}
