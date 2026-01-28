<script lang="ts" setup>
import type { AudioQualityOption, Episode, QualityOption } from '@/types/video'
import { computed } from 'vue'
import { getQualityDisplayText } from '@/utils/quality'
import Select from './Select.vue'

const props = defineProps<{
  episodes: Episode[]
  selectedIndices: Set<number>
  globalVideoQualities?: QualityOption[]
  globalAudioQualities?: AudioQualityOption[]
  episodeQualities: Record<number, { videoQuality: number, audioQuality: number }>
  isFavorite?: boolean
}>()

const emit = defineEmits<{
  toggle: [index: number]
  selectAll: []
  deselectAll: []
  updateQuality: [index: number, videoQuality: number, audioQuality: number]
}>()

const allSelected = computed(() => {
  return props.episodes.length > 0
    && props.episodes.every(ep => props.selectedIndices.has(ep.index))
})

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${minutes}:${String(secs).padStart(2, '0')}`
}

function handleSelectAll() {
  if (allSelected.value) {
    emit('deselectAll')
  }
  else {
    emit('selectAll')
  }
}

function getVideoQualityOptions(episode: Episode): { label: string, value: number, disabled: boolean }[] {
  const qualities = episode.available_qualities || props.globalVideoQualities || []
  return qualities.map(q => ({
    label: getQualityDisplayText(q),
    value: q.quality,
    disabled: q.available === false,
  }))
}

function getAudioQualityOptions(episode: Episode): { label: string, value: number, disabled: boolean }[] {
  const qualities = episode.available_audio_qualities || props.globalAudioQualities || []
  return qualities.map(q => ({
    label: getQualityDisplayText(q),
    value: q.quality,
    disabled: q.available === false,
  }))
}

function updateVideoQuality(episode: Episode, quality: number) {
  const current = props.episodeQualities[episode.index] || { videoQuality: 80, audioQuality: 30280 }
  emit('updateQuality', episode.index, quality, current.audioQuality)
}

function updateAudioQuality(episode: Episode, quality: number) {
  const current = props.episodeQualities[episode.index] || { videoQuality: 80, audioQuality: 30280 }
  emit('updateQuality', episode.index, current.videoQuality, quality)
}
</script>

<template>
  <div class="space-y-3">
    <!-- 标题和全选按钮 -->
    <div class="flex items-center justify-between">
      <label class="text-sm text-text-primary font-medium">
        选择剧集 ({{ selectedIndices.size }}/{{ episodes.length }})
      </label>
      <button
        class="text-sm text-teal-500 transition-colors hover:text-teal-600"
        @click="handleSelectAll"
      >
        {{ allSelected ? '取消全选' : '全选' }}
      </button>
    </div>

    <!-- 剧集列表 -->
    <div class="p-2 border border-border-primary rounded-xl bg-bg-secondary max-h-[600px] overflow-y-auto">
      <div
        v-for="episode in episodes"
        :key="episode.index"
        class="mb-2 p-3 border border-border-primary rounded-lg bg-bg-primary transition-colors hover:bg-bg-tertiary"
      >
        <!-- 剧集标题行 -->
        <label class="flex gap-3 cursor-pointer items-center">
          <input
            type="checkbox"
            :checked="selectedIndices.has(episode.index)"
            class="text-teal-500 border-border-primary rounded flex-shrink-0 h-4 w-4 cursor-pointer focus:ring-2 focus:ring-teal-500 focus:ring-offset-0"
            @change="emit('toggle', episode.index)"
          >
          <div class="flex-1 min-w-0">
            <div class="flex gap-2 items-center">
              <span v-show="!isFavorite" class="text-sm text-text-secondary font-medium">
                第{{ episode.index }}话
              </span>
              <span class="text-sm text-text-primary truncate">
                {{ episode.title }}
              </span>
            </div>
            <div v-if="episode.duration > 0" class="text-xs text-text-tertiary mt-1">
              时长: {{ formatDuration(episode.duration) }}
            </div>
          </div>
        </label>

        <!-- 质量选择器 -->
        <div v-if="selectedIndices.has(episode.index)" class="mt-3 pl-7 flex gap-3 space-y-2">
          <div>
            <label class="text-xs text-text-secondary mb-1 block">视频质量</label>
            <Select
              class="min-w-200px"
              :model-value="episodeQualities[episode.index]?.videoQuality || 80"
              :options="getVideoQualityOptions(episode)"
              @update:model-value="(val) => updateVideoQuality(episode, Number(val))"
            />
          </div>
          <div>
            <label class="text-xs text-text-secondary mb-1 block">音频质量</label>
            <Select
              class="min-w-120px"
              :model-value="episodeQualities[episode.index]?.audioQuality || 30280"
              :options="getAudioQualityOptions(episode)"
              @update:model-value="(val) => updateAudioQuality(episode, Number(val))"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 选择提示 -->
    <p v-if="selectedIndices.size === 0" class="text-xs text-error">
      请至少选择一个剧集
    </p>
  </div>
</template>

<style scoped>
/* 自定义滚动条 */
.overflow-y-auto::-webkit-scrollbar {
  width: 0.375rem;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: var(--color-border-primary);
  border-radius: 0.25rem;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-secondary);
}
</style>
