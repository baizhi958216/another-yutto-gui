import type { DownloadTask } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted } from 'vue'
import { useQueueStore } from '@/stores/queue'

/**
 * 任务轮询和实时事件监听组合式函数
 * 使用 Tauri 事件进行实时更新，轮询作为备份机制
 */
export function useTaskPolling(intervalMs: number = 5000) {
  const queueStore = useQueueStore()
  let pollingTimer: number | null = null
  let unlistenProgress: UnlistenFn | null = null

  async function pollTasks() {
    try {
      // 获取后端的活动下载任务
      const tasksJson = await invoke<string>('get_active_downloads')
      const backendTasks: DownloadTask[] = JSON.parse(tasksJson)

      // 调试日志
      if (backendTasks.length > 0) {
        console.log('[轮询] 后端任务数量:', backendTasks.length)
      }

      // 更新前端任务状态
      backendTasks.forEach((backendTask) => {
        const frontendTask = queueStore.tasks.find(t => t.id === backendTask.id)
        if (frontendTask) {
          // 更新任务状态和进度（包括新字段）
          queueStore.updateTask(backendTask.id, {
            status: backendTask.status,
            progress: backendTask.progress,
            speed: backendTask.speed,
            eta: backendTask.eta,
            error: backendTask.error,
            warning: backendTask.warning,
            totalSize: backendTask.totalSize,
            downloadedBytes: backendTask.downloadedBytes,
            totalBytes: backendTask.totalBytes,
            speedBytesPerSec: backendTask.speedBytesPerSec,
            etaSeconds: backendTask.etaSeconds,
            filesCount: backendTask.filesCount,
            savedFilePath: backendTask.savedFilePath,
            commentFilePath: backendTask.commentFilePath,
            commentDownloadProgress: backendTask.commentDownloadProgress,
            isDownloadingComments: backendTask.isDownloadingComments,
            startTime: backendTask.startTime,
          })
        }
      })
    }
    catch (error) {
      console.error('轮询任务状态失败:', error)
    }
  }

  async function setupRealtimeListener() {
    try {
      // 监听来自 Rust 后端的实时进度事件
      unlistenProgress = await listen<DownloadTask>('download-progress', (event) => {
        const task = event.payload
        console.log(`[实时更新] 任务 ${task.id.substring(0, 8)}: ${task.progress.toFixed(1)}% - ${task.speed}`)

        // 更新任务状态
        const frontendTask = queueStore.tasks.find(t => t.id === task.id)
        if (frontendTask) {
          queueStore.updateTask(task.id, task)
        }
      })

      console.log('[实时监听] 已启动下载进度事件监听')
    }
    catch (error) {
      console.error('设置实时监听失败:', error)
    }
  }

  function startPolling() {
    if (pollingTimer !== null) {
      return
    }

    // 立即执行一次
    pollTasks()

    // 设置定时轮询（降低频率，因为有实时事件）
    pollingTimer = window.setInterval(pollTasks, intervalMs)
  }

  function stopPolling() {
    if (pollingTimer !== null) {
      clearInterval(pollingTimer)
      pollingTimer = null
    }

    // 清理事件监听器
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
  }

  // 组件挂载时开始轮询和实时监听
  onMounted(() => {
    setupRealtimeListener()
    startPolling()
  })

  // 组件卸载时停止轮询和监听
  onUnmounted(() => {
    stopPolling()
  })

  return {
    startPolling,
    stopPolling,
    pollTasks,
  }
}
