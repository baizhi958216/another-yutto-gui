<script lang="ts" setup>
import type { Comment } from '@/types'
import { convertFileSrc } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { ArrowLeft, Download, FolderOpen, MessageCircle } from 'lucide-vue-next'
import * as PlyrNamespace from 'plyr'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { readCsvFile } from '@/services/tauri'
import { useDownloadStore } from '@/stores/download'
import { useHistoryStore } from '@/stores/history'
import { formatFileSize, formatRelativeTime } from '@/utils/format'
import { formatAudioQuality, formatVideoQuality } from '@/utils/quality'
import 'plyr/dist/plyr.css'

const Plyr = (PlyrNamespace as any).default || PlyrNamespace

const route = useRoute()
const router = useRouter()
const historyStore = useHistoryStore()
const downloadStore = useDownloadStore()
const entryId = route.params.id as string
const entry = computed(() =>
  historyStore.entries.find(e => e.id === entryId),
)

const videoUrl = computed(() =>
  entry.value ? convertFileSrc(entry.value.filePath) : '',
)

const videoElement = ref<HTMLVideoElement | null>(null)
let player: Plyr | null = null

// 评论相关状态
const comments = ref<Comment[]>([])
const loadingComments = ref(false)
const commentsError = ref<string | null>(null)
const hasCommentFile = ref(false)

// 排序相关状态
type SortType = 'time' | 'likes'
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

function nextPage() {
  if (hasNextPage.value) {
    currentPage.value++
    // Scroll to top of comments section
    const commentsSection = document.querySelector('.comments-section')
    if (commentsSection) {
      commentsSection.scrollIntoView({ behavior: 'smooth' })
    }
  }
}

function prevPage() {
  if (hasPrevPage.value) {
    currentPage.value--
    // Scroll to top of comments section
    const commentsSection = document.querySelector('.comments-section')
    if (commentsSection) {
      commentsSection.scrollIntoView({ behavior: 'smooth' })
    }
  }
}

function goToPage(page: number) {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page
    // Scroll to top of comments section
    const commentsSection = document.querySelector('.comments-section')
    if (commentsSection) {
      commentsSection.scrollIntoView({ behavior: 'smooth' })
    }
  }
}

function changeSortType(type: SortType) {
  sortType.value = type
  currentPage.value = 1 // 切换排序时重置到第一页
}

onMounted(async () => {
  if (videoElement.value) {
    player = new Plyr(videoElement.value, {
      controls: ['play-large', 'play', 'progress', 'current-time', 'mute', 'volume', 'settings', 'fullscreen'],
      settings: ['quality', 'speed'],
    })
  }

  // 自动加载本地评论
  if (entry.value?.commentFilePath) {
    await loadLocalComments()
  }
})

onUnmounted(() => {
  player?.destroy()
})

async function loadLocalComments() {
  if (!entry.value?.commentFilePath)
    return

  loadingComments.value = true
  commentsError.value = null

  try {
    const csvContent = await readCsvFile(entry.value.commentFilePath)
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

function parseCsvComments(csvContent: string): Comment[] {
  const lines = csvContent.trim().split('\n')
  if (lines.length <= 1) {
    return []
  }

  // Skip header line
  const dataLines = lines.slice(1)
  const parsedComments: Comment[] = []

  for (const line of dataLines) {
    try {
      // Simple CSV parsing (handles quoted fields)
      const fields: string[] = []
      let currentField = ''
      let inQuotes = false

      for (let i = 0; i < line.length; i++) {
        const char = line[i]

        if (char === '"') {
          if (inQuotes && line[i + 1] === '"') {
            // Escaped quote
            currentField += '"'
            i++
          }
          else {
            inQuotes = !inQuotes
          }
        }
        else if (char === ',' && !inQuotes) {
          fields.push(currentField)
          currentField = ''
        }
        else {
          currentField += char
        }
      }
      fields.push(currentField)

      // CSV format: rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent
      if (fields.length >= 12) {
        parsedComments.push({
          rpid: Number.parseInt(fields[0]),
          oid: Number.parseInt(fields[1]),
          mid: Number.parseInt(fields[2]),
          uname: fields[3],
          sex: fields[4],
          content: fields[5],
          avatar: fields[6],
          ctime: Number.parseInt(fields[7]),
          like: Number.parseInt(fields[8]),
          current_level: Number.parseInt(fields[9]),
          location: fields[10],
          parent: Number.parseInt(fields[11]),
          pictures: [],
        })
      }
    }
    catch (error) {
      console.error('Failed to parse comment line:', error)
    }
  }

  return parsedComments
}

function formatCommentTime(timestamp: number): string {
  return formatRelativeTime(timestamp * 1000) // 转换为毫秒
}

async function handleRedownload() {
  if (!entry.value)
    return
  await router.push('/')
  await downloadStore.fetchVideoInfo(entry.value.url)
}

async function handleOpenFolder() {
  if (!entry.value)
    return
  await revealItemInDir(entry.value.filePath)
}
</script>

<template>
  <div class="page-container pt-0">
    <Button v-if="!entry" variant="secondary" class="b-none hover:bg-#f8f9fa" @click="router.push('/history')">
      <ArrowLeft :size="16" />
    </Button>

    <div v-if="!entry" class="py-20 text-center">
      <div class="mx-auto mb-6 rounded-full bg-bg-tertiary flex h-32 w-32 items-center justify-center">
        <span class="text-5xl">❌</span>
      </div>
      <h2 class="text-xl text-text-secondary font-bold">
        未找到该视频记录
      </h2>
      <p class="text-sm text-text-tertiary mt-2">
        该视频可能已被删除
      </p>
    </div>

    <div v-else class="mt-6 space-y-6">
      <div class="flex gap-2">
        <Button class="b-none h-fit hover:bg-#f8f9fa" variant="secondary" @click="router.push('/history')">
          <ArrowLeft :size="16" @click="router.push('/history')" />
        </Button>

        <Tooltip
          :text="entry.title"
          class="w-full"
        >
          <h1 class="text-2xl text-text-primary font-bold w-90% truncate">
            {{ entry.title }}
          </h1>
        </Tooltip>
      </div>

      <div class="video-container">
        <video
          ref="videoElement"
          :src="videoUrl"
          class="rounded-2xl w-full"
        />
      </div>
      <div class="flex gap-3">
        <Button variant="primary" @click="handleRedownload">
          <div class="flex items-center justify-center">
            <Download :size="16" class="mr-2" />
            重新下载
          </div>
        </Button>
        <Button variant="secondary" @click="handleOpenFolder">
          <div class="flex items-center justify-center">
            <FolderOpen :size="16" class="mr-2" />
            打开文件夹
          </div>
        </Button>
      </div>

      <div class="pt-1">
        <div class="text-xs gap-3 grid grid-cols-2">
          <div v-if="!entry.audioOnly">
            <span class="text-text-tertiary">视频质量:</span>
            <span class="text-text-primary ml-2">
              {{ formatVideoQuality(entry.videoQuality) }}
            </span>
          </div>
          <div v-if="!entry.videoOnly">
            <span class="text-text-tertiary">音频质量:</span>
            <span class="text-text-primary ml-2">
              {{ formatAudioQuality(entry.audioQuality) }}
            </span>
          </div>
          <div>
            <span class="text-text-tertiary">文件大小:</span>
            <span class="text-text-primary ml-2">
              {{ formatFileSize(entry.size) }}
            </span>
          </div>
          <div>
            <span class="text-text-tertiary">下载时间:</span>
            <span class="text-text-primary ml-2">
              {{ formatRelativeTime(entry.downloadDate) }}
            </span>
          </div>
          <div class="col-span-2">
            <span class="text-text-tertiary">原始链接:</span>
            <a
              :href="entry.url"
              target="_blank"
              class="text-primary ml-2 block truncate hover:underline"
            >
              {{ entry.url }}
            </a>
          </div>
          <div class="col-span-2">
            <span class="text-text-tertiary">本地路径:</span>
            <span class="text-text-secondary ml-2 block truncate">
              {{ entry.filePath }}
            </span>
          </div>
        </div>
      </div>

      <!-- 评论区域 -->
      <div v-if="hasCommentFile" class="comments-section mt-8">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="text-xl text-text-primary font-bold flex gap-2 items-center">
            评论 {{ totalComments > 0 ? `(${totalComments})` : '' }}
          </h2>

          <!-- 排序选项 -->
          <div class="flex gap-2">
            <button
              class="text-sm px-3 py-1.5 rounded transition-colors"
              :class="sortType === 'time' ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
              @click="changeSortType('time')"
            >
              按时间
            </button>
            <button
              class="text-sm px-3 py-1.5 rounded transition-colors"
              :class="sortType === 'likes' ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
              @click="changeSortType('likes')"
            >
              按点赞
            </button>
          </div>
        </div>

        <div v-if="commentsError" class="text-sm text-error p-4 rounded bg-bg-secondary">
          {{ commentsError }}
        </div>

        <div v-if="loadingComments" class="py-8 text-center">
          <div class="text-text-tertiary">
            加载评论中...
          </div>
        </div>

        <div v-else-if="totalComments === 0" class="py-8 text-center">
          <div class="text-text-tertiary">
            暂无评论
          </div>
        </div>

        <div v-else>
          <TransitionGroup name="comment-list" tag="div" class="space-y-4">
            <div
              v-for="comment in paginatedComments"
              :key="comment.rpid"
              class="p-4 rounded-lg bg-bg-secondary transition-colors hover:bg-bg-tertiary"
            >
              <div class="flex gap-3">
                <!-- 用户头像 -->
                <img
                  :src="comment.avatar"
                  :alt="comment.uname"
                  class="rounded-full flex-shrink-0 h-10 w-10"
                  referrerpolicy="no-referrer"
                >
                <div class="flex-1 min-w-0">
                  <!-- 用户信息 -->
                  <div class="mb-2 flex gap-2 items-center">
                    <span class="text-sm text-text-primary font-medium">
                      {{ comment.uname }}
                    </span>
                    <span class="text-xs text-text-tertiary">
                      LV{{ comment.current_level }}
                    </span>
                    <span class="text-xs text-text-tertiary">
                      {{ formatCommentTime(comment.ctime) }}
                    </span>
                    <span v-if="comment.location" class="text-xs text-text-tertiary">
                      {{ comment.location }}
                    </span>
                  </div>
                  <!-- 评论内容 -->
                  <div class="text-sm text-text-primary whitespace-pre-wrap break-words">
                    {{ comment.content }}
                  </div>
                  <!-- 评论图片 -->
                  <div v-if="comment.pictures && comment.pictures.length > 0" class="mt-2 flex flex-wrap gap-2">
                    <img
                      v-for="(pic, idx) in comment.pictures"
                      :key="idx"
                      :src="pic.img_src"
                      class="rounded max-h-32 object-cover"
                      referrerpolicy="no-referrer"
                    >
                  </div>
                  <!-- 点赞数 -->
                  <div class="text-xs text-text-tertiary mt-2 flex gap-1 w-fit items-center justify-center">
                    <div class="i-carbon:thumbs-up-filled mt--0.5" />
                    {{ comment.like }}
                  </div>
                </div>
              </div>
            </div>
          </TransitionGroup>

          <!-- 分页控件 -->
          <div v-if="totalPages > 1" class="mt-6 flex gap-2 items-center justify-center">
            <Button
              variant="secondary"
              :disabled="!hasPrevPage"
              @click="prevPage"
            >
              上一页
            </Button>

            <div class="flex gap-1">
              <!-- 第一页 -->
              <button
                v-if="currentPage > 3"
                class="text-sm px-3 py-1 rounded transition-colors"
                :class="currentPage === 1 ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
                @click="goToPage(1)"
              >
                1
              </button>
              <span v-if="currentPage > 4" class="text-text-tertiary px-2 py-1">...</span>

              <!-- 当前页附近的页码 -->
              <button
                v-for="page in [currentPage - 2, currentPage - 1, currentPage, currentPage + 1, currentPage + 2].filter(p => p >= 1 && p <= totalPages)"
                :key="page"
                class="text-sm px-3 py-1 rounded transition-colors"
                :class="page === currentPage ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
                @click="goToPage(page)"
              >
                {{ page }}
              </button>

              <!-- 最后一页 -->
              <span v-if="currentPage < totalPages - 3" class="text-text-tertiary px-2 py-1">...</span>
              <button
                v-if="currentPage < totalPages - 2"
                class="text-sm px-3 py-1 rounded transition-colors"
                :class="currentPage === totalPages ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
                @click="goToPage(totalPages)"
              >
                {{ totalPages }}
              </button>
            </div>

            <Button
              variant="secondary"
              :disabled="!hasNextPage"
              @click="nextPage"
            >
              下一页
            </Button>

            <span class="text-sm text-text-tertiary ml-4">
              第 {{ currentPage }} / {{ totalPages }} 页
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.video-container {
  position: relative;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto 1rem;
  background: var(--color-bg-tertiary);
  border-radius: 1rem;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-container video {
  max-height: 80vh;
  width: 100%;
  height: auto;
  object-fit: contain;
}

/* Plyr custom theme - Teal colors */
:deep(.plyr--video) {
  --plyr-color-main: #14b8a6;
}

:deep(.plyr__control--overlaid) {
  background: rgba(20, 184, 166, 0.9);
}

:deep(.plyr__control:hover) {
  background: #0d9488;
}

:deep(.plyr__menu__container .plyr__control[role='menuitemradio'][aria-checked='true']::before) {
  background: #14b8a6;
}

/* 评论列表动画 */
.comment-list-move,
.comment-list-enter-active,
.comment-list-leave-active {
  transition: all 0.3s ease;
}

.comment-list-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.comment-list-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

.comment-list-leave-active {
  position: absolute;
  width: 100%;
}
</style>
