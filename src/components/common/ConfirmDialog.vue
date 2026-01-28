<script lang="ts" setup>
import Button from './Button.vue'

defineProps<{
  show: boolean
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'danger' | 'warning' | 'info'
}>()

defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="flex items-center inset-0 justify-center fixed z-50">
        <div class="bg-black bg-opacity-50 inset-0 absolute" @click="$emit('cancel')" />
        <div class="mx-4 rounded-2xl bg-bg-secondary max-w-md w-full shadow-xl relative">
          <div class="p-6">
            <div class="mb-4 flex gap-4 items-start">
              <div class="flex-1">
                <h3 v-if="title" class="text-lg text-text-primary font-semibold mb-2">
                  {{ title }}
                </h3>
                <p class="text-sm text-text-secondary">
                  {{ message }}
                </p>
              </div>
            </div>
            <div class="flex gap-3 justify-end">
              <Button variant="secondary" @click="$emit('cancel')">
                {{ cancelText || '取消' }}
              </Button>
              <Button variant="primary" @click="$emit('confirm')">
                {{ confirmText || '确定' }}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .rounded-2xl,
.modal-leave-active .rounded-2xl {
  transition: transform 0.2s ease;
}

.modal-enter-from .rounded-2xl,
.modal-leave-to .rounded-2xl {
  transform: scale(0.95);
}
</style>
