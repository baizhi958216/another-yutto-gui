<script lang="ts" setup>
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
    <div v-if="loading" class="i-carbon-loading mr-2 inline-block animate-spin" />
    <slot />
  </button>
</template>
