/**
 * API 服务层
 * 用于封装与后端的通信逻辑
 */

import type { DownloadConfig, DownloadProgress, VideoInfo } from '@/types'
import * as tauri from './tauri'

/**
 * API 服务类
 */
export class ApiService {
  /**
   * 获取视频信息
   */
  static async fetchVideoInfo(url: string, isVip?: boolean, sessdata?: string): Promise<VideoInfo> {
    return await tauri.getVideoInfo(url, isVip, sessdata)
  }

  /**
   * 创建下载任务
   */
  static async createDownloadTask(config: DownloadConfig, videoInfo?: { title: string, thumbnail: string }): Promise<string> {
    return await tauri.startDownload(config, videoInfo)
  }

  /**
   * 暂停下载任务
   */
  static async pauseTask(taskId: string): Promise<void> {
    await tauri.pauseDownload(taskId)
  }

  /**
   * 恢复下载任务
   */
  static async resumeTask(taskId: string): Promise<void> {
    await tauri.resumeDownload(taskId)
  }

  /**
   * 取消下载任务
   */
  static async cancelTask(taskId: string): Promise<void> {
    await tauri.cancelDownload(taskId)
  }

  /**
   * 选择下载目录
   */
  static async selectDownloadDirectory(): Promise<string | null> {
    return await tauri.selectFolder()
  }
}

/**
 * 下载进度监听器类型
 */
export type ProgressListener = (progress: DownloadProgress) => void

/**
 * 下载进度管理器
 */
export class ProgressManager {
  private listeners: Map<string, ProgressListener[]> = new Map()

  /**
   * 添加进度监听器
   */
  addListener(taskId: string, listener: ProgressListener): void {
    if (!this.listeners.has(taskId)) {
      this.listeners.set(taskId, [])
    }
    this.listeners.get(taskId)!.push(listener)
  }

  /**
   * 移除进度监听器
   */
  removeListener(taskId: string, listener: ProgressListener): void {
    const listeners = this.listeners.get(taskId)
    if (listeners) {
      const index = listeners.indexOf(listener)
      if (index > -1) {
        listeners.splice(index, 1)
      }
    }
  }

  /**
   * 触发进度更新
   */
  notifyProgress(progress: DownloadProgress): void {
    const listeners = this.listeners.get(progress.downloadId)
    if (listeners) {
      listeners.forEach(listener => listener(progress))
    }
  }

  /**
   * 清除任务的所有监听器
   */
  clearListeners(taskId: string): void {
    this.listeners.delete(taskId)
  }
}

// 导出单例实例
export const progressManager = new ProgressManager()
