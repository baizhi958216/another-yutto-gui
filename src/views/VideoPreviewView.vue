<script lang="ts" setup>
import { convertFileSrc } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { ArrowLeft, Download, FolderOpen } from 'lucide-vue-next'
import * as PlyrNamespace from 'plyr'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AudioVisualizer from '@/components/common/AudioVisualizer.vue'
import Button from '@/components/common/Button.vue'
import CommentList from '@/components/common/CommentList.vue'
import DanmakuPlayer from '@/components/common/DanmakuPlayer.vue'
import { useComments } from '@/composables/useComments'
import { useDownloadStore } from '@/stores/download'
import { useHistoryStore } from '@/stores/history'
import { useTitleBarStore } from '@/stores/titleBar'
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

const isAudioOnly = computed(() => entry.value?.audioOnly ?? false)

const videoElement = ref<HTMLVideoElement | null>(null)
let player: Plyr | null = null

// 弹幕显示状态
const danmakuVisible = ref(true)

// 使用评论组合式函数
const {
  paginatedComments,
  loadingComments,
  commentsError,
  hasCommentFile,
  sortType,
  currentPage,
  totalComments,
  totalPages,
  hasNextPage,
  hasPrevPage,
  nextPage,
  prevPage,
  goToPage,
  changeSortType,
  loadLocalComments,
} = useComments()

const titleBar = useTitleBarStore()
onMounted(async () => {
  // 只在非音频模式下初始化视频播放器
  if (videoElement.value && !entry.value?.audioOnly) {
    player = new Plyr(videoElement.value, {
      controls: [
        'play-large',
        'play',
        'progress',
        'current-time',
        'mute',
        'volume',
        'settings',
        'fullscreen',
      ],
      settings: ['quality', 'speed'],
    })

    // 等待 Plyr 完全初始化后添加弹幕按钮
    player!.on('ready', () => {
      const controlsContainer = document.querySelector('.plyr__controls')
      if (controlsContainer) {
        // 创建弹幕切换按钮
        const danmakuButton = document.createElement('button')
        danmakuButton.type = 'button'
        danmakuButton.className = 'plyr__controls__item plyr__control'
        danmakuButton.setAttribute('data-plyr', 'danmaku')
        danmakuButton.innerHTML = `
          <svg class="icon--pressed" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
          </svg>
          <svg class="icon--not-pressed" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
            <line x1="3" y1="3" x2="21" y2="21"></line>
          </svg>
          <span class="plyr__tooltip">弹幕</span>
        `
        danmakuButton.setAttribute('aria-pressed', danmakuVisible.value.toString())

        // 添加点击事件
        danmakuButton.addEventListener('click', () => {
          danmakuVisible.value = !danmakuVisible.value
          danmakuButton.setAttribute('aria-pressed', danmakuVisible.value.toString())
        })

        // 插入到全屏按钮之前
        const fullscreenButton = controlsContainer.querySelector('[data-plyr="fullscreen"]')
        if (fullscreenButton) {
          controlsContainer.insertBefore(danmakuButton, fullscreenButton)
        }
        else {
          controlsContainer.appendChild(danmakuButton)
        }
      }
    })
  }

  // 自动加载本地评论
  if (entry.value?.commentFilePath) {
    await loadLocalComments(entry.value.commentFilePath)
  }

  titleBar.setBranding({
    title: entry.value?.title,
    actions: [
      { id: 'back', label:
      `<div><div class="i-ic:round-arrow-back text-5" /></div>`, onClick: () => {
        router.push('/history')
      } },
    ],
  })
})

onUnmounted(() => {
  player?.destroy()
  titleBar.setBranding({
    visible: false,
  })
})

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
    <Button v-if="!entry" variant="secondary" class="b-none hover:bg-bg-tertiary" @click="router.push('/history')">
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
      <!-- 音频可视化播放器 -->
      <AudioVisualizer
        v-if="isAudioOnly"
        :src="videoUrl"
        :title="entry.title"
        :cover="entry.thumbnail"
      />

      <!-- 视频播放器 -->
      <div v-else class="video-container">
        <video
          ref="videoElement"
          :src="videoUrl"
          class="rounded-2xl w-full"
        />
        <DanmakuPlayer
          v-if="entry"
          :video-element="videoElement"
          :video-file-path="entry.filePath"
          :visible="danmakuVisible"
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
      <CommentList
        v-if="hasCommentFile"
        :comments="paginatedComments"
        :total-comments="totalComments"
        :sort-type="sortType"
        :current-page="currentPage"
        :total-pages="totalPages"
        :has-next-page="hasNextPage"
        :has-prev-page="hasPrevPage"
        :loading="loadingComments"
        :error="commentsError"
        @change-sort-type="changeSortType"
        @next-page="nextPage"
        @prev-page="prevPage"
        @go-to-page="goToPage"
      />
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

/* Plyr custom theme - Accent colors */
:deep(.plyr--video) {
  --plyr-color-main: var(--color-accent-500);
}

:deep(.plyr__control--overlaid) {
  background: rgba(var(--color-accent-500-rgb), 0.9);
}

:deep(.plyr__control:hover) {
  background: var(--color-accent-600);
}

:deep(.plyr__menu__container .plyr__control[role='menuitemradio'][aria-checked='true']::before) {
  background: var(--color-accent-500);
}

/* 弹幕按钮样式 */
:deep([data-plyr='danmaku']) {
  position: relative;
}

:deep([data-plyr='danmaku'] svg) {
  width: 18px;
  height: 18px;
}

:deep([data-plyr='danmaku'] .icon--pressed) {
  display: none;
}

:deep([data-plyr='danmaku'] .icon--not-pressed) {
  display: block;
}

:deep([data-plyr='danmaku'][aria-pressed='true'] .icon--pressed) {
  display: block;
}

:deep([data-plyr='danmaku'][aria-pressed='true'] .icon--not-pressed) {
  display: none;
}
</style>
