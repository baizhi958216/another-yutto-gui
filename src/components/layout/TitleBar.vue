<script lang="ts" setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Sparkles, Square, X } from 'lucide-vue-next'
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
  <div class="pl-3 border-b border-gray-50 bg-#f5f6fa flex select-none items-center relative z-1000">
    <div data-tauri-drag-region class="flex w-full items-center justify-between">
      <!-- App Branding -->
      <div class="flex gap-2 items-center relative z--1">
        <div class="text-white rounded-lg bg-teal-400 flex h-6 w-6 shadow-sm items-center justify-center">
          <Sparkles :size="14" />
        </div>
        <span class="text-sm text-teal-600 tracking-wide font-bold">yutto</span>
      </div>

      <!-- Window Controls -->
      <div class="flex gap-0">
        <button
          class="text-gray-600 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-all items-center justify-center active:bg-gray-200 hover:bg-gray-100"
          aria-label="Minimize"
          @click="minimizeWindow"
        >
          <Minus :size="14" />
        </button>
        <button
          class="text-gray-600 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-all items-center justify-center active:bg-gray-200 hover:bg-gray-100"
          aria-label="Maximize"
          @click="toggleMaximize"
        >
          <Square :size="14" />
        </button>
        <button
          class="text-gray-600 border-none bg-transparent flex h-10 w-12 cursor-pointer transition-all items-center justify-center hover:text-white active:bg-red-600 hover:bg-red-500"
          aria-label="Close"
          @click="closeWindow"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>
