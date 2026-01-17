<script lang="ts" setup>
import { computed, watch } from 'vue'

const props = defineProps<{
  progress: number
  showLabel?: boolean
}>()

const progressPercent = computed(() => Math.min(100, Math.max(0, props.progress)))

// 添加调试日志
watch(() => props.progress, (newVal, oldVal) => {
  if (newVal !== oldVal) {
    console.log(`[ProgressBar] 进度变化: ${oldVal?.toFixed(1)}% → ${newVal?.toFixed(1)}%`)
  }
})
</script>

<template>
  <div class="w-full">
    <div v-if="showLabel" class="mb-1 flex items-center justify-between">
      <span class="text-text-gray text-sm">{{ progressPercent.toFixed(1) }}%</span>
    </div>
    <div class="rounded-full bg-gray-100 h-2 w-full overflow-hidden">
      <div
        class="h-full transition-all duration-1000 ease-out from-teal-300 to-teal-500 bg-gradient-to-r"
        :style="{ width: `${progressPercent}%` }"
      />
    </div>
  </div>
</template>
