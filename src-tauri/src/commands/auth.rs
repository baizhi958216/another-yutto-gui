use tauri::{AppHandle, Manager, State, Emitter, Listener};
use tauri::webview::WebviewWindowBuilder;
use crate::services::storage::Storage;
use url::Url;

// Validate SESSDATA format (at least 32 characters)
fn validate_sessdata_format(sessdata: &str) -> bool {
    // SESSDATA should be at least 32 characters
    // It can be longer due to URL encoding or other reasons
    sessdata.len() >= 32
}

#[tauri::command]
pub async fn save_sessdata(sessdata: String, storage: State<'_, Storage>) -> Result<(), String> {
    if !validate_sessdata_format(&sessdata) {
        return Err("Invalid SESSDATA format. Must be 32 hexadecimal characters.".to_string());
    }

    storage
        .save_sessdata(sessdata)
        .map_err(|e| format!("Failed to save SESSDATA: {}", e))
}

#[tauri::command]
pub async fn get_sessdata(storage: State<'_, Storage>) -> Result<Option<String>, String> {
    storage
        .get_sessdata()
        .map_err(|e| format!("Failed to get SESSDATA: {}", e))
}

#[tauri::command]
pub async fn validate_sessdata(sessdata: String) -> Result<bool, String> {
    if !validate_sessdata_format(&sessdata) {
        return Ok(false);
    }

    // Call Bilibili API to validate SESSDATA
    let client = reqwest::Client::new();
    let response = client
        .get("https://bilibili.com")
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await
        .map_err(|e| format!("Failed to validate SESSDATA: {}", e))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Check if code is 0 (success)
    Ok(json["code"].as_i64() == Some(0))
}

#[tauri::command]
pub async fn clear_auth(storage: State<'_, Storage>) -> Result<(), String> {
    storage
        .clear_sessdata()
        .map_err(|e| format!("Failed to clear auth: {}", e))
}

#[tauri::command]
pub async fn check_vip_status(sessdata: String) -> Result<bool, String> {
    if !validate_sessdata_format(&sessdata) {
        eprintln!("[VIP Check] Invalid SESSDATA format");
        return Ok(false);
    }

    // Call Bilibili API to check VIP status
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await
        .map_err(|e| format!("Failed to check VIP status: {}", e))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    eprintln!("[VIP Check] API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

    // Check if code is 0 (success) and vip.status is 1 or vip.type is 2 (annual VIP)
    if json["code"].as_i64() == Some(0) {
        if let Some(data) = json.get("data") {
            if let Some(vip) = data.get("vip") {
                eprintln!("[VIP Check] VIP data: {}", serde_json::to_string_pretty(vip).unwrap_or_default());

                // Check vip.type: 0=无, 1=月度大会员, 2=年度大会员
                if let Some(vip_type) = vip.get("type").and_then(|v| v.as_i64()) {
                    eprintln!("[VIP Check] VIP type: {}", vip_type);
                    if vip_type >= 1 {
                        return Ok(true);
                    }
                }

                // Also check vip.status as fallback
                if let Some(vip_status) = vip.get("status").and_then(|v| v.as_i64()) {
                    eprintln!("[VIP Check] VIP status: {}", vip_status);
                    if vip_status == 1 {
                        return Ok(true);
                    }
                }
            }
        }
    }

    eprintln!("[VIP Check] User is not VIP");
    Ok(false)
}

#[tauri::command]
pub async fn open_login_window(app: AppHandle) -> Result<(), String> {
    // Create login window
    let webview = WebviewWindowBuilder::new(
        &app,
        "bilibili-login",
        tauri::WebviewUrl::External("https://passport.bilibili.com/login".parse().unwrap()),
    )
    .title("登录 Bilibili")
    .inner_size(500.0, 700.0)
    .center()
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to create login window: {}", e))?;

    // Inject cookie monitoring script that checks for login success
    let script = r#"
(function() {
  console.log('[Bilibili Login] Monitoring for login success');
  let checkCount = 0;
  const interval = setInterval(() => {
    checkCount++;

    // Check if user is logged in by looking for DedeUserID cookie
    const cookies = document.cookie;
    const hasDedeUserID = cookies.includes('DedeUserID');

    if (checkCount % 10 === 0) {
      console.log('[Bilibili Login] Check #' + checkCount + ', URL:', window.location.href);
      console.log('[Bilibili Login] Has DedeUserID:', hasDedeUserID);
    }

    // If logged in, just log it - backend will detect and close the window
    if (hasDedeUserID && window.location.hostname.includes('bilibili.com')) {
      console.log('[Bilibili Login] Login detected! Backend will close this window automatically.');
      clearInterval(interval);
    }
  }, 1000);

  setTimeout(() => {
    console.log('[Bilibili Login] Timeout reached after 5 minutes');
    clearInterval(interval);
  }, 300000);
})();
"#;

    webview
        .eval(script)
        .map_err(|e| format!("Failed to inject script: {}", e))?;

    // Monitor the window for navigation to detect login success
    let app_handle = app.clone();
    let webview_label = webview.label().to_string();

    // Poll for login success by checking cookies periodically
    tauri::async_runtime::spawn(async move {
        let mut check_count = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            check_count += 1;

            if check_count % 5 == 0 {
                println!("[Auth] Polling check #{}", check_count);
            }

            if let Some(window) = app_handle.get_webview_window(&webview_label) {
                // Try to get cookies
                let domains = vec![
                    "https://bilibili.com",
                    "https://www.bilibili.com",
                    "https://passport.bilibili.com",
                ];

                let mut found_sessdata = false;

                for domain in &domains {
                    if let Ok(url) = Url::parse(domain) {
                        match window.cookies_for_url(url) {
                            Ok(cookies) => {
                                if check_count % 5 == 0 {
                                    println!("[Auth] Found {} cookies for domain: {}", cookies.len(), domain);
                                }

                                for cookie in cookies {
                                    if cookie.name() == "SESSDATA" {
                                        let sessdata = cookie.value().to_string();
                                        println!("[Auth] Found SESSDATA (length: {})", sessdata.len());

                                        // Save SESSDATA
                                        if let Some(storage) = app_handle.try_state::<Storage>() {
                                            if let Err(e) = storage.save_sessdata(sessdata.clone()) {
                                                eprintln!("[Auth] Failed to save SESSDATA: {}", e);
                                                continue;
                                            }
                                            println!("[Auth] SESSDATA saved successfully");

                                            // Emit login-success event
                                            if let Err(e) = app_handle.emit("login-success", sessdata) {
                                                eprintln!("[Auth] Failed to emit login-success: {}", e);
                                            }

                                            // Close login window
                                            if let Err(e) = window.close() {
                                                eprintln!("[Auth] Failed to close window: {}", e);
                                            } else {
                                                println!("[Auth] Window closed successfully");
                                            }

                                            found_sessdata = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if check_count % 5 == 0 {
                                    eprintln!("[Auth] Failed to get cookies for {}: {:?}", domain, e);
                                }
                            }
                        }
                    }

                    if found_sessdata {
                        break;
                    }
                }

                if found_sessdata {
                    break;
                }

                // Timeout after 5 minutes
                if check_count > 150 {
                    println!("[Auth] Timeout reached after 5 minutes");
                    break;
                }
            } else {
                // Window was closed
                println!("[Auth] Login window was closed");
                break;
            }
        }
    });

    Ok(())
}
