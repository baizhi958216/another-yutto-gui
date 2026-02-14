/// 处理打包的二进制文件（yutto 和 ffmpeg）
///
/// 此模块提供获取打包的 yutto 和 ffmpeg 二进制文件路径的功能。
/// 实现了回退机制：优先使用打包的版本，如果不存在则回退到系统版本。
use std::path::PathBuf;
use tauri::Manager;

/// 获取平台特定的二进制文件名
fn get_platform_binary_name(base_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        return format!("{}-x86_64-pc-windows-msvc.exe", base_name);
        #[cfg(target_arch = "aarch64")]
        return format!("{}-aarch64-pc-windows-msvc.exe", base_name);
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "x86_64")]
        return format!("{}-x86_64-apple-darwin", base_name);
        #[cfg(target_arch = "aarch64")]
        return format!("{}-aarch64-apple-darwin", base_name);
    }

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        return format!("{}-x86_64-unknown-linux-gnu", base_name);
        #[cfg(target_arch = "aarch64")]
        return format!("{}-aarch64-unknown-linux-gnu", base_name);
    }

    // 默认回退
    #[allow(unreachable_code)]
    base_name.to_string()
}

/// 获取打包的 yutto 可执行文件路径
///
/// 返回打包在应用中的 yutto 二进制文件的完整路径。
/// 如果文件不存在，返回 None。
pub fn get_bundled_yutto_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    // Tauri 会自动根据平台选择正确的二进制文件：
    // - Windows: binaries/yutto-x86_64-pc-windows-msvc.exe
    // - macOS Intel: binaries/yutto-x86_64-apple-darwin
    // - macOS Apple Silicon: binaries/yutto-aarch64-apple-darwin
    // - Linux: binaries/yutto-x86_64-unknown-linux-gnu

    let resource_path = app_handle
        .path()
        .resolve("yutto", tauri::path::BaseDirectory::Resource)
        .ok()?;

    eprintln!(
        "[bundled_binaries] 尝试使用打包的 yutto: {:?}",
        resource_path
    );

    if resource_path.exists() {
        eprintln!("[bundled_binaries] 找到打包的 yutto");
        return Some(resource_path);
    }

    eprintln!("[bundled_binaries] 未找到打包的 yutto，尝试开发模式路径");

    // 开发模式回退：尝试从项目根目录的 binaries 文件夹加载
    #[cfg(debug_assertions)]
    {
        let dev_path = std::env::current_dir()
            .ok()?
            .join("binaries")
            .join(get_platform_binary_name("yutto"));

        eprintln!("[bundled_binaries] 尝试开发模式路径: {:?}", dev_path);

        if dev_path.exists() {
            eprintln!("[bundled_binaries] 找到开发模式的 yutto");
            return Some(dev_path);
        }

        // 如果当前目录不对，尝试从 src-tauri 的父目录查找
        let parent_dev_path = std::env::current_dir()
            .ok()?
            .parent()?
            .join("binaries")
            .join(get_platform_binary_name("yutto"));

        eprintln!(
            "[bundled_binaries] 尝试父目录开发模式路径: {:?}",
            parent_dev_path
        );

        if parent_dev_path.exists() {
            eprintln!("[bundled_binaries] 找到父目录开发模式的 yutto");
            return Some(parent_dev_path);
        }
    }

    eprintln!("[bundled_binaries] 未找到打包的 yutto");
    None
}

/// 获取打包的 ffmpeg 可执行文件路径
///
/// 返回打包在应用中的 ffmpeg 二进制文件的完整路径。
/// 如果文件不存在，返回 None。
pub fn get_bundled_ffmpeg_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    let resource_path = app_handle
        .path()
        .resolve("ffmpeg", tauri::path::BaseDirectory::Resource)
        .ok()?;

    eprintln!(
        "[bundled_binaries] 尝试使用打包的 ffmpeg: {:?}",
        resource_path
    );

    if resource_path.exists() {
        eprintln!("[bundled_binaries] 找到打包的 ffmpeg");
        return Some(resource_path);
    }

    eprintln!("[bundled_binaries] 未找到打包的 ffmpeg，尝试开发模式路径");

    // 开发模式回退：尝试从项目根目录的 binaries 文件夹加载
    #[cfg(debug_assertions)]
    {
        let dev_path = std::env::current_dir()
            .ok()?
            .join("binaries")
            .join(get_platform_binary_name("ffmpeg"));

        eprintln!("[bundled_binaries] 尝试开发模式路径: {:?}", dev_path);

        if dev_path.exists() {
            eprintln!("[bundled_binaries] 找到开发模式的 ffmpeg");
            return Some(dev_path);
        }

        // 如果当前目录不对，尝试从 src-tauri 的父目录查找
        let parent_dev_path = std::env::current_dir()
            .ok()?
            .parent()?
            .join("binaries")
            .join(get_platform_binary_name("ffmpeg"));

        eprintln!(
            "[bundled_binaries] 尝试父目录开发模式路径: {:?}",
            parent_dev_path
        );

        if parent_dev_path.exists() {
            eprintln!("[bundled_binaries] 找到父目录开发模式的 ffmpeg");
            return Some(parent_dev_path);
        }
    }

    eprintln!("[bundled_binaries] 未找到打包的 ffmpeg");
    None
}

/// 获取 yutto 命令路径（带回退机制）
///
/// 优先级：
/// 1. 用户配置的自定义路径（如果提供）
/// 2. 打包的 yutto 二进制文件
/// 3. 系统 PATH 中的 yutto
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `custom_path`: 用户配置的自定义 yutto 路径（可选）
///
/// # 返回
/// 返回 yutto 可执行文件的路径字符串
pub fn get_yutto_command_path(app_handle: &tauri::AppHandle, custom_path: Option<&str>) -> String {
    // 1. 优先使用用户配置的自定义路径
    if let Some(path) = custom_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            eprintln!("[bundled_binaries] 使用自定义 yutto 路径: {}", trimmed);
            return trimmed.to_string();
        }
    }

    // 2. 尝试使用打包的版本
    if let Some(bundled_path) = get_bundled_yutto_path(app_handle) {
        eprintln!("[bundled_binaries] 使用打包的 yutto");
        return bundled_path.to_string_lossy().to_string();
    }

    // 3. 回退到系统版本
    eprintln!("[bundled_binaries] 回退到系统 yutto");
    "yutto".to_string()
}

/// 设置 ffmpeg 环境变量
///
/// 如果找到打包的 ffmpeg，将其目录添加到 PATH 环境变量中，
/// 以便 yutto 可以找到并使用它。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
///
/// # 返回
/// 返回更新后的 PATH 环境变量值（如果找到打包的 ffmpeg）
pub fn get_path_with_ffmpeg(app_handle: &tauri::AppHandle) -> Option<String> {
    if let Some(ffmpeg_path) = get_bundled_ffmpeg_path(app_handle) {
        if let Some(ffmpeg_dir) = ffmpeg_path.parent() {
            let current_path = std::env::var("PATH").unwrap_or_default();

            // 根据平台使用不同的路径分隔符
            #[cfg(windows)]
            let separator = ";";
            #[cfg(not(windows))]
            let separator = ":";

            let new_path = format!("{}{}{}", ffmpeg_dir.display(), separator, current_path);
            eprintln!(
                "[bundled_binaries] 添加 ffmpeg 到 PATH: {}",
                ffmpeg_dir.display()
            );
            return Some(new_path);
        }
    }
    None
}

/// 检查是否有可用的 yutto
///
/// 检查是否可以找到 yutto（打包版本或系统版本）
pub fn is_yutto_available(app_handle: &tauri::AppHandle) -> bool {
    // 检查打包版本
    if get_bundled_yutto_path(app_handle).is_some() {
        return true;
    }

    // 检查系统版本
    #[cfg(windows)]
    let check_cmd = std::process::Command::new("where").arg("yutto").output();

    #[cfg(not(windows))]
    let check_cmd = std::process::Command::new("which").arg("yutto").output();

    if let Ok(output) = check_cmd {
        output.status.success()
    } else {
        false
    }
}

/// 检查是否有可用的 ffmpeg
///
/// 检查是否可以找到 ffmpeg（打包版本或系统版本）
pub fn is_ffmpeg_available(app_handle: &tauri::AppHandle) -> bool {
    // 检查打包版本
    if get_bundled_ffmpeg_path(app_handle).is_some() {
        return true;
    }

    // 检查系统版本
    #[cfg(windows)]
    let check_cmd = std::process::Command::new("where").arg("ffmpeg").output();

    #[cfg(not(windows))]
    let check_cmd = std::process::Command::new("which").arg("ffmpeg").output();

    if let Ok(output) = check_cmd {
        output.status.success()
    } else {
        false
    }
}
