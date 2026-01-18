import type { HistoryEntry } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { addToHistory, clearHistory as clearHistoryBackend, deleteHistoryEntry, getHistory } from '@/services/tauri'

export const useHistoryStore = defineStore('history', () => {
  // State
  const entries = ref<HistoryEntry[]>([])
  const filter = ref<string>('')
  const sortBy = ref<'date' | 'title' | 'size'>('date')
  const isLoading = ref(false)

  // Getters
  const filteredEntries = computed(() => {
    let result = [...entries.value]

    // 应用筛选
    if (filter.value) {
      const lowerFilter = filter.value.toLowerCase()
      result = result.filter(entry =>
        entry.title.toLowerCase().includes(lowerFilter)
        || entry.url.toLowerCase().includes(lowerFilter),
      )
    }

    // 应用排序
    result.sort((a, b) => {
      switch (sortBy.value) {
        case 'date':
          return b.downloadDate - a.downloadDate
        case 'title':
          return a.title.localeCompare(b.title, 'zh-CN')
        case 'size':
          return b.size - a.size
        default:
          return 0
      }
    })

    return result
  })

  const totalSize = computed(() =>
    entries.value.reduce((sum, entry) => sum + entry.size, 0),
  )

  const totalCount = computed(() => entries.value.length)

  // Actions
  async function loadHistory() {
    try {
      isLoading.value = true
      const history = await getHistory()
      entries.value = history
    }
    catch (error) {
      console.error('Failed to load history:', error)
    }
    finally {
      isLoading.value = false
    }
  }

  async function addEntry(entry: HistoryEntry) {
    try {
      // 检查是否已存在
      const exists = entries.value.some(e => e.id === entry.id)
      if (!exists) {
        // 保存到后端
        await addToHistory(entry)
        // 添加到本地状态
        entries.value.unshift(entry)
      }
    }
    catch (error) {
      console.error('Failed to add history entry:', error)
      throw error
    }
  }

  async function removeEntry(id: string) {
    try {
      // 从后端删除
      await deleteHistoryEntry(id)
      // 从本地状态删除
      const index = entries.value.findIndex(entry => entry.id === id)
      if (index > -1) {
        entries.value.splice(index, 1)
      }
    }
    catch (error) {
      console.error('Failed to remove history entry:', error)
      throw error
    }
  }

  async function clearHistoryAction() {
    try {
      // 清空后端
      await clearHistoryBackend()
      // 清空本地状态
      entries.value = []
    }
    catch (error) {
      console.error('Failed to clear history:', error)
      throw error
    }
  }

  function setFilter(newFilter: string) {
    filter.value = newFilter
  }

  function setSortBy(newSortBy: 'date' | 'title' | 'size') {
    sortBy.value = newSortBy
  }

  return {
    // State
    entries,
    filter,
    sortBy,
    isLoading,
    // Getters
    filteredEntries,
    totalSize,
    totalCount,
    // Actions
    loadHistory,
    addEntry,
    removeEntry,
    clearHistory: clearHistoryAction,
    setFilter,
    setSortBy,
  }
})
