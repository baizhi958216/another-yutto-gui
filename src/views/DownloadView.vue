<script lang="ts" setup>
import { ref, computed, watch, onMounted } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useDownload } from '@/composables/useDownload'
import { useDownloadStore } from '@/stores/download'
import { usePresetsStore } from '@/stores/presets'

const downloadStore = useDownloadStore()
const presetsStore = usePresetsStore()
const { submitDownload } = useDownload()

// Sync URL with store to persist across route changes
const url = ref(downloadStore.currentUrl)

// Watch store changes and update local ref
watch(() => downloadStore.currentUrl, (newUrl) => {
  url.value = newUrl
})

// Compute available quality options from videoInfo or use defaults
const qualityOptions = computed(() => {
  if (downloadStore.videoInfo?.available_qualities && downloadStore.videoInfo.available_qualities.length > 0) {
    return downloadStore.videoInfo.available_qualities
  }
  // Default quality options if not available from backend
  return [
    { quality: 127, description: '8K 超高清' },
    { quality: 126, description: '杜比视界' },
    { quality: 125, description: 'HDR 真彩' },
    { quality: 120, description: '4K 超清' },
    { quality: 116, description: '1080P 60帧' },
    { quality: 112, description: '1080P 高码率' },
    { quality: 80, description: '1080P 高清' },
    { quality: 64, description: '720P 高清' },
  ]
})

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
  <div class="page-container">
    <Card title="下载视频">
      <div class="space-y-4">
        <!-- URL 输入 -->
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            视频链接
          </label>
          <div class="flex gap-2">
            <Input
              v-model="url"
              placeholder="请输入 B 站视频链接 (BV号/番剧)"
              class="flex-1"
              @keyup.enter="handleFetchInfo"
            />
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
        <div v-if="downloadStore.videoInfo" class="card-hover p-4">
          <div class="flex gap-4">
            <img
              :src="downloadStore.videoInfo.thumbnail"
              :alt="downloadStore.videoInfo.title"
              class="rounded-lg h-20 w-32 object-cover"
              referrerpolicy="no-referrer"
            >
            <div class="flex-1">
              <h3 class="text-lg text-text-primary font-semibold mb-2">
                {{ downloadStore.videoInfo.title }}
              </h3>
              <p class="text-sm text-text-secondary">
                UP主: {{ downloadStore.videoInfo.owner.name }}
              </p>
            </div>
          </div>
        </div>

        <!-- 下载配置 -->
        <div v-if="downloadStore.currentConfig" class="space-y-3">
          <div>
            <label class="text-sm text-text-primary font-medium mb-2 block">
              视频质量
            </label>
            <select
              v-model="downloadStore.currentConfig.videoQuality"
              class="input-base"
            >
              <option
                v-for="option in qualityOptions"
                :key="option.quality"
                :value="option.quality"
              >
                {{ option.description }}
              </option>
            </select>
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
    </Card>
  </div>
</template>
