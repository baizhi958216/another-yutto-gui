/// 视频质量代码到描述的映射
pub fn video_quality_to_description(quality: i32) -> String {
    match quality {
        127 => "8K 超高清".to_string(),
        126 => "杜比视界".to_string(),
        125 => "HDR 真彩".to_string(),
        120 => "4K 超清".to_string(),
        116 => "1080P 60帧".to_string(),
        112 => "1080P 高码率".to_string(),
        100 => "1080P 高清+".to_string(),
        80 => "1080P 高清".to_string(),
        74 => "720P 60帧".to_string(),
        64 => "720P 高清".to_string(),
        32 => "480P 清晰".to_string(),
        16 => "360P 流畅".to_string(),
        _ => format!("质量 {}", quality),
    }
}

/// 音频质量代码到描述的映射
pub fn audio_quality_to_description(quality: i32) -> String {
    match quality {
        30280 => "Hi-Res无损".to_string(),
        30255 => "杜比全景声".to_string(),
        30251 => "无损".to_string(),
        30250 => "杜比全景声".to_string(),
        30232 => "132K".to_string(),
        30216 => "64K".to_string(),
        _ => format!("音频质量 {}", quality),
    }
}

/// 判断视频质量的权限要求 (vip_only, login_required)
///
/// 返回 (vip_only, login_required)
pub fn get_video_quality_requirements(quality: i32) -> (bool, bool) {
    match quality {
        // 需要大会员的质量
        127 | 126 | 125 | 120 | 116 | 112 | 100 => (true, false),
        // 需要登录的质量
        80 | 74 | 64 | 32 => (false, true),
        // 不需要登录的质量
        16 => (false, false),
        // 默认需要登录
        _ => (false, true),
    }
}

/// 判断音频质量的权限要求 (vip_only, login_required)
///
/// 返回 (vip_only, login_required)
pub fn get_audio_quality_requirements(quality: i32) -> (bool, bool) {
    match quality {
        // 需要大会员的音频质量
        30251 | 30255 | 30250 => (true, false),
        // 需要登录的音频质量
        30280 | 30232 => (false, true),
        // 不需要登录的音频质量
        30216 => (false, false),
        // 默认需要登录
        _ => (false, true),
    }
}
