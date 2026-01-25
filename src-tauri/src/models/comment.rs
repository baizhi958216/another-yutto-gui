// Comment models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub rpid: i64,
    pub oid: i64,
    pub mid: i64,
    pub uname: String,
    pub avatar: String,
    pub sex: String,
    pub content: String,
    pub ctime: i64,
    pub like: i64,
    pub current_level: i32,
    pub location: String,
    pub parent: i64,
    pub pictures: Vec<Picture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    pub img_src: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyItem {
    pub rpid: i64,
    pub oid: i64,
    pub mid: i64,
    pub parent: i64,
    pub ctime: i64,
    pub like: i64,
    pub member: Member,
    pub content: Content,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyControl {
    pub location: Option<String>,
    pub time_desc: Option<String>,
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
    pub next_offset: String,
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
