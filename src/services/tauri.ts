import type { Comment, DownloadConfig, HistoryEntry, VideoInfo } from '@/types'

import { invoke, isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { open as openUrl } from '@tauri-apps/plugin-shell'

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
 * @param videoInfo 视频信息（可选）
 * @param videoInfo.title 视频标题
 * @param videoInfo.thumbnail 视频缩略图
 * @returns 任务 ID
 */
export async function startDownload(config: DownloadConfig, videoInfo?: { title: string, thumbnail: string }): Promise<string> {
  try {
    return await invoke<string>('start_download', { config, videoInfo })
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
 * 设置最大并发下载数
 * @param maxConcurrent 最大并发数
 */
export async function setMaxConcurrentDownloads(maxConcurrent: number): Promise<void> {
  try {
    await invoke('set_max_concurrent_downloads', { maxConcurrent })
  }
  catch (error) {
    console.error('Failed to set max concurrent downloads:', error)
    throw error
  }
}

/**
 * 选择文件夹
 * @returns 选择的文件夹路径，如果取消则返回 null
 */
export async function selectFolder(): Promise<string | null> {
  if (!isTauri())
    return null

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
  if (!isTauri())
    return null

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

/**
 * 添加历史记录
 * @param entry 历史记录条目
 */
export async function addToHistory(entry: HistoryEntry): Promise<void> {
  if (!isTauri())
    return

  try {
    await invoke('add_to_history', { entry })
  }
  catch (error) {
    console.error('Failed to add to history:', error)
    throw error
  }
}

/**
 * 获取历史记录
 * @param page 页码（可选）
 * @param pageSize 每页大小（可选）
 * @returns 历史记录列表
 */
export async function getHistory(page?: number, pageSize?: number): Promise<HistoryEntry[]> {
  if (!isTauri())
    return []

  try {
    return await invoke<HistoryEntry[]>('get_history', { page, pageSize })
  }
  catch (error) {
    console.error('Failed to get history:', error)
    throw error
  }
}

/**
 * 删除历史记录条目
 * @param entryId 条目 ID
 */
export async function deleteHistoryEntry(entryId: string): Promise<void> {
  if (!isTauri())
    return

  try {
    await invoke('delete_history_entry', { entryId })
  }
  catch (error) {
    console.error('Failed to delete history entry:', error)
    throw error
  }
}

/**
 * 清空所有历史记录
 */
export async function clearHistory(): Promise<void> {
  if (!isTauri())
    return

  try {
    await invoke('clear_history')
  }
  catch (error) {
    console.error('Failed to clear history:', error)
    throw error
  }
}

/**
 * 获取文件或目录大小
 * @param path 文件或目录路径
 * @returns 文件大小（字节）
 */
export async function getFileSize(path: string): Promise<number> {
  try {
    return await invoke<number>('get_file_size', { path })
  }
  catch (error) {
    console.error('Failed to get file size:', error)
    return 0 // Return 0 if unable to get file size
  }
}

/**
 * 查找目录中最新的文件
 * @param dirPath 目录路径
 * @param afterTimestamp 只查找在此时间戳之后修改的文件（毫秒）
 * @returns 最新文件的路径，如果没有找到则返回 null
 */
export async function findNewestFileInDir(
  dirPath: string,
  afterTimestamp?: number,
  extensions?: string[],
  nameHint?: string,
): Promise<string | null> {
  try {
    return await invoke<string | null>('find_newest_file_in_dir', {
      dirPath,
      afterTimestamp,
      extensions,
      nameHint,
    })
  }
  catch (error) {
    console.error('Failed to find newest file:', error)
    return null
  }
}

/**
 * 获取视频评论
 * @param aid 视频 AID
 * @param page 页码
 * @param pageSize 每页大小
 * @param sessdata 用户的 SESSDATA cookie
 * @returns 评论列表
 */
export async function getVideoComments(aid: number, page: number = 1, pageSize: number = 20, sessdata?: string): Promise<Comment[]> {
  try {
    return await invoke<Comment[]>('fetch_video_comments', {
      aid,
      page,
      pageSize,
      sessdata: sessdata || null,
    })
  }
  catch (error) {
    console.error('Failed to get video comments:', error)
    throw error
  }
}

/**
 * 下载视频所有评论到本地
 * @param aid 视频 AID
 * @param bvid 视频 BVID
 * @param savePath 保存路径
 * @param downloadAvatars 是否下载头像
 * @param delaySeconds 请求延迟秒数
 * @param sessdata 用户的 SESSDATA cookie
 * @returns 下载结果消息
 */
export async function downloadVideoComments(
  aid: number,
  bvid: string,
  savePath: string,
  downloadAvatars: boolean = true,
  delaySeconds: number = 3,
  sessdata?: string,
): Promise<string> {
  try {
    return await invoke<string>('download_video_comments', {
      aid,
      bvid,
      savePath,
      downloadAvatars,
      delaySeconds,
      sessdata: sessdata || null,
    })
  }
  catch (error) {
    console.error('Failed to download video comments:', error)
    throw error
  }
}

/**
 * 读取CSV文件内容
 * @param filePath CSV文件路径
 * @returns CSV文件内容
 */
export async function readCsvFile(filePath: string): Promise<string> {
  try {
    return await invoke<string>('read_csv_file', { filePath })
  }
  catch (error) {
    console.error('Failed to read CSV file:', error)
    throw error
  }
}

/**
 * 通过后端代理获取图片并返回 data URL
 * 用于绕过 WebView 跨平台差异导致的 403 防盗链问题
 */
export async function fetchImageDataUrl(url: string): Promise<string> {
  try {
    return await invoke<string>('fetch_image_data_url', { url })
  }
  catch (error) {
    console.error('Failed to fetch image data URL:', error)
    throw error
  }
}

/**
 * 下载评论到指定文件
 * @param aid 视频 AID
 * @param csvFilePath CSV文件路径
 * @param delaySeconds 请求延迟秒数
 * @param sessdata 用户的 SESSDATA cookie
 * @returns 下载结果消息
 */
export async function downloadCommentsToFile(
  aid: number,
  csvFilePath: string,
  delaySeconds: number = 3,
  sessdata?: string,
): Promise<string> {
  try {
    return await invoke<string>('download_comments_to_file', {
      aid,
      csvFilePath,
      delaySeconds,
      sessdata: sessdata || null,
    })
  }
  catch (error) {
    console.error('Failed to download comments to file:', error)
    throw error
  }
}

/**
 * 在系统默认浏览器中打开 URL
 * @param url 要打开的 URL
 */
export async function openInBrowser(url: string): Promise<void> {
  if (!isTauri()) {
    window.open(url, '_blank', 'noopener,noreferrer')
    return
  }

  try {
    await openUrl(url)
  }
  catch (error) {
    console.error('Failed to open URL in browser:', error)
    throw error
  }
}
