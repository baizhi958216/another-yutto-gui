mod comments;
mod quality;
mod user_info;
mod video_info;
mod wbi_auth;

use crate::models::comment::Comment;
use crate::models::video::{Owner, VideoInfo};

pub struct BilibiliApi;

impl BilibiliApi {
    /// 从B站API获取视频信息
    pub async fn fetch_video_info_from_html(
        url: &str,
        sessdata: Option<&str>,
        is_vip: bool,
    ) -> Result<VideoInfo, String> {
        video_info::fetch_video_info_from_html(url, sessdata, is_vip).await
    }

    /// 获取视频评论（使用WBI签名API）
    pub async fn fetch_comments(
        aid: i64,
        pagination_str: &str,
        sessdata: Option<&str>,
    ) -> Result<(Vec<Comment>, Option<String>, bool), String> {
        comments::fetch_comments(aid, pagination_str, sessdata).await
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
        comments::download_all_comments(
            aid,
            bvid,
            save_path,
            download_avatars,
            delay_seconds,
            sessdata,
        )
        .await
    }

    /// 下载所有评论到指定文件路径
    pub async fn download_comments_to_file(
        aid: i64,
        csv_file_path: &str,
        delay_seconds: u64,
        sessdata: Option<&str>,
    ) -> Result<String, String> {
        comments::download_comments_to_file(aid, csv_file_path, delay_seconds, sessdata).await
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
        comments::download_comments_to_file_with_progress(
            aid,
            csv_file_path,
            delay_seconds,
            sessdata,
            progress_callback,
        )
        .await
    }

    /// 获取收藏夹用户信息
    pub async fn fetch_favorite_owner_info(
        fid: i64,
        sessdata: Option<&str>,
    ) -> Result<Owner, String> {
        user_info::fetch_favorite_owner_info(fid, sessdata).await
    }

    /// 获取用户详细信息
    pub async fn fetch_user_info(uid: i64, sessdata: Option<&str>) -> Result<Owner, String> {
        user_info::fetch_user_info(uid, sessdata).await
    }
}
