use std::fmt;

/// Custom error types for Yutto GUI
#[derive(Debug)]
pub enum YuttoError {
    /// HTTP client creation or request errors
    HttpClient(String),
    /// API request errors (non-2xx status codes)
    ApiRequest(String),
    /// JSON parsing errors
    JsonParse(String),
    /// File system errors (read, write, delete)
    FileSystem(String),
    /// Database errors
    Database(String),
    /// Video information errors
    VideoInfo(String),
    /// Download errors
    Download(String),
    /// Authentication errors
    Auth(String),
    /// Process control errors
    Process(String),
}

impl fmt::Display for YuttoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YuttoError::HttpClient(msg) => write!(f, "HTTP客户端错误: {}", msg),
            YuttoError::ApiRequest(msg) => write!(f, "API请求失败: {}", msg),
            YuttoError::JsonParse(msg) => write!(f, "JSON解析失败: {}", msg),
            YuttoError::FileSystem(msg) => write!(f, "文件系统错误: {}", msg),
            YuttoError::Database(msg) => write!(f, "数据库错误: {}", msg),
            YuttoError::VideoInfo(msg) => write!(f, "视频信息错误: {}", msg),
            YuttoError::Download(msg) => write!(f, "下载错误: {}", msg),
            YuttoError::Auth(msg) => write!(f, "认证错误: {}", msg),
            YuttoError::Process(msg) => write!(f, "进程控制错误: {}", msg),
        }
    }
}

impl std::error::Error for YuttoError {}

/// Convert YuttoError to String for Tauri commands
impl From<YuttoError> for String {
    fn from(err: YuttoError) -> String {
        err.to_string()
    }
}

/// Convert from reqwest::Error
impl From<reqwest::Error> for YuttoError {
    fn from(err: reqwest::Error) -> Self {
        YuttoError::HttpClient(err.to_string())
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for YuttoError {
    fn from(err: serde_json::Error) -> Self {
        YuttoError::JsonParse(err.to_string())
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for YuttoError {
    fn from(err: std::io::Error) -> Self {
        YuttoError::FileSystem(err.to_string())
    }
}
