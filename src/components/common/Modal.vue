<script lang="ts" setup>
import { X } from 'lucide-vue-next'

defineProps<{
  show: boolean
  title?: string
}>()

defineEmits<{
  close: []
}>()
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="flex items-center inset-0 justify-center fixed z-50">
        <div class="bg-black bg-opacity-50 inset-0 absolute" @click="$emit('close')" />
        <div class="mx-4 rounded-3xl bg-white max-h-90vh max-w-2xl w-full shadow-lg relative overflow-auto">
          <div class="p-4 border-b border-gray-200 flex items-center justify-between">
            <h2 class="text-text-dark text-xl font-semibold">
              {{ title }}
            </h2>
            <button class="text-text-gray hover:text-text-dark transition-colors" @click="$emit('close')">
              <X :size="20" />
            </button>
          </div>
          <div class="p-4">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
