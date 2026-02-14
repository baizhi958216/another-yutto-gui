import type { Comment } from '@/types'
import { computed, ref } from 'vue'
import { readCsvFile } from '@/services/tauri'
import { parseCsvComments } from '@/utils/csv-parser'
import { normalizeCommentImages } from '@/utils/image'

export type SortType = 'time' | 'likes'

function getRootId(comment: Comment): number {
  if (comment.root > 0) {
    return comment.root
  }

  if (comment.parent > 0) {
    return comment.parent
  }

  return comment.rpid
}

export function useComments() {
  // Flat comments loaded from CSV (both top-level comments and replies).
  const comments = ref<Comment[]>([])
  const loadingComments = ref(false)
  const commentsError = ref<string | null>(null)
  const hasCommentFile = ref(false)

  // 排序相关状态
  const sortType = ref<SortType>('time')

  // 分页相关状态
  const currentPage = ref(1)
  const pageSize = 20

  const topLevelComments = computed(() => {
    return comments.value.filter(comment => comment.parent === 0)
  })

  const repliesByRoot = computed(() => {
    const map = new Map<number, Comment[]>()

    for (const comment of comments.value) {
      if (comment.parent === 0)
        continue

      const rootId = getRootId(comment)
      const existingReplies = map.get(rootId)
      if (existingReplies) {
        existingReplies.push(comment)
      }
      else {
        map.set(rootId, [comment])
      }
    }

    for (const [rootId, replies] of map.entries()) {
      map.set(rootId, [...replies].sort((a, b) => a.ctime - b.ctime))
    }

    return map
  })

  const sortedComments = computed(() => {
    const sorted = [...topLevelComments.value]
    if (sortType.value === 'time') {
      // 按时间降序排序（最新的在前）
      return sorted.sort((a, b) => b.ctime - a.ctime)
    }
    else {
      // 按点赞数降序排序（点赞最多的在前）
      return sorted.sort((a, b) => b.like - a.like)
    }
  })

  const totalComments = computed(() => sortedComments.value.length)

  const paginatedComments = computed(() => {
    const start = (currentPage.value - 1) * pageSize
    const end = start + pageSize

    return sortedComments.value.slice(start, end).map(comment => ({
      ...comment,
      replies: repliesByRoot.value.get(comment.rpid) || [],
    }))
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
      comments.value = parseCsvComments(csvContent).map(normalizeCommentImages)
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
