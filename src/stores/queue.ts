import type { DownloadProgress, DownloadTask } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

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

  function updateTask(id: string, updates: Partial<DownloadTask>) {
    const index = tasks.value.findIndex(t => t.id === id)
    if (index !== -1) {
      // 创建新对象以确保触发响应式更新
      const oldTask = tasks.value[index]
      const newTask = { ...oldTask, ...updates }

      // 强制创建新数组引用以触发 Vue 响应式
      tasks.value = [
        ...tasks.value.slice(0, index),
        newTask,
        ...tasks.value.slice(index + 1),
      ]

      console.log(`[Queue Store] 更新任务 ${id.substring(0, 8)}: progress=${updates.progress?.toFixed(1)}%, speed=${updates.speed}`)
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
