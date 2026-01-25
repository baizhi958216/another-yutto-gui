<script lang="ts" setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Square, X } from 'lucide-vue-next'
import { storeToRefs } from 'pinia'
import { computed, onMounted, ref } from 'vue'
import { useTitleBarStore } from '@/stores/titleBar'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const titleBarStore = useTitleBarStore()
const { branding } = storeToRefs(titleBarStore)

const visibleActions = computed(() =>
  branding.value.actions.filter(action => !action.hidden),
)

const showBranding = computed(() =>
  branding.value.visible && (branding.value.title || visibleActions.value.length > 0),
)

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()

  // Listen for window resize events
  await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

async function minimizeWindow() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  await appWindow.toggleMaximize()
}

async function closeWindow() {
  await appWindow.close()
}
</script>

<template>
  <div class="bg-bg-secondary flex select-none items-center relative">
    <div data-tauri-drag-region class="flex w-full items-center justify-between">
      <!-- App Branding -->
      <div data-tauri-drag-region class="px-3 flex flex-1 gap-3 min-w-0 items-center">
        <div v-if="showBranding" data-tauri-drag-region class="flex gap-3 min-w-0 items-center">
          <div v-if="visibleActions.length" class="flex gap-2 items-center">
            <button
              v-for="action in visibleActions"
              :key="action.id"
              class="text-xs text-text-secondary font-semibold p-1 border border-border-primary rounded-full bg-bg-secondary transition-colors hover:text-text-primary hover:border-teal-500 hover:bg-teal-50 disabled:opacity-50 disabled:cursor-not-allowed"
              :aria-label="action.label"
              :disabled="action.disabled"
              @click="action.onClick?.()"
              v-html="action.label"
            />
          </div>
          <div class="flex gap-2 min-w-0 items-center">
            <span data-tauri-drag-region class="text-sm text-text-primary font-semibold truncate">
              {{ branding.title }}
            </span>
          </div>
        </div>
      </div>

      <!-- Window Controls -->
      <div class="flex gap-0">
        <button
          class="text-text-tertiary p-1 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-colors items-center justify-center hover:text-text-secondary hover:bg-bg-tertiary"
          aria-label="Minimize"
          @click="minimizeWindow"
        >
          <Minus :size="14" />
        </button>
        <button
          class="text-text-tertiary p-1 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-colors items-center justify-center hover:text-text-secondary hover:bg-bg-tertiary"
          aria-label="Maximize"
          @click="toggleMaximize"
        >
          <Square :size="14" />
        </button>
        <button
          class="text-text-tertiary p-1 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-colors items-center justify-center hover:text-red-500 hover:bg-red-50"
          aria-label="Close"
          @click="closeWindow"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>
