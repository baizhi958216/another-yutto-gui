<script lang="ts" setup>
import { Pause, Play, Video, X } from 'lucide-vue-next'
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import ProgressBar from '@/components/common/ProgressBar.vue'
import { useDownload } from '@/composables/useDownload'
import { useTaskPolling } from '@/composables/useTaskPolling'
import { useQueueStore } from '@/stores/queue'

const queueStore = useQueueStore()
const { pauseDownload, resumeDownload, cancelDownload } = useDownload()

// 启动任务轮询，每秒更新一次
useTaskPolling(1000)

const allTasks = computed(() => queueStore.tasks)
const hasActiveTasks = computed(() => allTasks.value.length > 0)

function getStatusText(status: string) {
  const statusMap: Record<string, string> = {
    pending: '等待中',
    downloading: '下载中',
    paused: '已暂停',
    completed: '已完成',
    error: '错误',
  }
  return statusMap[status] || status
}

function getStatusColor(status: string) {
  const colorMap: Record<string, string> = {
    pending: 'text-text-secondary',
    downloading: 'text-teal-500',
    paused: 'text-warning',
    completed: 'text-success',
    error: 'text-error',
  }
  return colorMap[status] || 'text-text-secondary'
}

// 格式化视频质量显示
function formatVideoQuality(quality: number): string {
  const qualityMap: Record<number, string> = {
    127: '8K',
    126: '杜比视界',
    125: 'HDR',
    120: '4K',
    116: '1080P60',
    112: '1080P+',
    100: '智能修复',
    80: '1080P',
    74: '720P60',
    64: '720P',
    32: '480P',
    16: '360P',
  }
  return qualityMap[quality] || `${quality}P`
}

// 格式化音频质量显示
function formatAudioQuality(quality: number): string {
  const qualityMap: Record<number, string> = {
    30251: 'Hi-Res',
    30255: '杜比音效',
    30250: '杜比全景声',
    30280: '320K',
    30232: '132K',
    30216: '64K',
  }
  return qualityMap[quality] || `${quality}`
}

// 获取下载参数标签
function getDownloadTags(task: any): string[] {
  const tags: string[] = []

  if (task.config.videoOnly) {
    tags.push('仅视频')
  }
  else if (task.config.audioOnly) {
    tags.push('仅音频')
  }

  // 添加质量信息
  if (!task.config.audioOnly) {
    tags.push(formatVideoQuality(task.config.videoQuality))
  }
  if (!task.config.videoOnly) {
    tags.push(formatAudioQuality(task.config.audioQuality))
  }

  // 添加其他选项
  if (task.config.withDanmaku) {
    tags.push('弹幕')
  }
  if (task.config.withSubtitle) {
    tags.push('字幕')
  }
  if (task.config.withCover) {
    tags.push('封面')
  }

  return tags
}
</script>

<template>
  <div class="page-container">
    <div class="mb-4 flex items-center justify-between">
      <h2 class="text-2xl text-text-primary font-bold">
        下载队列
      </h2>
      <Button
        v-if="queueStore.completedTasks.length > 0"
        variant="secondary"
        @click="queueStore.clearCompleted"
      >
        清除已完成
      </Button>
    </div>

    <!-- 空状态 -->
    <Card v-if="!hasActiveTasks">
      <div class="text-center opacity-50 flex flex-col min-h-[60vh] items-center justify-center">
        <div class="mb-6 rounded-full bg-gray-50 flex h-32 w-32 items-center justify-center">
          <span class="text-5xl">🥡</span>
        </div>
        <h2 class="text-xl text-gray-500 font-bold">
          暂时没有正在下载的任务哦～
        </h2>
        <p class="text-sm text-gray-400 mt-2">
          快去主页粘贴个链接试试吧！
        </p>
      </div>
    </Card>

    <!-- 任务列表 -->
    <div v-else class="space-y-3">
      <Card
        v-for="task in allTasks"
        :key="task.id"
        class="transition-shadow hover:shadow-md"
      >
        <div class="flex gap-4">
          <!-- 缩略图 -->
          <img
            v-if="task.videoInfo?.thumbnail"
            :src="task.videoInfo.thumbnail"
            :alt="task.videoInfo.title"
            class="rounded-2xl h-16 w-24 object-cover"
            referrerpolicy="no-referrer"
          >
          <div v-else class="rounded-2xl bg-bg-tertiary flex h-16 w-24 items-center justify-center">
            <Video :size="24" class="text-text-tertiary" />
          </div>

          <!-- 任务信息 -->
          <div class="flex-1 min-w-0">
            <h3 class="text-base text-text-primary font-semibold mb-1 truncate">
              {{ task.videoInfo?.title || '未知视频' }}
            </h3>

            <!-- 下载参数标签 -->
            <div class="mb-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getDownloadTags(task)"
                :key="tag"
                class="text-xs text-text-secondary px-2 py-0.5 rounded bg-bg-tertiary"
              >
                {{ tag }}
              </span>
            </div>

            <div class="text-sm text-text-secondary mb-2 flex gap-4 items-center">
              <span :class="getStatusColor(task.status)">
                {{ getStatusText(task.status) }}
              </span>
              <span v-if="task.status === 'downloading'">
                {{ task.speed }}
              </span>
              <span v-if="task.status === 'downloading'">
                剩余 {{ task.eta }}
              </span>
            </div>
            <ProgressBar
              v-if="task.status !== 'completed'"
              :progress="task.progress"
              :show-label="true"
            />
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2 items-center">
            <Button
              v-if="task.status === 'downloading'"
              variant="secondary"
              @click="pauseDownload(task.id)"
            >
              <Pause :size="16" />
            </Button>
            <Button
              v-if="task.status === 'paused'"
              variant="secondary"
              @click="resumeDownload(task.id)"
            >
              <Play :size="16" />
            </Button>
            <Button
              v-if="task.status !== 'completed'"
              variant="secondary"
              @click="cancelDownload(task.id)"
            >
              <X :size="16" />
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
