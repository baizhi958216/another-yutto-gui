<script lang="ts" setup>
import { AlertTriangle, CheckCircle2, Info, XCircle } from 'lucide-vue-next'
import { computed } from 'vue'

const props = defineProps<{
  message: string
  type?: 'success' | 'error' | 'info' | 'warning'
}>()

const iconComponent = computed(() => {
  switch (props.type) {
    case 'success':
      return CheckCircle2
    case 'error':
      return XCircle
    case 'warning':
      return AlertTriangle
    default:
      return Info
  }
})

const iconColor = computed(() => {
  switch (props.type) {
    case 'success':
      return 'text-teal-500'
    case 'error':
      return 'text-error'
    case 'warning':
      return 'text-warning'
    default:
      return 'text-info'
  }
})
</script>

<template>
  <div class="p-4 rounded-2xl bg-white flex gap-3 max-w-md min-w-80 shadow-lg items-center">
    <component :is="iconComponent" :size="24" :class="iconColor" />
    <p class="text-text-dark flex-1">
      {{ message }}
    </p>
  </div>
</template>
