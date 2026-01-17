<script lang="ts" setup>
import { Loader2 } from 'lucide-vue-next'

defineProps<{
  variant?: 'primary' | 'secondary'
  disabled?: boolean
  loading?: boolean
}>()

defineEmits<{
  click: []
}>()
</script>

<template>
  <button
    :class="[
      variant === 'secondary' ? 'btn-secondary' : 'btn-primary',
      disabled || loading ? 'opacity-50 cursor-not-allowed' : '',
    ]"
    :disabled="disabled || loading"
    @click="$emit('click')"
  >
    <Loader2 v-if="loading" :size="20" class="mr-2 inline-block animate-spin" />
    <slot />
  </button>
</template>
