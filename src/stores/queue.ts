import type { DownloadProgress, DownloadTask } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { generateId } from '@/utils/helpers'
import { useHistoryStore } from './history'

export const useQueueStore = defineStore('queue', () => {
  // State
  const tasks = ref<DownloadTask[]>([])
  const activeTaskId = ref<string | null>(null)

  // Getters
  const pendingTasks = computed(() =>
    tasks.value.filter(task => task.status === 'pending'),
  )

  const downloadingTasks = computed(() =>
    tasks.value.filter(task => task.status === 'downloading'),
  )

  const completedTasks = computed(() =>
    tasks.value.filter(task => task.status === 'completed'),
  )

  const errorTasks = computed(() =>
    tasks.value.filter(task => task.status === 'error'),
  )

  const pausedTasks = computed(() =>
    tasks.value.filter(task => task.status === 'paused'),
  )

  const activeTask = computed(() =>
    tasks.value.find(task => task.id === activeTaskId.value),
  )

  // Actions
  function addTask(task: DownloadTask) {
    tasks.value.push(task)
  }

  function removeTask(id: string) {
    const index = tasks.value.findIndex(task => task.id === id)
    if (index > -1) {
      tasks.value.splice(index, 1)
    }
    if (activeTaskId.value === id) {
      activeTaskId.value = null
    }
  }

  async function updateTask(id: string, updates: Partial<DownloadTask>) {
    const index = tasks.value.findIndex(t => t.id === id)
    if (index !== -1) {
      // 创建新对象以确保触发响应式更新
      const oldTask = tasks.value[index]
      const newTask = { ...oldTask, ...updates }

      // Check if task just completed
      const justCompleted = oldTask.status !== 'completed' && newTask.status === 'completed'

      // 强制创建新数组引用以触发 Vue 响应式
      tasks.value = [
        ...tasks.value.slice(0, index),
        newTask,
        ...tasks.value.slice(index + 1),
      ]

      console.log(`[Queue Store] 更新任务 ${id.substring(0, 8)}: progress=${updates.progress?.toFixed(1)}%, speed=${updates.speed}`)

      // If task just completed, save to history
      if (justCompleted && newTask.videoInfo) {
        try {
          const historyStore = useHistoryStore()

          // Get actual file size and path
          let fileSize = newTask.totalSize || 0 // Use parsed size from yutto as default
          let actualFilePath = newTask.config.downloadPath

          const normalizedDownloadPath = newTask.config.downloadPath.replace(/[\\/]+$/, '')
          const normalizedSavedPath = newTask.savedFilePath?.replace(/[\\/]+$/, '')
          const savedFileName = normalizedSavedPath?.split(/[\\/]/).pop() ?? ''
          const shouldUseSavedPath = Boolean(
            normalizedSavedPath
            && normalizedSavedPath !== normalizedDownloadPath
            && /\.[A-Z0-9]{1,5}$/i.test(savedFileName),
          )

          // Strategy 1: If we have a saved file path from yutto output, use it
          if (newTask.savedFilePath && shouldUseSavedPath) {
            actualFilePath = newTask.savedFilePath
            console.log(`[Queue Store] 使用解析的文件路径: ${actualFilePath}`)

            // Get actual file size from filesystem
            try {
              const { getFileSize } = await import('@/services/tauri')
              fileSize = await getFileSize(actualFilePath)
              console.log(`[Queue Store] 实际文件大小: ${fileSize} bytes (${(fileSize / 1024 / 1024).toFixed(2)} MB)`)
            }
            catch (error) {
              console.error('[Queue Store] 获取文件大小失败:', error)
            }
          }
          // Strategy 2: Find the newest file in the download directory after the download started
          else {
            console.log(`[Queue Store] 未解析到文件路径，尝试查找最新文件`)
            try {
              const { findNewestFileInDir } = await import('@/services/tauri')
              const mediaExtensions = ['mp4', 'mkv', 'flv', 'mov', 'm4v', 'webm', 'ts', 'mp3', 'm4a', 'aac', 'flac', 'ogg', 'wav']
              const newestFile = await findNewestFileInDir(
                newTask.config.downloadPath,
                newTask.startTime,
                mediaExtensions,
                newTask.videoInfo?.title,
              )
              const fallbackFile = newestFile
                ?? await findNewestFileInDir(newTask.config.downloadPath, newTask.startTime, mediaExtensions)
              if (fallbackFile) {
                actualFilePath = fallbackFile
                console.log(`[Queue Store] 找到最新文件: ${actualFilePath}`)

                // Get actual file size from filesystem
                try {
                  const { getFileSize } = await import('@/services/tauri')
                  fileSize = await getFileSize(actualFilePath)
                  console.log(`[Queue Store] 实际文件大小: ${fileSize} bytes (${(fileSize / 1024 / 1024).toFixed(2)} MB)`)
                }
                catch (error) {
                  console.error('[Queue Store] 获取文件大小失败:', error)
                }
              }
              else {
                console.log(`[Queue Store] 未找到最新文件，使用解析的文件大小: ${fileSize} bytes`)
              }
            }
            catch (error) {
              console.error('[Queue Store] 查找最新文件失败:', error)
            }
          }

          await historyStore.addEntry({
            id: generateId(),
            title: newTask.videoInfo.title,
            url: newTask.config.url,
            thumbnail: newTask.videoInfo.thumbnail,
            downloadDate: Date.now(),
            filePath: actualFilePath,
            videoQuality: newTask.config.videoQuality,
            audioQuality: newTask.config.audioQuality,
            videoOnly: newTask.config.videoOnly,
            audioOnly: newTask.config.audioOnly,
            size: fileSize,
          })
          console.log(`[Queue Store] 已保存到历史记录: ${newTask.videoInfo.title}`)
        }
        catch (error) {
          console.error('[Queue Store] 保存历史记录失败:', error)
        }
      }
    }
  }

  function updateProgress(progress: DownloadProgress) {
    const task = tasks.value.find(t => t.id === progress.downloadId)
    if (task) {
      task.progress = progress.progress
      task.speed = progress.speed
      task.eta = progress.eta
    }
  }

  function pauseTask(id: string) {
    const task = tasks.value.find(t => t.id === id)
    if (task && task.status === 'downloading') {
      task.status = 'paused'
    }
  }

  function resumeTask(id: string) {
    const task = tasks.value.find(t => t.id === id)
    if (task && task.status === 'paused') {
      task.status = 'downloading'
    }
  }

  function setActiveTask(id: string | null) {
    activeTaskId.value = id
  }

  function clearCompleted() {
    tasks.value = tasks.value.filter(task => task.status !== 'completed')
  }

  return {
    // State
    tasks,
    activeTaskId,
    // Getters
    pendingTasks,
    downloadingTasks,
    completedTasks,
    errorTasks,
    pausedTasks,
    activeTask,
    // Actions
    addTask,
    removeTask,
    updateTask,
    updateProgress,
    pauseTask,
    resumeTask,
    setActiveTask,
    clearCompleted,
  }
}, {
  persist: true,
})
