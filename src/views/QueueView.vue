<script lang="ts" setup>
import { Pause, Play, Video, X } from 'lucide-vue-next'
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import ProgressBar from '@/components/common/ProgressBar.vue'
import { useDownload } from '@/composables/useDownload'
import { useQueueStore } from '@/stores/queue'
import { formatDownloadProgress, formatEta, formatSpeed, generateDownloadTags } from '@/utils/format'

const queueStore = useQueueStore()
const { pauseDownload, resumeDownload, cancelDownload } = useDownload()

const allTasks = computed(() => queueStore.tasks)
const hasActiveTasks = computed(() => allTasks.value.length > 0)

function getStatusText(status: string, task: any) {
  // If downloading comments, show special status
  if (task.isDownloadingComments) {
    return '下载评论中'
  }

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

// 获取下载参数标签
const getDownloadTags = (task: any) => generateDownloadTags(task.config)

// 获取下载进度信息（已下载/总大小）
function getDownloadInfo(task: any): string {
  if (task.downloadedBytes && task.totalBytes) {
    return formatDownloadProgress(task.downloadedBytes, task.totalBytes)
  }
  // 回退到旧的 totalSize 字段
  if (task.totalSize > 0) {
    const downloaded = (task.progress / 100) * task.totalSize
    return formatDownloadProgress(downloaded, task.totalSize)
  }
  return ''
}

// 获取速度信息
function getSpeedInfo(task: any): string {
  // 优先使用新的精确速度字段
  if (task.speedBytesPerSec !== undefined && task.speedBytesPerSec > 0) {
    return formatSpeed(task.speedBytesPerSec)
  }
  // 回退到旧的字符串速度字段
  return task.speed || '0 B/s'
}

// 获取 ETA 信息
function getEtaInfo(task: any): string {
  // 优先使用新的精确 ETA 字段
  if (task.etaSeconds !== undefined && task.etaSeconds !== null) {
    return formatEta(task.etaSeconds)
  }
  // 回退到旧的字符串 ETA 字段
  return task.eta || '--:--'
}
</script>

<template>
  <div class="page-container">
    <div class="mb-4 flex items-center justify-end">
      <Button
        v-if="queueStore.completedTasks.length > 0"
        variant="secondary"
        @click="queueStore.clearCompleted"
      >
        清除已完成
      </Button>
    </div>

    <!-- 空状态 -->
    <div v-if="!hasActiveTasks">
      <div class="text-center opacity-50 flex flex-col min-h-[60vh] items-center justify-center">
        <div class="mb-6 rounded-full bg-bg-tertiary flex h-32 w-32 items-center justify-center">
          <span class="text-5xl">🥡</span>
        </div>
        <h2 class="text-xl text-text-secondary font-bold">
          暂时没有正在下载的任务哦～
        </h2>
        <p class="text-sm text-text-tertiary mt-2">
          快去主页粘贴个链接试试吧！
        </p>
      </div>
    </div>

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
            class="rounded h-16 w-24 object-cover"
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

            <div class="text-xs text-text-secondary mb-2 flex flex-wrap gap-4 items-center">
              <span :class="getStatusColor(task.status)">
                {{ getStatusText(task.status, task) }}
              </span>

              <!-- 显示下载大小信息 -->
              <span v-if="task.status === 'downloading' && !task.isDownloadingComments && getDownloadInfo(task)">
                {{ getDownloadInfo(task) }}
              </span>

              <!-- Show paused progress if available -->
              <span v-if="task.status === 'paused' && task.pausedAtProgress !== undefined">
                {{ task.pausedAtProgress.toFixed(1) }}%
              </span>
              <span v-if="task.status === 'paused' && task.pausedAtSpeed">
                {{ task.pausedAtSpeed }}
              </span>

              <!-- Show current progress for downloading tasks -->
              <span v-else-if="task.isDownloadingComments && task.commentDownloadProgress">
                {{ task.commentDownloadProgress }}
              </span>
              <span v-else-if="task.status === 'downloading'">
                {{ getSpeedInfo(task) }}
              </span>
            </div>
            <div v-if="task.status === 'downloading' && !task.isDownloadingComments" class="text-xs text-text-secondary mb-2">
              剩余 {{ getEtaInfo(task) }}
            </div>
            <!-- 警告信息 -->
            <div v-if="task.warning" class="text-xs text-warning mb-2">
              {{ task.warning }}
            </div>
            <!-- 错误信息 -->
            <div v-if="task.error" class="text-xs text-error mb-2">
              {{ task.error }}
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
