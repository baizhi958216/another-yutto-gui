use crate::models::video::Episode;
use quick_xml::events::Event;
use quick_xml::Reader;

/// 从文件名提取剧集序号
pub fn extract_episode_index(path: &std::path::Path, fallback_idx: usize) -> i64 {
    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // 尝试 S01E01 格式（番剧）
    let bangumi_re = regex::Regex::new(r"S\d+E(\d+)").unwrap();
    if let Some(caps) = bangumi_re.captures(filename) {
        if let Ok(idx) = caps[1].parse::<i64>() {
            return idx;
        }
    }

    // 尝试 P1 格式（分P视频）
    let part_re = regex::Regex::new(r"P(\d+)").unwrap();
    if let Some(caps) = part_re.captures(filename) {
        if let Ok(idx) = caps[1].parse::<i64>() {
            return idx;
        }
    }

    // 尝试"第X话"格式（中文番剧）
    let chinese_re = regex::Regex::new(r"第(\d+)话").unwrap();
    if let Some(caps) = chinese_re.captures(filename) {
        if let Ok(idx) = caps[1].parse::<i64>() {
            return idx;
        }
    }

    // 回退到文件顺序（从1开始）
    (fallback_idx + 1) as i64
}

/// 从 URL 提取剧集 ID
pub fn extract_episode_id(url: &str, fallback: i64) -> i64 {
    if url.is_empty() {
        eprintln!("[extract_episode_id] URL 为空，使用 fallback: {}", fallback);
        return fallback;
    }

    // 尝试从番剧 URL 提取 ep_id (支持多种格式)
    // 格式1: /ep123456 或 ep123456
    let ep_re = regex::Regex::new(r"ep(\d+)").unwrap();
    if let Some(caps) = ep_re.captures(url) {
        if let Ok(id) = caps[1].parse::<i64>() {
            eprintln!("[extract_episode_id] 从 URL 提取到 ep_id: {}", id);
            return id;
        }
    }

    // 尝试从视频 URL 提取分P序号
    let page_re = regex::Regex::new(r"[?&]p=(\d+)").unwrap();
    if let Some(caps) = page_re.captures(url) {
        if let Ok(id) = caps[1].parse::<i64>() {
            eprintln!("[extract_episode_id] 从 URL 提取到分P序号: {}", id);
            return id;
        }
    }

    eprintln!(
        "[extract_episode_id] 无法从 URL 提取 ID，使用 fallback: {}",
        fallback
    );
    // 回退到 index
    fallback
}

/// 解析单个剧集的 XML 信息
pub fn parse_episode_xml(xml: &str, index: i64) -> Result<Episode, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut title = String::new();
    let mut show_title = String::new();
    let mut duration = 0i64;
    let mut website = String::new();
    let mut source = String::new();
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "title" => title = text,
                    "show_title" => show_title = text,
                    "runtime" => duration = text.parse().unwrap_or(0),
                    "website" => website = text,
                    "source" => source = text,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML 解析错误: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    // 优先使用 show_title，如果为空则使用 title
    let final_title = if !show_title.is_empty() {
        show_title
    } else {
        title
    };

    // 从 URL 提取剧集 ID，优先使用 source，然后是 website
    let url_to_parse = if !source.is_empty() {
        eprintln!("[parse_episode_xml] 使用 source 字段提取 ep_id: {}", source);
        &source
    } else if !website.is_empty() {
        eprintln!(
            "[parse_episode_xml] 使用 website 字段提取 ep_id: {}",
            website
        );
        &website
    } else {
        eprintln!(
            "[parse_episode_xml] source 和 website 字段都为空，使用 fallback index: {}",
            index
        );
        ""
    };

    let id = extract_episode_id(url_to_parse, index);

    Ok(Episode {
        id,
        title: final_title,
        duration,
        index,
        available_qualities: None,
        available_audio_qualities: None,
    })
}
