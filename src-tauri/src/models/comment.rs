// Comment models
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub rpid: i64,
    pub oid: i64,
    pub mid: i64,
    pub root: i64,
    pub uname: String,
    pub avatar: String,
    pub sex: String,
    pub content: String,
    pub ctime: i64,
    pub like: i64,
    pub reply_count: i64,
    pub current_level: i32,
    pub location: String,
    pub parent: i64,
    pub pictures: Vec<Picture>,
    pub emotes: Vec<Emote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    pub img_src: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyItem {
    pub rpid: i64,
    pub oid: i64,
    pub mid: i64,
    #[serde(default)]
    pub root: i64,
    pub parent: i64,
    #[serde(default)]
    pub count: i64,
    pub ctime: i64,
    pub like: i64,
    pub member: Member,
    pub content: Content,
    #[serde(default)]
    pub reply_control: ReplyControl,
    pub replies: Option<Vec<ReplyItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub mid: String,
    pub uname: String,
    pub sex: String,
    pub avatar: String,
    pub level_info: LevelInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub current_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub message: String,
    pub pictures: Option<Vec<Picture>>,
    #[serde(default)]
    pub emote: Option<HashMap<String, EmoteItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmoteItem {
    pub text: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyControl {
    pub location: Option<String>,
    pub time_desc: Option<String>,
}

impl Default for ReplyControl {
    fn default() -> Self {
        Self {
            location: None,
            time_desc: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub is_begin: bool,
    pub prev: i64,
    pub next: i64,
    pub is_end: bool,
    pub all_count: i64,
    pub pagination_reply: Option<PaginationReply>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationReply {
    #[serde(default)]
    pub next_offset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<CommentData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentData {
    pub cursor: Cursor,
    pub replies: Option<Vec<ReplyItem>>,
    pub top_replies: Option<Vec<ReplyItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<ReplyData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyData {
    pub page: ReplyPage,
    pub replies: Option<Vec<ReplyItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyPage {
    pub num: i32,
    pub size: i32,
    pub count: i64,
}
