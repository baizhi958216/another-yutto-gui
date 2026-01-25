use crate::models::download::{DownloadConfig, DownloadTask, DownloadStatus, VideoInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, BufReader};
use std::process::Stdio;
use regex::Regex;

pub struct DownloadManager {
    pub active_downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    pub max_concurrent: Arc<AtomicUsize>,
}

impl DownloadManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent: Arc::new(AtomicUsize::new(max_concurrent.max(1))),
        }
    }

    pub fn set_max_concurrent(&self, max_concurrent: usize) {
        self.max_concurrent.store(max_concurrent.max(1), Ordering::SeqCst);
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent.load(Ordering::SeqCst)
    }

    async fn start_pending_if_available(
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
        max_concurrent: Arc<AtomicUsize>,
    ) {
        let mut tasks_to_start: Vec<(String, DownloadConfig)> = Vec::new();
        {
            let mut downloads_guard = downloads.lock().await;
            let active_count = downloads_guard
                .values()
                .filter(|task| task.status == DownloadStatus::Downloading)
                .count();
            let max_allowed = max_concurrent.load(Ordering::SeqCst);
            let mut slots = max_allowed.saturating_sub(active_count);

            if slots == 0 {
                return;
            }

            let mut pending_tasks: Vec<(String, i64)> = downloads_guard
                .iter()
                .filter_map(|(id, task)| {
                    if task.status == DownloadStatus::Pending {
                        Some((id.clone(), task.start_time))
                    } else {
                        None
                    }
                })
                .collect();
            pending_tasks.sort_by_key(|(_, start_time)| *start_time);

            for (task_id, _) in pending_tasks {
                if slots == 0 {
                    break;
                }
                if let Some(task) = downloads_guard.get_mut(&task_id) {
                    task.status = DownloadStatus::Downloading;
                    task.progress = 0.0;
                    task.speed = "0 KB/s".to_string();
                    task.eta = "--:--".to_string();
                    task.error = None;
                    task.start_time = Self::now_millis();
                    tasks_to_start.push((task_id.clone(), task.config.clone()));
                    slots = slots.saturating_sub(1);
                }
            }
        }

        for (task_id, config) in tasks_to_start {
            Self::spawn_download_task(task_id, config, downloads.clone(), max_concurrent.clone());
        }
    }

    fn spawn_download_task(
        task_id: String,
        config: DownloadConfig,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
        max_concurrent: Arc<AtomicUsize>,
    ) {
        tokio::spawn(async move {
            let result = Self::run_yutto_download(task_id.clone(), config, downloads.clone()).await;
            if let Err(e) = result {
                let mut downloads = downloads.lock().await;
                if let Some(task) = downloads.get_mut(&task_id) {
                    task.status = DownloadStatus::Error;
                    task.error = Some(e);
                }
            }
            Self::start_pending_if_available(downloads, max_concurrent).await;
        });
    }

    fn now_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    pub async fn start_download(&self, task_id: String, config: DownloadConfig, video_info: Option<VideoInfo>) -> Result<(), String> {
        let mut downloads = self.active_downloads.lock().await;
        let active_count = downloads
            .values()
            .filter(|task| task.status == DownloadStatus::Downloading)
            .count();
        let max_allowed = self.max_concurrent();
        let status = if active_count >= max_allowed {
            DownloadStatus::Pending
        } else {
            DownloadStatus::Downloading
        };

        // Create initial task
        let task = DownloadTask {
            id: task_id.clone(),
            config: config.clone(),
            status,
            progress: 0.0,
            speed: "0 KB/s".to_string(),
            eta: "--:--".to_string(),
            error: None,
            warning: None,
            video_info: video_info.clone(),
            total_size: 0, // Will be updated from yutto output
            saved_file_path: None, // Will be parsed from yutto output
            comment_file_path: None, // Will be set after comment download
            comment_download_progress: None, // Will be updated during comment download
            is_downloading_comments: false, // Will be set to true when downloading comments
            start_time: Self::now_millis(),
        };

        downloads.insert(task_id.clone(), task);
        drop(downloads);

        if status == DownloadStatus::Downloading {
            Self::spawn_download_task(task_id, config, self.active_downloads.clone(), self.max_concurrent.clone());
        }

        Ok(())
    }

    async fn run_yutto_download(
        task_id: String,
        config: DownloadConfig,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    ) -> Result<(), String> {
        // Build yutto command
        let mut cmd = match config
            .yutto_cli_path
            .as_ref()
            .map(|path| path.trim())
            .filter(|path| !path.is_empty())
        {
            Some(path) => Command::new(path),
            None => Command::new("yutto"),
        };
        cmd.arg(&config.url)
            .arg("-q").arg(config.video_quality.to_string())
            .arg("-aq").arg(config.audio_quality.to_string())
            .arg("-d").arg(&config.download_path)
            .arg("--no-color")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set UTF-8 environment variables to fix Windows encoding issues
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUTF8", "1");

        // Add sessdata if provided
        if let Some(sessdata) = &config.sessdata {
            cmd.arg("-c").arg(sessdata);
        }

        // Add optional flags
        if !config.with_danmaku {
            cmd.arg("--no-danmaku");
        }
        if !config.with_subtitle {
            cmd.arg("--no-subtitle");
        }
        if !config.with_cover {
            cmd.arg("--no-cover");
        }

        // Add video-only or audio-only flags
        if config.video_only.unwrap_or(false) {
            cmd.arg("--video-only");
        }
        if config.audio_only.unwrap_or(false) {
            cmd.arg("--audio-only");
        }

        // Hide console window on Windows when launching yutto.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        // Log the command being executed
        println!("[下载管理器] 开始下载任务 {}", task_id);
        println!("[下载管理器] URL: {}", config.url);
        println!("[下载管理器] 下载路径: {}", config.download_path);

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            println!("[下载管理器] 启动失败: {}", e);
            format!("启动 yutto 失败: {}", e)
        })?;

        // Get stdout and stderr
        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

        // Create readers
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn tasks to read output
        let downloads_clone = downloads.clone();
        let task_id_clone = task_id.clone();
        let stdout_handle = tokio::spawn(async move {
            Self::process_output(stdout_reader, task_id_clone, downloads_clone).await;
        });

        let downloads_clone = downloads.clone();
        let task_id_clone = task_id.clone();
        let stderr_handle = tokio::spawn(async move {
            Self::process_output(stderr_reader, task_id_clone, downloads_clone).await;
        });

        // Wait for the process to complete
        let status = child.wait().await.map_err(|e| format!("等待进程失败: {}", e))?;

        // Wait for output processing to complete
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        if status.success() {
            println!("[下载管理器] 视频下载完成: {}", task_id);

            // Download comments if requested
            if config.with_comments {
                println!("[下载管理器] 开始下载评论...");
                match Self::download_comments_for_task(&task_id, &config, &downloads).await {
                    Ok(_) => {
                        println!("[下载管理器] 评论下载完成");
                    }
                    Err(e) => {
                        println!("[下载管理器] 评论下载失败: {}", e);
                        // Set warning but don't fail the entire task
                        let mut downloads = downloads.lock().await;
                        if let Some(task) = downloads.get_mut(&task_id) {
                            task.warning = Some(format!("评论下载失败: {}", e));
                        }
                    }
                }
            }

            // Mark as completed
            let mut downloads = downloads.lock().await;
            if let Some(task) = downloads.get_mut(&task_id) {
                task.status = DownloadStatus::Completed;
                task.progress = 100.0;
            }
            Ok(())
        } else {
            println!("[下载管理器] 下载失败，退出码: {:?}", status.code());
            Err(format!("下载失败，退出码: {:?}", status.code()))
        }
    }

    async fn download_comments_for_task(
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
        let video_info = BilibiliApi::fetch_video_info_from_html(
            &config.url,
            config.sessdata.as_deref(),
            false,
        ).await?;

        // Determine comment file path
        let saved_file_path = {
            let downloads_guard = downloads.lock().await;
            downloads_guard.get(task_id)
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
        ).await;

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

    async fn process_output<R: tokio::io::AsyncRead + Unpin>(
        reader: BufReader<R>,
        task_id: String,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    ) {
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

                                // Only print non-progress lines to avoid spam
                                if !line.contains("MiB/") && !line.is_empty() {
                                    println!("[yutto] {}", line);
                                }

                                // Try to parse progress information
                                let mut updated = false;
                                let mut progress_val = None;
                                let mut speed_val = None;
                                let mut total_size_val = None;
                                let mut saved_path_val = None;

                                // Parse: "77.86 MiB/110.21 MiB"
                                if let Some(caps) = progress_re.captures(line) {
                                    if let (Ok(current), Ok(total)) = (
                                        caps[1].parse::<f64>(),
                                        caps[2].parse::<f64>(),
                                    ) {
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
                                            println!("[文件大小] {} bytes ({:.2} MiB)", size, size as f64 / 1024.0 / 1024.0);
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

    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        let mut downloads = self.active_downloads.lock().await;
        downloads.remove(task_id);
        drop(downloads);
        Self::start_pending_if_available(self.active_downloads.clone(), self.max_concurrent.clone()).await;
        Ok(())
    }

    pub async fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        let downloads = self.active_downloads.lock().await;
        downloads.get(task_id).cloned()
    }

    pub async fn start_pending(&self) {
        Self::start_pending_if_available(self.active_downloads.clone(), self.max_concurrent.clone()).await;
    }
}
