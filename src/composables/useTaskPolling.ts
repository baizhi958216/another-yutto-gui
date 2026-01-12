import { onMounted, onUnmounted } from 'vue'
import { useQueueStore } from '@/stores/queue'
import { invoke } from '@tauri-apps/api/core'
import type { DownloadTask } from '@/types'

/**
 * 任务轮询组合式函数
 * 定期从后端获取任务状态并更新前端
 */
export function useTaskPolling(intervalMs: number = 1000) {
  const queueStore = useQueueStore()
  let pollingTimer: number | null = null

  async function pollTasks() {
    try {
      // 获取后端的活动下载任务
      const tasksJson = await invoke<string>('get_active_downloads')
      const backendTasks: DownloadTask[] = JSON.parse(tasksJson)

      // 调试日志
      if (backendTasks.length > 0) {
        console.log('[轮询] 后端任务数量:', backendTasks.length)
        backendTasks.forEach(task => {
          console.log(`[轮询] 任务 ${task.id.substring(0, 8)}: ${task.progress.toFixed(1)}% - ${task.speed}`)
        })
      }

      // 更新前端任务状态
      backendTasks.forEach((backendTask) => {
        const frontendTask = queueStore.tasks.find(t => t.id === backendTask.id)
        if (frontendTask) {
          // 更新任务状态和进度
          queueStore.updateTask(backendTask.id, {
            status: backendTask.status,
            progress: backendTask.progress,
            speed: backendTask.speed,
            eta: backendTask.eta,
            error: backendTask.error,
          })
        }
      })
    }
    catch (error) {
      console.error('轮询任务状态失败:', error)
    }
  }

  function startPolling() {
    if (pollingTimer !== null) {
      return
    }

    // 立即执行一次
    pollTasks()

    // 设置定时轮询
    pollingTimer = window.setInterval(pollTasks, intervalMs)
  }

  function stopPolling() {
    if (pollingTimer !== null) {
      clearInterval(pollingTimer)
      pollingTimer = null
    }
  }

  // 组件挂载时开始轮询
  onMounted(() => {
    startPolling()
  })

  // 组件卸载时停止轮询
  onUnmounted(() => {
    stopPolling()
  })

  return {
    startPolling,
    stopPolling,
    pollTasks,
  }
}
