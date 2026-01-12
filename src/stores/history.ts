import type { HistoryEntry } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useHistoryStore = defineStore('history', () => {
  // State
  const entries = ref<HistoryEntry[]>([])
  const filter = ref<string>('')
  const sortBy = ref<'date' | 'title' | 'size'>('date')

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
  function addEntry(entry: HistoryEntry) {
    // 检查是否已存在
    const exists = entries.value.some(e => e.id === entry.id)
    if (!exists) {
      entries.value.unshift(entry)
    }
  }

  function removeEntry(id: string) {
    const index = entries.value.findIndex(entry => entry.id === id)
    if (index > -1) {
      entries.value.splice(index, 1)
    }
  }

  function clearHistory() {
    entries.value = []
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
    // Getters
    filteredEntries,
    totalSize,
    totalCount,
    // Actions
    addEntry,
    removeEntry,
    clearHistory,
    setFilter,
    setSortBy,
  }
}, {
  persist: true,
})
