<script lang="ts" setup>
import { onMounted, watch } from 'vue'
import ToastContainer from '@/components/common/ToastContainer.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import { useTaskPolling } from '@/composables/useTaskPolling'
import { setMaxConcurrentDownloads } from '@/services/tauri'
import { useAuthStore } from '@/stores/auth'
import { useDownloadStore } from '@/stores/download'
import { useSettingsStore } from '@/stores/settings'

const authStore = useAuthStore()
const downloadStore = useDownloadStore()
const settingsStore = useSettingsStore()
useTaskPolling(1000)

watch(
  () => settingsStore.settings.theme,
  (theme) => {
    const root = document.documentElement
    root.setAttribute('data-theme', theme)
    root.classList.toggle('dark', theme === 'dark')
  },
  { immediate: true },
)

watch(
  () => settingsStore.settings.language,
  (language) => {
    document.documentElement.lang = language || 'zh-CN'
  },
  { immediate: true },
)

watch(
  () => settingsStore.settings.defaultDownloadPath,
  (defaultDownloadPath) => {
    if (!downloadStore.currentConfig) {
      return
    }
    const normalizedPath = defaultDownloadPath && defaultDownloadPath.trim() !== '' ? defaultDownloadPath : './'
    downloadStore.updateConfig({ downloadPath: normalizedPath })
  },
  { immediate: true },
)

watch(
  () => settingsStore.settings.yuttoCliPath,
  (yuttoCliPath) => {
    if (!downloadStore.currentConfig) {
      return
    }
    const normalizedYuttoPath = yuttoCliPath && yuttoCliPath.trim() !== '' ? yuttoCliPath : undefined
    downloadStore.updateConfig({ yuttoCliPath: normalizedYuttoPath })
  },
  { immediate: true },
)

watch(
  () => settingsStore.settings.maxConcurrentDownloads,
  (maxConcurrent) => {
    const parsed = Number(maxConcurrent)
    const normalized = Number.isFinite(parsed) ? Math.max(1, Math.round(parsed)) : 1
    void setMaxConcurrentDownloads(normalized).catch((error) => {
      console.error('Failed to apply max concurrent downloads:', error)
    })
  },
  { immediate: true },
)

// Load auth state on app mount
onMounted(async () => {
  await authStore.loadSessdata()
})
</script>

<template>
  <div class="text-text-primary bg-bg-primary flex h-screen overflow-hidden">
    <Sidebar />
    <div class="flex flex-1 flex-col min-w-0">
      <TitleBar />
      <main class="p-8 bg-#fff flex-1 relative overflow-x-hidden overflow-y-auto">
        <div class="mx-auto h-full max-w-4xl">
          <RouterView v-slot="{ Component }">
            <Transition name="fade" mode="out-in">
              <component :is="Component" />
            </Transition>
          </RouterView>
        </div>
      </main>
    </div>
    <ToastContainer />
  </div>
</template>

<style lang="scss">
:root {
  color-scheme: light;
  --color-bg-primary: #f8f9fa;
  --color-bg-secondary: #ffffff;
  --color-bg-tertiary: #f0f4f8;
  --color-text-primary: #2c3e50;
  --color-text-secondary: #6c757d;
  --color-text-tertiary: #adb5bd;
  --color-border-primary: #e2e8f0;
  --color-border-secondary: #cbd5e1;
}

:root[data-theme='dark'] {
  color-scheme: dark;
  --color-bg-primary: #0f172a;
  --color-bg-secondary: #111827;
  --color-bg-tertiary: #1f2937;
  --color-text-primary: #e5e7eb;
  --color-text-secondary: #cbd5e1;
  --color-text-tertiary: #94a3b8;
  --color-border-primary: #1f2937;
  --color-border-secondary: #334155;
}

* {
  border: none;
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Quicksand', sans-serif;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

/* 自定义滚动条样式 */
/* Webkit 浏览器 (Chrome, Edge, Safari) */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 10px;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 10px;
  transition: background 0.2s ease;
}

::-webkit-scrollbar-thumb:hover {
  background: #14b8a6;
}

::-webkit-scrollbar-thumb:active {
  background: #0d9488;
}

/* Firefox */
* {
  scrollbar-width: thin;
  scrollbar-color: #cbd5e1 transparent;
}

input[type='checkbox'] {
  appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid #cbd5e1;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

input[type='checkbox']:hover {
  border-color: #14b8a6;
}

input[type='checkbox']:checked {
  background: #14b8a6;
  border-color: #14b8a6;
}

input[type='checkbox']:checked::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

input[type='checkbox']:focus {
  outline: none;
  box-shadow: 0 0 0 3px rgba(20, 184, 166, 0.1);
}

input[type='checkbox']:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 页面过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
