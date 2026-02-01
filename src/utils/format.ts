import { formatAudioQuality, formatVideoQuality } from './quality'

/**
 * 格式化字节数为可读格式 (使用二进制单位 KiB/MiB/GiB，与 Rust 后端保持一致)
 * @param bytes 字节数
 * @returns 格式化后的字符串，如 "75.70 MiB"
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0)
    return '0 B'

  const k = 1024
  const sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB']
  const i = Math.floor(Math.log(Math.abs(bytes)) / Math.log(k))

  return `${(bytes / k ** i).toFixed(2)} ${sizes[i]}`
}

/**
 * 格式化文件大小 (使用十进制单位 KB/MB/GB，保留用于兼容性)
 * @param bytes 字节数
 * @returns 格式化后的文件大小字符串
 * @deprecated 建议使用 formatBytes 以与后端保持一致
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0)
    return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / k ** i).toFixed(2)} ${sizes[i]}`
}

/**
 * 格式化下载速度
 * @param bytesPerSec 每秒字节数
 * @returns 格式化后的速度字符串，如 "5.20 MiB/s"
 */
export function formatSpeed(bytesPerSec: number): string {
  if (!isFinite(bytesPerSec) || bytesPerSec < 0)
    return '0 B/s'

  return `${formatBytes(bytesPerSec)}/s`
}

/**
 * 格式化 ETA (预计剩余时间)
 * @param seconds 秒数
 * @returns 格式化后的时间字符串，如 "02:35" 或 "1:23:45"
 */
export function formatEta(seconds: number): string {
  if (!isFinite(seconds) || seconds < 0)
    return '--:--'

  return formatDuration(Math.round(seconds))
}

/**
 * 格式化下载进度信息
 * @param downloaded 已下载字节数
 * @param total 总字节数
 * @returns 格式化后的进度字符串，如 "75.70 MiB / 110.21 MiB"
 */
export function formatDownloadProgress(downloaded: number, total: number): string {
  return `${formatBytes(downloaded)} / ${formatBytes(total)}`
}


function resolveLocale(): string {
  if (typeof document !== 'undefined') {
    const lang = document.documentElement.lang
    if (lang) {
      return lang
    }
  }
  return 'zh-CN'
}

function isEnglishLocale(locale: string): boolean {
  return locale.toLowerCase().startsWith('en')
}

/**
 * 格式化时长
 * @param seconds 秒数
 * @returns 格式化后的时长字符串 (HH:MM:SS 或 MM:SS)
 */
export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0)
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

/**
 * 格式化日期
 * @param timestamp 时间戳（毫秒）
 * @returns 格式化后的日期字符串
 */
export function formatDate(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleDateString(resolveLocale(), {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 格式化相对时间
 * @param timestamp 时间戳（毫秒）
 * @returns 相对时间字符串（如：刚刚、5分钟前、2小时前）
 */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  const locale = resolveLocale()
  const isEnglish = isEnglishLocale(locale)

  if (seconds < 60)
    return isEnglish ? 'just now' : '刚刚'
  if (minutes < 60)
    return isEnglish ? `${minutes} minutes ago` : `${minutes}分钟前`
  if (hours < 24)
    return isEnglish ? `${hours} hours ago` : `${hours}小时前`
  if (days < 7)
    return isEnglish ? `${days} days ago` : `${days}天前`
  return formatDate(timestamp)
}

/**
 * 生成下载配置标签
 * @param config 下载配置对象
 * @param config.videoOnly 仅视频
 * @param config.audioOnly 仅音频
 * @param config.videoQuality  视频质量
 * @param config.audioQuality  音频质量
 * @param config.withDanmaku 是否有弹幕
 * @param config.withSubtitle  是否有字幕
 * @param config.withCover  是否有封面
 * @param config.withComments  是否有评论
 * @param config.commentFilePath 字幕文件地址
 * @returns 标签数组
 */
export function generateDownloadTags(config: {
  videoOnly?: boolean
  audioOnly?: boolean
  videoQuality?: number
  audioQuality?: number
  withDanmaku?: boolean
  withSubtitle?: boolean
  withCover?: boolean
  withComments?: boolean
  commentFilePath?: string
}): string[] {
  const tags: string[] = []

  // 导入质量格式化函数

  if (config.videoOnly) {
    tags.push('仅视频')
  }
  else if (config.audioOnly) {
    tags.push('仅音频')
  }

  // 添加质量信息
  if (!config.audioOnly && config.videoQuality !== undefined) {
    tags.push(formatVideoQuality(config.videoQuality))
  }
  if (!config.videoOnly && config.audioQuality !== undefined) {
    tags.push(formatAudioQuality(config.audioQuality))
  }

  // 添加其他选项
  if (config.withDanmaku) {
    tags.push('弹幕')
  }
  if (config.withSubtitle) {
    tags.push('字幕')
  }
  if (config.withCover) {
    tags.push('封面')
  }
  if (config.withComments || config.commentFilePath) {
    tags.push('评论')
  }

  return tags
}
