use reqwest::Client;

/// 创建用于 Bilibili API 的 HTTP 客户端
///
/// 使用标准的浏览器 User-Agent 以避免被 Bilibili API 拒绝
pub fn create_bilibili_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
}

/// 创建带 Cookie 的 HTTP 客户端
///
/// 用于需要认证的 Bilibili API 请求
pub fn create_authenticated_client(cookie: &str) -> Result<Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(cookie)
            .map_err(|e| format!("无效的 Cookie: {}", e))?,
    );

    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .default_headers(headers)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
}
