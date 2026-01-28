use crate::models::video::Owner;
use crate::utils::http_client::create_bilibili_client;
use serde_json::Value;

/// 获取收藏夹用户信息
pub async fn fetch_favorite_owner_info(fid: i64, sessdata: Option<&str>) -> Result<Owner, String> {
    // 构建收藏夹信息 API URL
    let api_url = format!("https://api.bilibili.com/x/v3/fav/folder/info?media_id={}", fid);

    eprintln!("调用收藏夹信息 API: {}", api_url);

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("收藏夹信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("收藏夹信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取收藏夹信息响应失败: {}", e))?;

    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析收藏夹信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("收藏夹信息API返回错误: {} (code: {})", message, code));
    }

    let data = json.get("data")
        .ok_or("收藏夹信息API响应中未找到data字段")?;

    let uid = data.get("mid")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // 获取用户详细信息
    fetch_user_info(uid, sessdata).await
}

/// 获取用户详细信息
pub async fn fetch_user_info(uid: i64, sessdata: Option<&str>) -> Result<Owner, String> {
    // 构建用户信息 API URL
    let api_url = format!("https://api.bilibili.com/x/space/wbi/acc/info?mid={}", uid);

    eprintln!("调用用户信息 API: {}", api_url);

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("用户信息API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("用户信息API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取用户信息响应失败: {}", e))?;

    let json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析用户信息JSON失败: {}", e))?;

    let code = json.get("code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    if code != 0 {
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        return Err(format!("用户信息API返回错误: {} (code: {})", message, code));
    }

    let data = json.get("data")
        .ok_or("用户信息API响应中未找到data字段")?;

    let owner = Owner {
        uid,
        name: data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        face: data.get("face")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sign: data.get("sign")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        level: data.get("level")
            .and_then(|v| v.as_i64())
            .map(|l| l as i32),
        location: data.get("live_room")
            .and_then(|lr| lr.get("area_name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    eprintln!("成功获取用户信息: {}", owner.name);

    Ok(owner)
}
