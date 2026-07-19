mod json_parser;
mod output_parser;

use crate::models::download::{DownloadConfig, DownloadStatus, DownloadTask, VideoInfo};
use crate::utils::bundled_binaries;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::Mutex;

pub struct DownloadManager {
    pub active_downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    pub max_concurrent: Arc<AtomicUsize>,
    pub app_handle: tauri::AppHandle,
}

impl DownloadManager {
    pub fn new(max_concurrent: usize, app_handle: tauri::AppHandle) -> Self {
        Self {
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent: Arc::new(AtomicUsize::new(max_concurrent.max(1))),
            app_handle,
        }
    }

    pub fn set_max_concurrent(&self, max_concurrent: usize) {
        self.max_concurrent
            .store(max_concurrent.max(1), Ordering::SeqCst);
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent.load(Ordering::SeqCst)
    }

    fn now_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    async fn start_pending_if_available(
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
        max_concurrent: Arc<AtomicUsize>,
        app_handle: tauri::AppHandle,
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
            Self::spawn_download_task(
                task_id,
                config,
                downloads.clone(),
                max_concurrent.clone(),
                app_handle.clone(),
            );
        }
    }

    fn spawn_download_task(
        task_id: String,
        config: DownloadConfig,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
        max_concurrent: Arc<AtomicUsize>,
        app_handle: tauri::AppHandle,
    ) {
        tokio::spawn(async move {
            let result = Self::run_yutto_download(
                task_id.clone(),
                config,
                downloads.clone(),
                app_handle.clone(),
            )
            .await;
            if let Err(e) = result {
                let mut downloads = downloads.lock().await;
                if let Some(task) = downloads.get_mut(&task_id) {
                    task.status = DownloadStatus::Error;
                    task.error = Some(e);
                }
            }
            Self::start_pending_if_available(downloads, max_concurrent, app_handle).await;
        });
    }

    pub async fn start_download(
        &self,
        task_id: String,
        config: DownloadConfig,
        video_info: Option<VideoInfo>,
    ) -> Result<(), String> {
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
            total_size: 0,                   // Will be updated from yutto output
            downloaded_bytes: 0,             // NEW: Will be updated from JSON output
            total_bytes: 0,                  // NEW: Will be updated from JSON output
            speed_bytes_per_sec: 0.0,        // NEW: Will be updated from JSON output
            eta_seconds: None,               // NEW: Will be updated from JSON output
            files_count: None,               // NEW: Will be updated from JSON output
            saved_file_path: None,           // Will be parsed from yutto output
            comment_file_path: None,         // Will be set after comment download
            comment_download_progress: None, // Will be updated during comment download
            is_downloading_comments: false,  // Will be set to true when downloading comments
            start_time: Self::now_millis(),
            process_id: None, // Will be set after process spawns
            paused_at_progress: None,
            paused_at_speed: None,
        };

        downloads.insert(task_id.clone(), task);
        drop(downloads);

        if status == DownloadStatus::Downloading {
            Self::spawn_download_task(
                task_id,
                config,
                self.active_downloads.clone(),
                self.max_concurrent.clone(),
                self.app_handle.clone(),
            );
        }

        Ok(())
    }

    async fn run_yutto_download(
        task_id: String,
        config: DownloadConfig,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Build yutto command (使用打包的二进制文件或系统版本)
        let yutto_path =
            bundled_binaries::get_yutto_command_path(&app_handle, config.yutto_cli_path.as_deref());
        let mut cmd = Command::new(&yutto_path);
        cmd.arg(&config.url)
            .arg("-q")
            .arg(config.video_quality.to_string())
            .arg("-aq")
            .arg(config.audio_quality.to_string())
            .arg("-d")
            .arg(&config.download_path)
            .arg("--no-color")
            .arg("--json-output") // 使用 JSON 输出模式获取精确进度
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set UTF-8 environment variables to fix Windows encoding issues
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
        if let Some(path_with_ffmpeg) = bundled_binaries::get_path_with_ffmpeg(&app_handle) {
            cmd.env("PATH", path_with_ffmpeg);
        }

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

        // Add episodes parameter if provided
        if let Some(episodes) = &config.episodes {
            if !episodes.is_empty() {
                // Check if URL needs batch mode (same logic as yutto_cli.rs)
                let url_lower = config.url.to_lowercase();
                let needs_batch = url_lower.contains("/bangumi/play/ss")
                    || url_lower.contains("/bangumi/media/md")
                    || url_lower.starts_with("ss")
                    || url_lower.starts_with("md")
                    || url_lower.contains("/cheese/play/ss")
                    || url_lower.contains("space.bilibili.com/")
                    || url_lower.contains("watchlater")
                    || url_lower.contains("/list/");

                if needs_batch {
                    eprintln!("[download_manager] 检测到需要批量模式的链接，添加 -b 参数");
                    cmd.arg("-b");
                }

                cmd.arg("-p").arg(episodes);
                eprintln!("[download_manager] 添加剧集参数: -p {}", episodes);
            }
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

        // Capture process ID for pause/resume control
        let pid = child.id();
        println!("[下载管理器] 进程 PID: {:?}", pid);

        // Store PID in task
        {
            let mut downloads_guard = downloads.lock().await;
            if let Some(task) = downloads_guard.get_mut(&task_id) {
                task.process_id = pid;
            }
        }

        // Get stdout and stderr
        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

        // Create readers
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn tasks to read output
        let downloads_clone = downloads.clone();
        let task_id_clone = task_id.clone();
        let app_handle_clone = app_handle.clone();
        let stdout_handle = tokio::spawn(async move {
            output_parser::process_output(
                stdout_reader,
                task_id_clone,
                downloads_clone,
                app_handle_clone,
            )
            .await;
        });

        let downloads_clone = downloads.clone();
        let task_id_clone = task_id.clone();
        let app_handle_clone = app_handle.clone();
        let stderr_handle = tokio::spawn(async move {
            output_parser::process_output(
                stderr_reader,
                task_id_clone,
                downloads_clone,
                app_handle_clone,
            )
            .await
        });

        // Wait for the process to complete
        let status = child
            .wait()
            .await
            .map_err(|e| format!("等待进程失败: {}", e))?;

        // Wait for output processing to complete
        let _ = stdout_handle.await;
        let stderr_lines = stderr_handle.await.unwrap_or_default();

        if status.success() {
            println!("[下载管理器] 视频下载完成: {}", task_id);

            // Download comments if requested
            if config.with_comments {
                println!("[下载管理器] 开始下载评论...");
                match output_parser::download_comments_for_task(&task_id, &config, &downloads).await
                {
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
                task.process_id = None; // Clear PID on completion
            }
            Ok(())
        } else {
            let exit_code = status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "未知".to_string());
            let details = stderr_lines
                .into_iter()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");

            let error = if details.is_empty() {
                format!("下载失败，退出码: {}", exit_code)
            } else {
                format!("下载失败，退出码: {}\n{}", exit_code, details)
            };
            println!("[下载管理器] {}", error);
            Err(error)
        }
    }

    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        use crate::services::process_control::ProcessController;

        let mut downloads = self.active_downloads.lock().await;

        // Get PID before removing task
        let pid = downloads.get(task_id).and_then(|task| task.process_id);

        // Kill process if it exists
        if let Some(pid) = pid {
            if ProcessController::is_process_alive(pid) {
                let _ = ProcessController::kill_process(pid);
                println!("[下载管理器] 已终止进程 PID: {}", pid);
            }
        }

        // Remove task
        downloads.remove(task_id);
        drop(downloads);

        // Start pending tasks
        Self::start_pending_if_available(
            self.active_downloads.clone(),
            self.max_concurrent.clone(),
            self.app_handle.clone(),
        )
        .await;
        Ok(())
    }

    pub async fn pause_download(&self, task_id: &str) -> Result<(), String> {
        use crate::services::process_control::ProcessController;

        let mut downloads = self.active_downloads.lock().await;
        let task = downloads.get_mut(task_id).ok_or("Task not found")?;

        // Check if task is downloading
        if task.status != DownloadStatus::Downloading {
            return Err("Task is not downloading".to_string());
        }

        // Get PID
        let pid = task.process_id.ok_or("Process ID not available")?;

        // Save current state
        task.paused_at_progress = Some(task.progress);
        task.paused_at_speed = Some(task.speed.clone());

        // Pause the process
        ProcessController::pause_process(pid)?;

        // Update status
        task.status = DownloadStatus::Paused;

        println!("[下载管理器] 已暂停任务 {} (PID: {})", task_id, pid);
        Ok(())
    }

    pub async fn resume_download(&self, task_id: &str) -> Result<(), String> {
        use crate::services::process_control::ProcessController;

        let mut downloads = self.active_downloads.lock().await;
        let task = downloads.get_mut(task_id).ok_or("Task not found")?;

        // Check if task is paused
        if task.status != DownloadStatus::Paused {
            return Err("Task is not paused".to_string());
        }

        // Get PID
        let pid = task.process_id.ok_or("Process ID not available")?;

        // Check if process still exists
        if !ProcessController::is_process_alive(pid) {
            task.status = DownloadStatus::Error;
            task.error = Some("进程已终止".to_string());
            task.process_id = None;
            return Err("Process no longer exists".to_string());
        }

        // Resume the process
        ProcessController::resume_process(pid)?;

        // Update status
        task.status = DownloadStatus::Downloading;

        println!("[下载管理器] 已恢复任务 {} (PID: {})", task_id, pid);
        Ok(())
    }

    pub async fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        let downloads = self.active_downloads.lock().await;
        downloads.get(task_id).cloned()
    }

    pub async fn start_pending(&self) {
        Self::start_pending_if_available(
            self.active_downloads.clone(),
            self.max_concurrent.clone(),
            self.app_handle.clone(),
        )
        .await;
    }
}
