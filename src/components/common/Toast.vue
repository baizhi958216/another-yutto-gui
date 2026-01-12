<script lang="ts" setup>
import { computed } from 'vue'

const props = defineProps<{
  message: string
  type?: 'success' | 'error' | 'info' | 'warning'
}>()

const iconClass = computed(() => {
  switch (props.type) {
    case 'success':
      return 'i-carbon-checkmark-filled text-success'
    case 'error':
      return 'i-carbon-close-filled text-error'
    case 'warning':
      return 'i-carbon-warning-filled text-warning'
    default:
      return 'i-carbon-information-filled text-info'
  }
})
</script>

<template>
  <div class="p-4 rounded-lg bg-white flex gap-3 max-w-md min-w-80 shadow-lg items-center">
    <div class="text-2xl" :class="[iconClass]" />
    <p class="text-text-dark flex-1">
      {{ message }}
    </p>
  </div>
</template>
