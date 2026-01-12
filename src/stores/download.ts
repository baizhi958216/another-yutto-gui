import type { DownloadConfig, VideoInfo } from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiService } from '@/services/api'
import { validateBilibiliUrl, normalizeBilibiliUrl } from '@/utils/validate'
import { useSettingsStore } from './settings'

export const useDownloadStore = defineStore('download', () => {
  // State
  const currentUrl = ref<string>('')
  const currentConfig = ref<DownloadConfig | null>(null)
  const videoInfo = ref<VideoInfo | null>(null)
  const isLoading = ref<boolean>(false)
  const error = ref<string | null>(null)

  // Actions
  async function fetchVideoInfo(url: string) {
    if (!validateBilibiliUrl(url)) {
      error.value = '请输入有效的 B 站视频链接'
      return
    }

    // 规范化 URL（将 BV 号转换为完整 URL）
    const normalizedUrl = normalizeBilibiliUrl(url)

    isLoading.value = true
    error.value = null

    try {
      videoInfo.value = await ApiService.fetchVideoInfo(normalizedUrl)
      currentUrl.value = normalizedUrl
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : '获取视频信息失败'
      videoInfo.value = null
    }
    finally {
      isLoading.value = false
    }
  }

  function updateConfig(config: Partial<DownloadConfig>) {
    if (currentConfig.value) {
      currentConfig.value = { ...currentConfig.value, ...config }
    }
    else {
      currentConfig.value = config as DownloadConfig
    }
  }

  function initConfig(url: string) {
    const settingsStore = useSettingsStore()
    let defaultPath = settingsStore.getSetting('defaultDownloadPath')

    // Ensure we have a valid path - use current directory if empty
    if (!defaultPath || defaultPath.trim() === '') {
      defaultPath = './'
    }

    currentConfig.value = {
      url,
      videoQuality: 80,
      audioQuality: 30280,
      downloadPath: defaultPath,
      withDanmaku: true,
      withSubtitle: true,
      withCover: true,
      batch: false,
    }
  }

  function clearError() {
    error.value = null
  }

  function reset() {
    currentUrl.value = ''
    currentConfig.value = null
    videoInfo.value = null
    isLoading.value = false
    error.value = null
  }

  return {
    // State
    currentUrl,
    currentConfig,
    videoInfo,
    isLoading,
    error,
    // Actions
    fetchVideoInfo,
    updateConfig,
    initConfig,
    clearError,
    reset,
  }
})
