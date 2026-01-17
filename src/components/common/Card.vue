<script lang="ts" setup>
defineProps<{
  title?: string
  hoverable?: boolean
}>()
</script>

<template>
  <div class="card" :class="[hoverable ? 'hover:shadow-lg transition-all cursor-pointer' : '']">
    <h3 v-if="title" class="text-text-dark text-lg font-semibold mb-3">
      {{ title }}
    </h3>
    <slot />
  </div>
</template>
