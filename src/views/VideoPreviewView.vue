<script lang="ts" setup>
import { convertFileSrc } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { ArrowLeft, Download, FolderOpen } from 'lucide-vue-next'
import * as PlyrNamespace from 'plyr'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Button from '@/components/common/Button.vue'
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

onMounted(() => {
  if (videoElement.value) {
    player = new Plyr(videoElement.value, {
      controls: ['play-large', 'play', 'progress', 'current-time', 'mute', 'volume', 'settings', 'fullscreen'],
      settings: ['quality', 'speed'],
    })
  }
})

onUnmounted(() => {
  player?.destroy()
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
  <div class="page-container">
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
      <div class="flex">
        <Button class="b-none hover:bg-#f8f9fa" variant="secondary" @click="router.push('/history')">
          <ArrowLeft :size="16" @click="router.push('/history')" />
        </Button>
        <h1 class="text-2xl text-text-primary font-bold">
          {{ entry.title }}
        </h1>
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
    </div>
  </div>
</template>

<style scoped>
.video-container {
  position: relative;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto 1rem;
  aspect-ratio: 16 / 9;
  background: var(--color-bg-tertiary);
  border-radius: 1rem;
  overflow: hidden;
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
</style>
