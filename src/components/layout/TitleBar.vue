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
  <div class="px-6 border-b border-gray-50 bg-white flex select-none items-center relative">
    <div data-tauri-drag-region class="flex w-full items-center justify-between">
      <!-- App Branding -->
      <div class="flex gap-2 items-center relative z--1">
        <div class="text-white rounded-lg bg-teal-400 flex h-6 w-6 shadow-sm items-center justify-center">
          <Sparkles :size="14" />
        </div>
        <span class="text-sm text-teal-600 tracking-wide font-bold">yutto</span>
        <span class="text-[10px] bg-teal-50 text-teal-400 px-1.5 py-0.5 rounded font-bold uppercase">GUI</span>
      </div>

      <!-- Window Controls -->
      <div class="flex gap-0">
        <button
          class="text-gray-400 hover:text-gray-600 p-1 rounded-full hover:bg-gray-50 transition-colors border-none bg-transparent flex h-10 w-12 cursor-pointer items-center justify-center"
          aria-label="Minimize"
          @click="minimizeWindow"
        >
          <Minus :size="14" />
        </button>
        <button
          class="text-gray-400 hover:text-gray-600 p-1 rounded-full hover:bg-gray-50 transition-colors border-none bg-transparent flex h-10 w-12 cursor-pointer items-center justify-center"
          aria-label="Maximize"
          @click="toggleMaximize"
        >
          <Square :size="14" />
        </button>
        <button
          class="text-gray-400 hover:text-red-500 hover:bg-red-50 p-1 rounded-full transition-colors border-none bg-transparent flex h-10 w-12 cursor-pointer items-center justify-center"
          aria-label="Close"
          @click="closeWindow"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>
