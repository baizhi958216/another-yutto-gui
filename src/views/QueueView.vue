<script lang="ts" setup>
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import ProgressBar from '@/components/common/ProgressBar.vue'
import { useDownload } from '@/composables/useDownload'
import { useQueueStore } from '@/stores/queue'
import { useTaskPolling } from '@/composables/useTaskPolling'

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
    downloading: 'text-primary-500',
    paused: 'text-warning',
    completed: 'text-success',
    error: 'text-error',
  }
  return colorMap[status] || 'text-text-secondary'
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
      <div class="py-12 text-center">
        <div class="i-carbon-list text-6xl text-text-tertiary mx-auto mb-4" />
        <p class="text-text-secondary">
          暂无下载任务
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
            class="rounded-lg h-16 w-24 object-cover"
            referrerpolicy="no-referrer"
          >
          <div v-else class="rounded-lg bg-bg-tertiary flex h-16 w-24 items-center justify-center">
            <div class="i-carbon-video text-2xl text-text-tertiary" />
          </div>

          <!-- 任务信息 -->
          <div class="flex-1 min-w-0">
            <h3 class="text-base text-text-primary font-semibold mb-1 truncate">
              {{ task.videoInfo?.title || '未知视频' }}
            </h3>
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
              <div class="i-carbon-pause" />
            </Button>
            <Button
              v-if="task.status === 'paused'"
              variant="secondary"
              @click="resumeDownload(task.id)"
            >
              <div class="i-carbon-play" />
            </Button>
            <Button
              v-if="task.status !== 'completed'"
              variant="secondary"
              @click="cancelDownload(task.id)"
            >
              <div class="i-carbon-close" />
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
