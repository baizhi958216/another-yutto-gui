<script lang="ts" setup>
import { Search } from 'lucide-vue-next'
import { computed, onMounted, ref, watch } from 'vue'
import TextPressure from '@/components/bits/TextPressure.vue'
import Button from '@/components/common/Button.vue'
import EpisodeSelector from '@/components/common/EpisodeSelector.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { useDownload } from '@/composables/useDownload'
import { useQualityOptions } from '@/composables/useQualityOptions'
import { useDownloadStore } from '@/stores/download'

const downloadStore = useDownloadStore()
const { submitDownload } = useDownload()

// Use quality options composable
const { qualityOptions, audioQualityOptions } = useQualityOptions(
  downloadStore.videoInfo?.available_qualities,
  downloadStore.videoInfo?.available_audio_qualities,
)

// Sync URL with store to persist across route changes
const url = ref(downloadStore.currentUrl)

// Watch store changes and update local ref
watch(() => downloadStore.currentUrl, (newUrl) => {
  url.value = newUrl
})

// 是否显示剧集选择器
const showEpisodeSelector = computed(() => {
  return downloadStore.videoInfo?.episodes
    && downloadStore.videoInfo.episodes.length > 1
})

// 判断是否为收藏夹（有多个剧集且有用户详细信息）
const isFavorite = computed(() => {
  return downloadStore.videoInfo?.episodes
    && downloadStore.videoInfo.episodes.length > 1
    && downloadStore.videoInfo.owner.sign !== undefined
})

// Handle video-only download
async function handleVideoOnlyDownload() {
  if (!downloadStore.currentConfig)
    return

  // Create a copy of the config with videoOnly flag
  // Disable danmaku, subtitle, and cover to avoid merging them
  const videoOnlyConfig = {
    ...downloadStore.currentConfig,
    videoOnly: true,
    audioOnly: false,
    withDanmaku: false,
    withSubtitle: false,
    withCover: false,
  }

  // Temporarily update the config
  const originalConfig = { ...downloadStore.currentConfig }
  downloadStore.updateConfig(videoOnlyConfig)

  // Submit the download
  await submitDownload()

  // Restore the original config
  downloadStore.updateConfig(originalConfig)
}

// Handle audio-only download
async function handleAudioOnlyDownload() {
  if (!downloadStore.currentConfig)
    return

  // Create a copy of the config with audioOnly flag
  // Disable danmaku, subtitle, and cover as they don't apply to audio-only
  const audioOnlyConfig = {
    ...downloadStore.currentConfig,
    videoOnly: false,
    audioOnly: true,
    withDanmaku: false,
    withSubtitle: false,
    withCover: false,
  }

  // Temporarily update the config
  const originalConfig = { ...downloadStore.currentConfig }
  downloadStore.updateConfig(audioOnlyConfig)

  // Submit the download
  await submitDownload()

  // Restore the original config
  downloadStore.updateConfig(originalConfig)
}

async function handleFetchInfo() {
  if (url.value) {
    await downloadStore.fetchVideoInfo(url.value)
    if (downloadStore.videoInfo) {
      // Use the normalized URL from currentUrl instead of the raw input
      downloadStore.initConfig(downloadStore.currentUrl)
    }
  }
}

async function handleSubmit() {
  // 验证剧集选择
  if (showEpisodeSelector.value && downloadStore.selectedEpisodes.size === 0) {
    downloadStore.error = '请至少选择一个剧集'
    return
  }

  await submitDownload()
  // Keep the URL and video info visible after adding to queue
}

// Restore URL from store on mount
onMounted(() => {
  if (downloadStore.currentUrl) {
    url.value = downloadStore.currentUrl
  }
})
</script>

<template>
  <div
    class="page-container flex h-full justify-center"
  >
    <div class="w-full space-y-4">
      <!-- Welcome Section -->
      <div v-if="!downloadStore.videoInfo" class="mb-10 mt-20 text-center">
        <TextPressure
          text="Another Yutto GUI"
          text-color="var(--color-accent-500)"
          stroke-color="#27FF64"
          :min-font-size="36"
          :italic="false"
        />
        <!-- <h1 class="text-3xl text-gray-800 font-extrabold mb-2">

        </h1> -->
      </div>
      <!-- URL 输入 -->
      <div>
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <Input
              v-model="url"
              placeholder="请输入 B 站链接 (视频/番剧/课程/收藏夹等，支持 BV/AV/EP/SS/MD 号)"
              class="pl-10 flex-1"
              @keyup.enter="handleFetchInfo"
            />
            <Search :size="18" class="text-text-tertiary left-3 top-1/2 absolute -translate-y-1/2" />
          </div>
          <Button
            variant="primary"
            :loading="downloadStore.isLoading"
            @click="handleFetchInfo"
          >
            获取信息
          </Button>
        </div>
        <p v-if="downloadStore.error" class="text-sm text-error mt-2">
          {{ downloadStore.error }}
        </p>
      </div>

      <!-- 视频信息预览 -->
      <div v-if="downloadStore.videoInfo" class="p-4 border border-border-primary rounded bg-bg-secondary shadow-sm transition-all hover:shadow-lg">
        <!-- 收藏夹用户信息展示 -->
        <div v-if="isFavorite" class="flex gap-4">
          <img
            :src="downloadStore.videoInfo.owner.face"
            :alt="downloadStore.videoInfo.owner.name"
            class="rounded-full h-20 w-20 object-cover"
            referrerpolicy="no-referrer"
          >
          <div class="flex-1">
            <h3 class="text-lg text-text-primary font-semibold mb-1">
              {{ downloadStore.videoInfo.owner.name }}
            </h3>
            <p v-if="downloadStore.videoInfo.owner.sign" class="text-sm text-text-secondary mb-1">
              {{ downloadStore.videoInfo.owner.sign }}
            </p>
            <div class="text-xs text-text-tertiary flex gap-3">
              <span v-if="downloadStore.videoInfo.owner.level">
                等级: Lv{{ downloadStore.videoInfo.owner.level }}
              </span>
              <span v-if="downloadStore.videoInfo.owner.location">
                IP属地: {{ downloadStore.videoInfo.owner.location }}
              </span>
            </div>
            <p class="text-sm text-text-secondary mt-2">
              收藏夹: {{ downloadStore.videoInfo.title }}
            </p>
          </div>
        </div>
        <!-- 普通视频信息展示 -->
        <div v-else class="flex gap-4">
          <img
            :src="downloadStore.videoInfo.thumbnail"
            :alt="downloadStore.videoInfo.title"
            class="rounded h-20 w-32 object-cover"
            referrerpolicy="no-referrer"
          >
          <div class="flex-1">
            <h3 class="text-lg text-text-primary font-semibold mb-2">
              {{ downloadStore.videoInfo.title }}
            </h3>
            <!-- 如果有 bvid 或 aid，显示 UP 主；否则显示制作信息 -->
            <p v-if="downloadStore.videoInfo.owner.name" class="text-sm text-text-secondary">
              <template v-if="downloadStore.videoInfo.bvid || downloadStore.videoInfo.aid">
                UP主: {{ downloadStore.videoInfo.owner.name }}
              </template>
              <template v-else>
                {{ downloadStore.videoInfo.owner.name }}
              </template>
            </p>
          </div>
        </div>
      </div>

      <!-- 下载配置 -->
      <div v-if="downloadStore.currentConfig" class="space-y-3">
        <!-- 只在非多剧集情况下显示全局质量选择 -->
        <div v-if="!showEpisodeSelector">
          <div>
            <div class="mb-2 flex items-center justify-between">
              <label class="text-sm text-text-primary font-medium">
                视频质量
              </label>
              <Button
                variant="secondary"
                @click="handleVideoOnlyDownload"
              >
                仅视频
              </Button>
            </div>
            <Select
              v-model="downloadStore.currentConfig.videoQuality"
              :options="qualityOptions.map(option => ({
                label: option.displayText,
                value: option.quality,
                disabled: !option.available,
              }))"
            />
          </div>

          <div>
            <div class="mb-2 flex items-center justify-between">
              <label class="text-sm text-text-primary font-medium">
                音频质量
              </label>
              <Button
                variant="secondary"
                @click="handleAudioOnlyDownload"
              >
                仅音频
              </Button>
            </div>
            <Select
              v-model="downloadStore.currentConfig.audioQuality"
              :options="audioQualityOptions.map(option => ({
                label: option.displayText,
                value: option.quality,
                disabled: !option.available,
              }))"
            />
          </div>
        </div>

        <!-- 剧集选择 -->
        <div v-if="showEpisodeSelector">
          <EpisodeSelector
            :episodes="downloadStore.videoInfo!.episodes!"
            :selected-indices="downloadStore.selectedEpisodes"
            :global-video-qualities="downloadStore.videoInfo!.available_qualities"
            :global-audio-qualities="downloadStore.videoInfo!.available_audio_qualities"
            :episode-qualities="downloadStore.episodeQualities"
            :is-favorite="isFavorite"
            @toggle="downloadStore.toggleEpisode"
            @select-all="downloadStore.selectAllEpisodes"
            @deselect-all="downloadStore.deselectAllEpisodes"
            @update-quality="downloadStore.updateEpisodeQuality"
          />
        </div>

        <div class="flex gap-4">
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="downloadStore.currentConfig.withDanmaku"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载弹幕</span>
          </label>
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="downloadStore.currentConfig.withSubtitle"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载字幕</span>
          </label>
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="downloadStore.currentConfig.withCover"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载封面</span>
          </label>
          <Tooltip
            :text="downloadStore.videoInfo?.comment_count
              ? `约 ${downloadStore.videoInfo.comment_count} 条评论`
              : '评论数量未知'"
          >
            <label class="flex gap-2 cursor-pointer items-center">
              <input
                v-model="downloadStore.currentConfig.withComments"
                type="checkbox"
                class="h-4 w-4"
              >
              <span class="text-sm text-text-primary">下载评论(耗时更长)</span>
            </label>
          </Tooltip>
        </div>

        <div class="flex gap-2 justify-end">
          <Button variant="secondary" @click="downloadStore.reset">
            取消
          </Button>
          <Button variant="primary" @click="handleSubmit">
            添加到队列
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
