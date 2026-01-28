use crate::models::comment::{Comment, CommentResponse};
use crate::utils::http_client::create_bilibili_client;
use std::path::Path;

/// 获取视频评论（使用WBI签名API）
pub async fn fetch_comments(aid: i64, pagination_str: &str, sessdata: Option<&str>) -> Result<(Vec<Comment>, Option<String>, bool), String> {
    // 构建参数
    let mut params = vec![
        ("oid".to_string(), aid.to_string()),
        ("type".to_string(), "1".to_string()),
        ("pagination_str".to_string(), pagination_str.to_string()),
        ("plat".to_string(), "1".to_string()),
        ("web_location".to_string(), "1315875".to_string()),
    ];

    // 对参数进行WBI签名
    super::wbi_auth::sign_wbi_params(&mut params).await?;

    // 构建URL
    let query_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let api_url = format!("https://api.bilibili.com/x/v2/reply/wbi/main?{}", query_string);

    eprintln!("调用评论 API: {}", api_url);

    let client = create_bilibili_client()?;

    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("评论API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("评论API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取评论响应失败: {}", e))?;

    let json: CommentResponse = serde_json::from_str(&json_text)
        .map_err(|e| format!("解析评论JSON失败: {}", e))?;

    if json.code != 0 {
        return Err(format!("评论API返回错误: {} (code: {})", json.message, json.code));
    }

    let data = json.data.ok_or("评论API响应中未找到data字段")?;

    // 获取下一页的offset
    let next_offset = data.cursor.pagination_reply
        .as_ref()
        .map(|p| p.next_offset.clone());

    let is_end = data.cursor.is_end;

    let replies = data.replies.unwrap_or_default();

    // 转换为简化的Comment结构
    let comments: Vec<Comment> = replies.iter().map(|reply| {
        Comment {
            rpid: reply.rpid,
            oid: reply.oid,
            mid: reply.mid,
            uname: reply.member.uname.clone(),
            avatar: reply.member.avatar.clone(),
            sex: reply.member.sex.clone(),
            content: reply.content.message.clone(),
            ctime: reply.ctime,
            like: reply.like,
            current_level: reply.member.level_info.current_level,
            location: reply.reply_control.location.clone().unwrap_or_default(),
            parent: reply.parent,
            pictures: reply.content.pictures.clone().unwrap_or_default(),
        }
    }).collect();

    eprintln!("成功获取 {} 条评论", comments.len());

    Ok((comments, next_offset, is_end))
}

/// 下载所有评论到本地文件
pub async fn download_all_comments(
    aid: i64,
    bvid: &str,
    save_path: &str,
    download_avatars: bool,
    delay_seconds: u64,
    sessdata: Option<&str>,
) -> Result<String, String> {
    use std::fs::{create_dir_all, File, OpenOptions};
    use std::io::Write;
    use std::path::Path;
    use tokio::time::{sleep, Duration};

    // 创建保存目录
    let video_dir = Path::new(save_path).join(bvid);
    create_dir_all(&video_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let csv_path = video_dir.join(format!("{}.csv", bvid));
    let avatars_dir = video_dir.join("avatars");

    if download_avatars {
        create_dir_all(&avatars_dir)
            .map_err(|e| format!("创建头像目录失败: {}", e))?;
    }

    // 检查CSV文件是否存在，如果不存在则创建并写入表头
    let file_exists = csv_path.exists();
    let mut csv_file = if file_exists {
        OpenOptions::new()
            .append(true)
            .open(&csv_path)
            .map_err(|e| format!("打开CSV文件失败: {}", e))?
    } else {
        let mut file = File::create(&csv_path)
            .map_err(|e| format!("创建CSV文件失败: {}", e))?;
        // 写入CSV表头
        writeln!(file, "bvid,upname,sex,content,avatar,rpid,oid,mid,parent,ctime,like,level,location")
            .map_err(|e| format!("写入CSV表头失败: {}", e))?;
        file
    };

    let mut total_downloaded = 0;
    let mut pagination_str = r#"{"offset":""}"#.to_string();

    eprintln!("开始下载评论，视频: {}, aid: {}", bvid, aid);

    loop {
        eprintln!("正在获取评论，pagination_str: {}", pagination_str);

        // 获取评论
        let (comments, next_offset, is_end) = fetch_comments(aid, &pagination_str, sessdata).await?;

        if comments.is_empty() {
            eprintln!("没有更多评论，下载完成");
            break;
        }

        // 保存评论到CSV
        for comment in &comments {
            let content_escaped = comment.content.replace("\"", "\"\"").replace("\n", " ");
            let location_escaped = comment.location.replace("\"", "\"\"");
            let uname_escaped = comment.uname.replace("\"", "\"\"");

            let line = format!(
                "{},\"{}\",{},\"{}\",{},{},{},{},{},{},{},{},\"{}\"\n",
                bvid,
                uname_escaped,
                comment.sex,
                content_escaped,
                comment.avatar,
                comment.rpid,
                comment.oid,
                comment.mid,
                comment.parent,
                comment.ctime,
                comment.like,
                comment.current_level,
                location_escaped
            );

            csv_file.write_all(line.as_bytes())
                .map_err(|e| format!("写入CSV失败: {}", e))?;

            // 下载头像
            if download_avatars && !comment.avatar.is_empty() {
                let avatar_filename = format!("{}_{}.jpg", comment.mid, comment.uname.replace("/", "_"));
                let avatar_path = avatars_dir.join(&avatar_filename);

                // 如果头像文件不存在，则下载
                if !avatar_path.exists() {
                    if let Err(e) = download_avatar(&comment.avatar, &avatar_path).await {
                        eprintln!("下载头像失败 {}: {}", comment.uname, e);
                    }
                }
            }
        }

        csv_file.flush()
            .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

        total_downloaded += comments.len();
        eprintln!("已下载 {} 条评论", total_downloaded);

        // 检查是否结束
        if is_end {
            eprintln!("已到达最后一页，下载完成");
            break;
        }

        // 更新pagination_str
        if let Some(offset) = next_offset {
            pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
        } else {
            eprintln!("未找到next_offset，下载完成");
            break;
        }

        // 延迟，避免请求过快
        if delay_seconds > 0 {
            eprintln!("等待 {} 秒后继续...", delay_seconds);
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }

    let result_msg = format!("评论下载完成！共下载 {} 条评论，保存至: {}", total_downloaded, csv_path.display());
    eprintln!("{}", result_msg);

    Ok(result_msg)
}

/// 下载所有评论到指定文件路径
pub async fn download_comments_to_file(
    aid: i64,
    csv_file_path: &str,
    delay_seconds: u64,
    sessdata: Option<&str>,
) -> Result<String, String> {
    download_comments_to_file_with_progress(
        aid,
        csv_file_path,
        delay_seconds,
        sessdata,
        |_, _| {},
    ).await
}

pub async fn download_comments_to_file_with_progress<F>(
    aid: i64,
    csv_file_path: &str,
    delay_seconds: u64,
    sessdata: Option<&str>,
    progress_callback: F,
) -> Result<String, String>
where
    F: Fn(usize, Option<usize>) + Send + 'static,
{
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::Path;
    use tokio::time::{sleep, Duration};

    let csv_path = Path::new(csv_file_path);

    // 创建父目录
    if let Some(parent) = csv_path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 创建CSV文件并写入表头
    let mut csv_file = File::create(&csv_path)
        .map_err(|e| format!("创建CSV文件失败: {}", e))?;
    writeln!(csv_file, "rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent")
        .map_err(|e| format!("写入CSV表头失败: {}", e))?;

    let mut total_downloaded = 0;
    let mut pagination_str = r#"{"offset":""}"#.to_string();
    let mut estimated_total: Option<usize> = None;

    eprintln!("开始下载评论，aid: {}", aid);

    loop {
        eprintln!("正在获取评论，pagination_str: {}", pagination_str);

        // 获取评论
        let result = fetch_comments(aid, &pagination_str, sessdata).await;

        let (comments, next_offset, is_end) = match result {
            Ok(data) => data,
            Err(e) => {
                // If we have downloaded some comments, treat as incomplete
                if total_downloaded > 0 {
                    eprintln!("评论下载中断（已下载 {} 条）: {}", total_downloaded, e);
                    return Err(format!("评论API请求失败: {}", e));
                } else {
                    return Err(e);
                }
            }
        };

        if comments.is_empty() {
            eprintln!("没有更多评论，下载完成");
            break;
        }

        // 保存评论到CSV
        for comment in &comments {
            let content_escaped = comment.content.replace("\"", "\"\"").replace("\n", " ");
            let location_escaped = comment.location.replace("\"", "\"\"");
            let uname_escaped = comment.uname.replace("\"", "\"\"");

            let line = format!(
                "{},{},{},\"{}\",{},\"{}\",{},{},{},{},\"{}\",{}\n",
                comment.rpid,
                comment.oid,
                comment.mid,
                uname_escaped,
                comment.sex,
                content_escaped,
                comment.avatar,
                comment.ctime,
                comment.like,
                comment.current_level,
                location_escaped,
                comment.parent
            );

            csv_file.write_all(line.as_bytes())
                .map_err(|e| format!("写入CSV失败: {}", e))?;
        }

        csv_file.flush()
            .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

        total_downloaded += comments.len();
        eprintln!("已下载 {} 条评论", total_downloaded);

        // Call progress callback
        progress_callback(total_downloaded, estimated_total);

        // 检查是否结束
        if is_end {
            eprintln!("已到达最后一页，下载完成");
            break;
        }

        // 更新pagination_str
        if let Some(offset) = next_offset {
            pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
        } else {
            eprintln!("未找到next_offset，下载完成");
            break;
        }

        // 延迟，避免请求过快
        if delay_seconds > 0 {
            eprintln!("等待 {} 秒后继续...", delay_seconds);
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }

    let result_msg = format!("评论下载完成！共下载 {} 条评论，保存至: {}", total_downloaded, csv_path.display());
    eprintln!("{}", result_msg);

    Ok(result_msg)
}

/// 下载头像
async fn download_avatar(avatar_url: &str, save_path: &Path) -> Result<(), String> {
    let client = create_bilibili_client()?;

    let response = client
        .get(avatar_url)
        .send()
        .await
        .map_err(|e| format!("下载头像请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载头像失败: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取头像数据失败: {}", e))?;

    std::fs::write(save_path, bytes)
        .map_err(|e| format!("保存头像文件失败: {}", e))?;

    Ok(())
}
