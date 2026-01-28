use crate::utils::http_client::create_bilibili_client;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// WBI签名相关常量
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49,
    33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40,
    61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11,
    36, 20, 34, 44, 52,
];

// WBI密钥缓存
static WBI_KEYS_CACHE: Mutex<Option<(String, String, u64)>> = Mutex::new(None);

/// 获取WBI密钥（带缓存）
pub async fn get_wbi_keys() -> Result<(String, String), String> {
    // 检查缓存
    {
        let cache = WBI_KEYS_CACHE.lock().unwrap();
        if let Some((img_key, sub_key, timestamp)) = cache.as_ref() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            // 缓存10分钟
            if now - timestamp < 600 {
                return Ok((img_key.clone(), sub_key.clone()));
            }
        }
    }

    // 获取新的密钥
    let client = create_bilibili_client()?;

    let response = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await
        .map_err(|e| format!("获取WBI密钥失败: {}", e))?;

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析WBI密钥响应失败: {}", e))?;

    let wbi_img = json
        .get("data")
        .and_then(|d| d.get("wbi_img"))
        .ok_or("未找到wbi_img字段")?;

    let img_url = wbi_img
        .get("img_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到img_url")?;

    let sub_url = wbi_img
        .get("sub_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到sub_url")?;

    // 从URL中提取文件名（去除扩展名）
    let img_key = img_url
        .split('/')
        .last()
        .and_then(|s| s.split('.').next())
        .ok_or("无法提取img_key")?
        .to_string();

    let sub_key = sub_url
        .split('/')
        .last()
        .and_then(|s| s.split('.').next())
        .ok_or("无法提取sub_key")?
        .to_string();

    // 更新缓存
    {
        let mut cache = WBI_KEYS_CACHE.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        *cache = Some((img_key.clone(), sub_key.clone(), now));
    }

    Ok((img_key, sub_key))
}

/// 生成混合密钥
fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    let combined = format!("{}{}", img_key, sub_key);
    let mut mixin_key = String::new();

    for &index in MIXIN_KEY_ENC_TAB.iter() {
        if index < combined.len() {
            if let Some(ch) = combined.chars().nth(index) {
                mixin_key.push(ch);
            }
        }
    }

    // 取前32个字符
    mixin_key.chars().take(32).collect()
}

/// 对参数进行WBI签名
pub async fn sign_wbi_params(params: &mut Vec<(String, String)>) -> Result<(), String> {
    // 获取WBI密钥
    let (img_key, sub_key) = get_wbi_keys().await?;
    let mixin_key = get_mixin_key(&img_key, &sub_key);

    // 添加时间戳
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    params.push(("wts".to_string(), wts.to_string()));

    // 按key排序
    params.sort_by(|a, b| a.0.cmp(&b.0));

    // 构建查询字符串
    let query_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // 计算MD5
    let to_hash = format!("{}{}", query_string, mixin_key);
    let digest = md5::compute(to_hash.as_bytes());
    let w_rid = format!("{:x}", digest);

    // 添加w_rid参数
    params.push(("w_rid".to_string(), w_rid));

    Ok(())
}
