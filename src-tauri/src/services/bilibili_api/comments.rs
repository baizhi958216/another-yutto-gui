use crate::models::comment::{Comment, CommentResponse, ReplyItem, ReplyResponse};
use crate::utils::http_client::create_bilibili_client;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

const REPLY_PAGE_SIZE: i64 = 20;

fn build_query(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn map_reply_item_to_comment(reply: &ReplyItem) -> Comment {
    Comment {
        rpid: reply.rpid,
        oid: reply.oid,
        mid: reply.mid,
        root: reply.root,
        uname: reply.member.uname.clone(),
        avatar: reply.member.avatar.clone(),
        sex: reply.member.sex.clone(),
        content: reply.content.message.clone(),
        ctime: reply.ctime,
        like: reply.like,
        reply_count: reply.count,
        current_level: reply.member.level_info.current_level,
        location: reply.reply_control.location.clone().unwrap_or_default(),
        parent: reply.parent,
        pictures: reply.content.pictures.clone().unwrap_or_default(),
        replies: None,
    }
}

fn write_csv_header<W: Write>(csv_file: &mut W) -> Result<(), String> {
    writeln!(
        csv_file,
        "rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent,root,reply_count"
    )
    .map_err(|e| format!("写入CSV表头失败: {}", e))
}

fn write_comment_to_csv<W: Write>(csv_file: &mut W, comment: &Comment) -> Result<(), String> {
    let content_escaped = comment.content.replace('"', "\"\"").replace('\n', " ");
    let location_escaped = comment.location.replace('"', "\"\"");
    let uname_escaped = comment.uname.replace('"', "\"\"");

    let line = format!(
        "{},{},{},\"{}\",{},\"{}\",{},{},{},{},\"{}\",{},{},{}\n",
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
        comment.parent,
        comment.root,
        comment.reply_count
    );

    csv_file
        .write_all(line.as_bytes())
        .map_err(|e| format!("写入CSV失败: {}", e))
}

fn should_fetch_replies(comment: &Comment) -> bool {
    comment.parent == 0 && comment.reply_count > 0
}

/// 获取视频评论（使用WBI签名API）
pub async fn fetch_comments(
    aid: i64,
    pagination_str: &str,
    sessdata: Option<&str>,
) -> Result<(Vec<Comment>, Option<String>, bool), String> {
    let mut params = vec![
        ("oid".to_string(), aid.to_string()),
        ("type".to_string(), "1".to_string()),
        ("pagination_str".to_string(), pagination_str.to_string()),
        ("plat".to_string(), "1".to_string()),
        ("web_location".to_string(), "1315875".to_string()),
    ];

    super::wbi_auth::sign_wbi_params(&mut params).await?;

    let api_url = format!(
        "https://api.bilibili.com/x/v2/reply/wbi/main?{}",
        build_query(&params)
    );

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

    let json: CommentResponse =
        serde_json::from_str(&json_text).map_err(|e| format!("解析评论JSON失败: {}", e))?;

    if json.code != 0 {
        return Err(format!(
            "评论API返回错误: {} (code: {})",
            json.message, json.code
        ));
    }

    let data = json.data.ok_or("评论API响应中未找到data字段")?;

    let next_offset = data
        .cursor
        .pagination_reply
        .as_ref()
        .and_then(|p| p.next_offset.clone());

    let is_end = data.cursor.is_end;

    let comments: Vec<Comment> = data
        .replies
        .unwrap_or_default()
        .iter()
        .map(map_reply_item_to_comment)
        .collect();

    eprintln!("成功获取 {} 条评论", comments.len());

    Ok((comments, next_offset, is_end))
}

/// 获取某条主评论下的二级回复
async fn fetch_comment_replies(
    aid: i64,
    root: i64,
    page_number: i64,
    page_size: i64,
    sessdata: Option<&str>,
) -> Result<(Vec<Comment>, i64), String> {
    let params = vec![
        ("oid".to_string(), aid.to_string()),
        ("type".to_string(), "1".to_string()),
        ("root".to_string(), root.to_string()),
        ("ps".to_string(), page_size.to_string()),
        ("pn".to_string(), page_number.to_string()),
        ("web_location".to_string(), "333.788".to_string()),
    ];

    let api_url = format!(
        "https://api.bilibili.com/x/v2/reply/reply?{}",
        build_query(&params)
    );

    eprintln!("调用二级回复 API: {}", api_url);

    let client = create_bilibili_client()?;
    let mut request = client.get(&api_url);

    if let Some(sessdata) = sessdata {
        request = request.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("回复API请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("回复API请求失败: {}", response.status()));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("读取回复响应失败: {}", e))?;

    let json: ReplyResponse =
        serde_json::from_str(&json_text).map_err(|e| format!("解析回复JSON失败: {}", e))?;

    if json.code != 0 {
        return Err(format!(
            "回复API返回错误: {} (code: {})",
            json.message, json.code
        ));
    }

    let data = json.data.ok_or("回复API响应中未找到data字段")?;
    let total_count = data.page.count;

    let comments = data
        .replies
        .unwrap_or_default()
        .iter()
        .map(map_reply_item_to_comment)
        .collect();

    Ok((comments, total_count))
}

async fn write_replies_for_root<W: Write>(
    aid: i64,
    root: i64,
    csv_file: &mut W,
    downloaded_rpids: &mut HashSet<i64>,
    total_downloaded: &mut usize,
    delay_seconds: u64,
    sessdata: Option<&str>,
) -> Result<(), String> {
    use tokio::time::{sleep, Duration};

    let mut page_number = 1;

    loop {
        let (replies, total_count) =
            fetch_comment_replies(aid, root, page_number, REPLY_PAGE_SIZE, sessdata).await?;

        if replies.is_empty() {
            break;
        }

        for reply in replies {
            if downloaded_rpids.insert(reply.rpid) {
                write_comment_to_csv(csv_file, &reply)?;
                *total_downloaded += 1;
            }
        }

        let should_stop = if total_count > 0 {
            page_number * REPLY_PAGE_SIZE >= total_count
        } else {
            false
        };

        if should_stop {
            break;
        }

        page_number += 1;

        if delay_seconds > 0 {
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }

    Ok(())
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
    use tokio::time::{sleep, Duration};

    let video_dir = Path::new(save_path).join(bvid);
    create_dir_all(&video_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let csv_path = video_dir.join(format!("{}.csv", bvid));
    let avatars_dir = video_dir.join("avatars");

    if download_avatars {
        create_dir_all(&avatars_dir).map_err(|e| format!("创建头像目录失败: {}", e))?;
    }

    let file_exists = csv_path.exists();
    let mut csv_file = if file_exists {
        OpenOptions::new()
            .append(true)
            .open(&csv_path)
            .map_err(|e| format!("打开CSV文件失败: {}", e))?
    } else {
        let mut file = File::create(&csv_path).map_err(|e| format!("创建CSV文件失败: {}", e))?;
        write_csv_header(&mut file)?;
        file
    };

    let mut total_downloaded = 0;
    let mut pagination_str = r#"{"offset":""}"#.to_string();
    let mut downloaded_rpids = HashSet::new();

    eprintln!("开始下载评论，视频: {}, aid: {}", bvid, aid);

    loop {
        eprintln!("正在获取评论，pagination_str: {}", pagination_str);

        let (comments, next_offset, is_end) =
            fetch_comments(aid, &pagination_str, sessdata).await?;

        if comments.is_empty() {
            eprintln!("没有更多评论，下载完成");
            break;
        }

        for comment in &comments {
            if downloaded_rpids.insert(comment.rpid) {
                write_comment_to_csv(&mut csv_file, comment)?;
                total_downloaded += 1;

                if download_avatars && !comment.avatar.is_empty() {
                    let avatar_filename =
                        format!("{}_{}.jpg", comment.mid, comment.uname.replace('/', "_"));
                    let avatar_path = avatars_dir.join(&avatar_filename);

                    if !avatar_path.exists() {
                        if let Err(e) = download_avatar(&comment.avatar, &avatar_path).await {
                            eprintln!("下载头像失败 {}: {}", comment.uname, e);
                        }
                    }
                }
            }

            if should_fetch_replies(comment) {
                write_replies_for_root(
                    aid,
                    comment.rpid,
                    &mut csv_file,
                    &mut downloaded_rpids,
                    &mut total_downloaded,
                    delay_seconds,
                    sessdata,
                )
                .await?;
            }
        }

        csv_file
            .flush()
            .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

        eprintln!("已下载 {} 条评论/回复", total_downloaded);

        if is_end {
            eprintln!("已到达最后一页，下载完成");
            break;
        }

        if let Some(offset) = next_offset {
            pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
        } else {
            eprintln!("未找到next_offset，下载完成");
            break;
        }

        if delay_seconds > 0 {
            eprintln!("等待 {} 秒后继续...", delay_seconds);
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }

    let result_msg = format!(
        "评论下载完成！共下载 {} 条评论/回复，保存至: {}",
        total_downloaded,
        csv_path.display()
    );
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
    download_comments_to_file_with_progress(aid, csv_file_path, delay_seconds, sessdata, |_, _| {})
        .await
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
    use tokio::time::{sleep, Duration};

    let csv_path = Path::new(csv_file_path);

    if let Some(parent) = csv_path.parent() {
        create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let mut csv_file = File::create(csv_path).map_err(|e| format!("创建CSV文件失败: {}", e))?;
    write_csv_header(&mut csv_file)?;

    let mut total_downloaded = 0;
    let mut pagination_str = r#"{"offset":""}"#.to_string();
    let estimated_total: Option<usize> = None;
    let mut downloaded_rpids = HashSet::new();

    eprintln!("开始下载评论，aid: {}", aid);

    loop {
        eprintln!("正在获取评论，pagination_str: {}", pagination_str);

        let result = fetch_comments(aid, &pagination_str, sessdata).await;

        let (comments, next_offset, is_end) = match result {
            Ok(data) => data,
            Err(e) => {
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

        for comment in &comments {
            if downloaded_rpids.insert(comment.rpid) {
                write_comment_to_csv(&mut csv_file, comment)?;
                total_downloaded += 1;
                progress_callback(total_downloaded, estimated_total);
            }

            if should_fetch_replies(comment) {
                let reply_result = write_replies_for_root(
                    aid,
                    comment.rpid,
                    &mut csv_file,
                    &mut downloaded_rpids,
                    &mut total_downloaded,
                    delay_seconds,
                    sessdata,
                )
                .await;

                if let Err(e) = reply_result {
                    if total_downloaded > 0 {
                        eprintln!("回复下载中断（已下载 {} 条）: {}", total_downloaded, e);
                        return Err(format!("回复API请求失败: {}", e));
                    }
                    return Err(e);
                }

                progress_callback(total_downloaded, estimated_total);
            }
        }

        csv_file
            .flush()
            .map_err(|e| format!("刷新CSV文件失败: {}", e))?;

        eprintln!("已下载 {} 条评论/回复", total_downloaded);
        progress_callback(total_downloaded, estimated_total);

        if is_end {
            eprintln!("已到达最后一页，下载完成");
            break;
        }

        if let Some(offset) = next_offset {
            pagination_str = format!(r#"{{"offset":"{}"}}"#, offset);
        } else {
            eprintln!("未找到next_offset，下载完成");
            break;
        }

        if delay_seconds > 0 {
            eprintln!("等待 {} 秒后继续...", delay_seconds);
            sleep(Duration::from_secs(delay_seconds)).await;
        }
    }

    let result_msg = format!(
        "评论下载完成！共下载 {} 条评论/回复，保存至: {}",
        total_downloaded,
        csv_path.display()
    );
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

    std::fs::write(save_path, bytes).map_err(|e| format!("保存头像文件失败: {}", e))?;

    Ok(())
}
