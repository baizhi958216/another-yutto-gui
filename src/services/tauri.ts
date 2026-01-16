import type { DownloadConfig, VideoInfo } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

/**
 * 获取视频信息
 * @param url 视频 URL
 * @param isVip 是否为 VIP 用户
 * @param sessdata 用户的 SESSDATA cookie
 * @returns 视频信息
 */
export async function getVideoInfo(url: string, isVip?: boolean, sessdata?: string): Promise<VideoInfo> {
  try {
    // 确保 isVip 是明确的 boolean 值，而不是 undefined
    const isVipValue = isVip === true
    console.log('[Tauri] Calling fetch_video_info with:', { url, isVip: isVipValue, sessdata: sessdata ? 'present' : 'null' })
    return await invoke<VideoInfo>('fetch_video_info', {
      url,
      isVip: isVipValue, // 使用驼峰命名，Tauri 会自动转换为 Rust 的 snake_case
      sessdata: sessdata || null,
    })
  }
  catch (error) {
    console.error('Failed to get video info:', error)
    throw error
  }
}

/**
 * 开始下载
 * @param config 下载配置
 * @returns 任务 ID
 */
export async function startDownload(config: DownloadConfig): Promise<string> {
  try {
    return await invoke<string>('start_download', { config })
  }
  catch (error) {
    console.error('Failed to start download:', error)
    throw error
  }
}

/**
 * 暂停下载
 * @param taskId 任务 ID
 */
export async function pauseDownload(taskId: string): Promise<void> {
  try {
    await invoke('pause_download', { downloadId: taskId })
  }
  catch (error) {
    console.error('Failed to pause download:', error)
    throw error
  }
}

/**
 * 恢复下载
 * @param taskId 任务 ID
 */
export async function resumeDownload(taskId: string): Promise<void> {
  try {
    await invoke('resume_download', { downloadId: taskId })
  }
  catch (error) {
    console.error('Failed to resume download:', error)
    throw error
  }
}

/**
 * 取消下载
 * @param taskId 任务 ID
 */
export async function cancelDownload(taskId: string): Promise<void> {
  try {
    await invoke('cancel_download', { downloadId: taskId })
  }
  catch (error) {
    console.error('Failed to cancel download:', error)
    throw error
  }
}

/**
 * 选择文件夹
 * @returns 选择的文件夹路径，如果取消则返回 null
 */
export async function selectFolder(): Promise<string | null> {
  try {
    const result = await open({
      directory: true,
      multiple: false,
    })
    return result as string | null
  }
  catch (error) {
    console.error('Failed to select folder:', error)
    return null
  }
}

/**
 * 选择文件
 * @returns 选择的文件路径，如果取消则返回 null
 */
export async function selectFile(): Promise<string | null> {
  try {
    const result = await open({
      directory: false,
      multiple: false,
    })
    return result as string | null
  }
  catch (error) {
    console.error('Failed to select file:', error)
    return null
  }
}

/**
 * 打开 Bilibili 登录窗口
 */
export async function openBilibiliLogin(): Promise<void> {
  try {
    await invoke('open_login_window')
  }
  catch (error) {
    console.error('Failed to open login window:', error)
    throw error
  }
}
