import type { DownloadTask } from '@/types'
import { ApiService } from '@/services/api'
import { useDownloadStore } from '@/stores/download'
import { useHistoryStore } from '@/stores/history'
import { useQueueStore } from '@/stores/queue'
import { useToast } from './useToast'

/**
 * 下载功能组合式函数
 */
export function useDownload() {
  const downloadStore = useDownloadStore()
  const queueStore = useQueueStore()
  const historyStore = useHistoryStore()
  const { showToast } = useToast()

  /**
   * 提交下载任务
   */
  async function submitDownload() {
    const { currentConfig, videoInfo } = downloadStore

    if (!currentConfig) {
      showToast('error', '请先配置下载参数')
      return
    }

    if (!videoInfo) {
      showToast('error', '请先获取视频信息')
      return
    }

    try {
      // 创建下载任务
      const taskId = await ApiService.createDownloadTask(currentConfig, {
        title: videoInfo.title,
        thumbnail: videoInfo.thumbnail,
      })

      const task: DownloadTask = {
        id: taskId,
        config: currentConfig,
        status: 'pending',
        progress: 0,
        speed: '0 KB/s',
        eta: '--:--',
        videoInfo: {
          title: videoInfo.title,
          thumbnail: videoInfo.thumbnail,
        },
        totalSize: 0,
        savedFilePath: undefined,
        startTime: Date.now(),
      }

      // 添加到队列
      queueStore.addTask(task)

      // 显示成功提示
      showToast('success', '已添加到下载队列')

      // 不重置下载表单，保持页面内容
    }
    catch (error) {
      showToast('error', error instanceof Error ? error.message : '添加下载任务失败')
    }
  }

  /**
   * 暂停下载
   */
  async function pauseDownload(taskId: string) {
    try {
      await ApiService.pauseTask(taskId)
      queueStore.pauseTask(taskId)
      showToast('info', '已暂停下载')
    }
    catch (error) {
      showToast('error', '暂停下载失败')
    }
  }

  /**
   * 恢复下载
   */
  async function resumeDownload(taskId: string) {
    try {
      await ApiService.resumeTask(taskId)
      queueStore.resumeTask(taskId)
      showToast('info', '已恢复下载')
    }
    catch (error) {
      showToast('error', '恢复下载失败')
    }
  }

  /**
   * 取消下载
   */
  async function cancelDownload(taskId: string) {
    try {
      await ApiService.cancelTask(taskId)
      queueStore.removeTask(taskId)
      showToast('info', '已取消下载')
    }
    catch (error) {
      showToast('error', '取消下载失败')
    }
  }

  return {
    submitDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
  }
}
