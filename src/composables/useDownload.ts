import type { DownloadTask } from '@/types'
import { ApiService } from '@/services/api'
import { useDownloadStore } from '@/stores/download'
import { useQueueStore } from '@/stores/queue'
import { useToast } from './useToast'

/**
 * 下载功能组合式函数
 */
export function useDownload() {
  const downloadStore = useDownloadStore()
  const queueStore = useQueueStore()
  const { showToast } = useToast()

  /**
   * 提交下载任务
   */
  async function submitDownload() {
    const { currentConfig, videoInfo, selectedEpisodes, episodeQualities } = downloadStore

    if (!currentConfig) {
      showToast('error', '请先配置下载参数')
      return
    }

    if (!videoInfo) {
      showToast('error', '请先获取视频信息')
      return
    }

    try {
      // 检查是否有多剧集且每个剧集有不同的质量设置
      const hasEpisodes = videoInfo.episodes && videoInfo.episodes.length > 1
      const hasEpisodeQualities = Object.keys(episodeQualities).length > 0

      if (hasEpisodes && hasEpisodeQualities) {
        // 为每个选中的剧集创建单独的下载任务
        const selectedEpisodesList = Array.from(selectedEpisodes).sort((a, b) => a - b)
        let successCount = 0

        for (const episodeIndex of selectedEpisodesList) {
          const episode = videoInfo.episodes!.find(ep => ep.index === episodeIndex)
          if (!episode)
            continue

          const episodeQuality = episodeQualities[episodeIndex]
          if (!episodeQuality)
            continue

          // 为每个剧集创建单独的配置
          const episodeConfig = {
            ...currentConfig,
            videoQuality: episodeQuality.videoQuality,
            audioQuality: episodeQuality.audioQuality,
            episodes: String(episodeIndex), // 只下载这一集
          }

          try {
            const taskId = await ApiService.createDownloadTask(episodeConfig, {
              title: `${videoInfo.title} - 第${episode.index}话`,
              thumbnail: videoInfo.thumbnail,
            })

            const task: DownloadTask = {
              id: taskId,
              config: episodeConfig,
              status: 'pending',
              progress: 0,
              speed: '0 KB/s',
              eta: '--:--',
              videoInfo: {
                title: `${videoInfo.title} - 第${episode.index}话`,
                thumbnail: videoInfo.thumbnail,
              },
              totalSize: 0,
              downloadedBytes: 0,
              totalBytes: 0,
              speedBytesPerSec: 0,
              savedFilePath: undefined,
              commentFilePath: undefined,
              commentDownloadProgress: undefined,
              isDownloadingComments: false,
              startTime: Date.now(),
            }

            queueStore.addTask(task)
            successCount++
          }
          catch (error) {
            console.error(`添加第${episode.index}话下载任务失败:`, error)
          }
        }

        if (successCount > 0) {
          showToast('success', `已添加 ${successCount} 个下载任务到队列`)
        }
        else {
          showToast('error', '添加下载任务失败')
        }
      }
      else {
        // 单个任务或批量下载（使用全局质量设置）
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
          downloadedBytes: 0,
          totalBytes: 0,
          speedBytesPerSec: 0,
          savedFilePath: undefined,
          commentFilePath: undefined,
          commentDownloadProgress: undefined,
          isDownloadingComments: false,
          startTime: Date.now(),
        }

        queueStore.addTask(task)
        showToast('success', '已添加到下载队列')
      }
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
