<script lang="ts" setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, ref } from 'vue'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)

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
  <div class="bg-#1e1e1e flex h-32px select-none items-center relative z-1000">
    <div data-tauri-drag-region class="flex w-full items-center justify-end">
      <div class="flex gap-0">
        <button
          class="text-16px text-#cccccc border-none bg-transparent flex h-32px w-46px cursor-pointer transition-background-color-150 items-center justify-center active:bg-white/15 hover:bg-white/10"
          aria-label="Minimize"
          @click="minimizeWindow"
        >
          <div class="i-carbon-minimize h-16px w-16px" />
        </button>
        <button
          class="text-16px text-#cccccc border-none bg-transparent flex h-32px w-46px cursor-pointer transition-background-color-150 items-center justify-center active:bg-white/15 hover:bg-white/10"
          aria-label="Maximize"
          @click="toggleMaximize"
        >
          <div v-if="!isMaximized" class="i-carbon-maximize h-16px w-16px" />
          <div v-else class="i-carbon-minimize h-16px w-16px" />
        </button>
        <button
          class="text-16px text-#cccccc border-none bg-transparent flex h-32px w-46px cursor-pointer transition-background-color-150 items-center justify-center hover:text-white active:bg-#c50f1f hover:bg-#e81123"
          aria-label="Close"
          @click="closeWindow"
        >
          <div class="i-carbon-close h-16px w-16px" />
        </button>
      </div>
    </div>
  </div>
</template>
