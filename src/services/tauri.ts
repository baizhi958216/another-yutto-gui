import type { DownloadConfig, VideoInfo } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

/**
 * 获取视频信息
 * @param url 视频 URL
 * @returns 视频信息
 */
export async function getVideoInfo(url: string): Promise<VideoInfo> {
  try {
    return await invoke<VideoInfo>('fetch_video_info', { url })
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
