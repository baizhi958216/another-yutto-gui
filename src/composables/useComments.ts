import type { Comment } from '@/types'
import { computed, ref } from 'vue'
import { readCsvFile } from '@/services/tauri'
import { parseCsvComments } from '@/utils/csv-parser'

export type SortType = 'time' | 'likes'

export function useComments() {
  const comments = ref<Comment[]>([])
  const loadingComments = ref(false)
  const commentsError = ref<string | null>(null)
  const hasCommentFile = ref(false)

  // 排序相关状态
  const sortType = ref<SortType>('time')

  // 分页相关状态
  const currentPage = ref(1)
  const pageSize = 20
  const totalComments = ref(0)

  const sortedComments = computed(() => {
    const sorted = [...comments.value]
    if (sortType.value === 'time') {
      // 按时间降序排序（最新的在前）
      return sorted.sort((a, b) => b.ctime - a.ctime)
    }
    else {
      // 按点赞数降序排序（点赞最多的在前）
      return sorted.sort((a, b) => b.like - a.like)
    }
  })

  const paginatedComments = computed(() => {
    const start = (currentPage.value - 1) * pageSize
    const end = start + pageSize
    return sortedComments.value.slice(start, end)
  })

  const totalPages = computed(() => {
    return Math.ceil(totalComments.value / pageSize)
  })

  const hasNextPage = computed(() => {
    return currentPage.value < totalPages.value
  })

  const hasPrevPage = computed(() => {
    return currentPage.value > 1
  })

  function scrollToComments() {
    const commentsSection = document.querySelector('.comments-section')
    if (commentsSection) {
      commentsSection.scrollIntoView({ behavior: 'smooth' })
    }
  }

  function nextPage() {
    if (hasNextPage.value) {
      currentPage.value++
      scrollToComments()
    }
  }

  function prevPage() {
    if (hasPrevPage.value) {
      currentPage.value--
      scrollToComments()
    }
  }

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page
      scrollToComments()
    }
  }

  function changeSortType(type: SortType) {
    sortType.value = type
    currentPage.value = 1 // 切换排序时重置到第一页
  }

  async function loadLocalComments(commentFilePath: string) {
    if (!commentFilePath)
      return

    loadingComments.value = true
    commentsError.value = null

    try {
      const csvContent = await readCsvFile(commentFilePath)
      comments.value = parseCsvComments(csvContent)
      totalComments.value = comments.value.length
      currentPage.value = 1 // Reset to first page
      hasCommentFile.value = true
    }
    catch (error) {
      console.error('Failed to load local comments:', error)
      commentsError.value = '加载本地评论失败'
      hasCommentFile.value = false
    }
    finally {
      loadingComments.value = false
    }
  }

  return {
    // State
    comments,
    loadingComments,
    commentsError,
    hasCommentFile,
    sortType,
    currentPage,
    totalComments,

    // Computed
    sortedComments,
    paginatedComments,
    totalPages,
    hasNextPage,
    hasPrevPage,

    // Methods
    nextPage,
    prevPage,
    goToPage,
    changeSortType,
    loadLocalComments,
  }
}
