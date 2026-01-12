import type { AppSettings } from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'

const DEFAULT_SETTINGS: AppSettings = {
  defaultDownloadPath: '',
  theme: 'light',
  language: 'zh-CN',
  yuttoCliPath: '',
  maxConcurrentDownloads: 3,
}

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS })

  // Actions
  function updateSettings(updates: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...updates }
  }

  function resetSettings() {
    settings.value = { ...DEFAULT_SETTINGS }
  }

  function getSetting<K extends keyof AppSettings>(key: K): AppSettings[K] {
    return settings.value[key]
  }

  return {
    // State
    settings,
    // Actions
    updateSettings,
    resetSettings,
    getSetting,
  }
}, {
  persist: true,
})
