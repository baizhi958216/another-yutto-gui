<script lang="ts" setup>
import { ref } from 'vue'

defineProps<{
  text: string
  position?: 'top' | 'bottom' | 'left' | 'right'
}>()

const showTooltip = ref(false)
</script>

<template>
  <div
    class="inline-block relative"
    @mouseenter="showTooltip = true"
    @mouseleave="showTooltip = false"
  >
    <slot />
    <div
      v-if="showTooltip"
      class="tooltip text-xs text-white px-2 py-1 rounded bg-gray-800 pointer-events-none whitespace-nowrap absolute z-50"
      :class="{
        'bottom-full left-1/2 -translate-x-1/2 mb-2': position === 'top' || !position,
        'top-full left-1/2 -translate-x-1/2 mt-2': position === 'bottom',
        'right-full top-1/2 -translate-y-1/2 mr-2': position === 'left',
        'left-full top-1/2 -translate-y-1/2 ml-2': position === 'right',
      }"
    >
      {{ text }}
      <div
        class="tooltip-arrow bg-gray-800 h-2 w-2 rotate-45 transform absolute"
        :class="{
          'bottom-0 left-1/2 -translate-x-1/2 translate-y-1': position === 'top' || !position,
          'top-0 left-1/2 -translate-x-1/2 -translate-y-1': position === 'bottom',
          'right-0 top-1/2 -translate-y-1/2 translate-x-1': position === 'left',
          'left-0 top-1/2 -translate-y-1/2 -translate-x-1': position === 'right',
        }"
      />
    </div>
  </div>
</template>

<style scoped>
.tooltip {
  animation: fadeIn 0.2s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
</style>
