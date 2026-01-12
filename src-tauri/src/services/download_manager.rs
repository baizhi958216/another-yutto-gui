use crate::models::download::{DownloadConfig, DownloadTask, DownloadStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, BufReader};
use std::process::Stdio;
use regex::Regex;

pub struct DownloadManager {
    pub active_downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    pub max_concurrent: usize,
}

impl DownloadManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent,
        }
    }

    pub async fn start_download(&self, task_id: String, config: DownloadConfig) -> Result<(), String> {
        let mut downloads = self.active_downloads.lock().await;

        // Create initial task
        let task = DownloadTask {
            id: task_id.clone(),
            config: config.clone(),
            status: DownloadStatus::Downloading,
            progress: 0.0,
            speed: "0 KB/s".to_string(),
            eta: "--:--".to_string(),
            error: None,
        };

        downloads.insert(task_id.clone(), task);
        drop(downloads);

        // Spawn download task
        let downloads_clone = self.active_downloads.clone();
        let task_id_clone = task_id.clone();
        tokio::spawn(async move {
            match Self::run_yutto_download(task_id_clone.clone(), config, downloads_clone.clone()).await {
                Ok(_) => {
                    // Success is already handled in run_yutto_download
                }
                Err(e) => {
                    let mut downloads = downloads_clone.lock().await;
                    if let Some(task) = downloads.get_mut(&task_id_clone) {
                        task.status = DownloadStatus::Error;
                        task.error = Some(e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn run_yutto_download(
        task_id: String,
        config: DownloadConfig,
        downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    ) -> Result<(), String> {
        // Build yutto command
        let mut cmd = Command::new("yutto");
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
            println!("[下载管理器] 下载完成: {}", task_id);
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
        Ok(())
    }

    pub async fn get_task(&self, task_id: &str) -> Option<DownloadTask> {
        let downloads = self.active_downloads.lock().await;
        downloads.get(task_id).cloned()
    }
}
