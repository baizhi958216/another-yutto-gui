<script lang="ts" setup>
import { ref } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useDownload } from '@/composables/useDownload'
import { useDownloadStore } from '@/stores/download'
import { usePresetsStore } from '@/stores/presets'

const downloadStore = useDownloadStore()
const presetsStore = usePresetsStore()
const { submitDownload } = useDownload()

const url = ref('')

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
              <option :value="127">
                8K 超高清
              </option>
              <option :value="126">
                杜比视界
              </option>
              <option :value="125">
                HDR 真彩
              </option>
              <option :value="120">
                4K 超清
              </option>
              <option :value="116">
                1080P 60帧
              </option>
              <option :value="112">
                1080P 高码率
              </option>
              <option :value="80">
                1080P 高清
              </option>
              <option :value="64">
                720P 高清
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
