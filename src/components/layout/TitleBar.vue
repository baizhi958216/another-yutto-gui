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
  <div class="bg-bg-secondary flex select-none items-center relative">
    <div data-tauri-drag-region class="flex w-full items-center justify-between">
      <!-- App Branding -->
      <div class="flex gap-2 items-center relative z--1">
        <div class="text-white rounded-lg bg-teal-400 flex h-6 w-6 shadow-sm items-center justify-center">
          <Sparkles :size="14" />
        </div>
        <span class="text-sm text-teal-600 tracking-wide font-bold">yutto</span>
        <span class="text-[10px] text-teal-400 font-bold px-1.5 py-0.5 rounded bg-teal-50 uppercase">GUI</span>
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
