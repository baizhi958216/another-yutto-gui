<script lang="ts" setup>
import { useUIStore } from '@/stores/ui'
import Toast from './Toast.vue'

const uiStore = useUIStore()
</script>

<template>
  <div class="flex flex-col gap-3 right-6 top-20 fixed z-50">
    <TransitionGroup name="toast">
      <Toast
        v-for="toast in uiStore.toasts"
        :key="toast.id"
        :message="toast.message"
        :type="toast.type"
      />
    </TransitionGroup>
  </div>
</template>

<style scoped>
/* Toast 进入动画 */
.toast-enter-active {
  transition: all 0.3s ease;
}

.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100px) scale(0.8);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
