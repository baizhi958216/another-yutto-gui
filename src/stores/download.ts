import type { DownloadConfig, VideoInfo } from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiService } from '@/services/api'
import { normalizeBilibiliUrl, validateBilibiliUrl } from '@/utils/validate'
import { useSettingsStore } from './settings'
import { useAuthStore } from './auth'

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
      const authStore = useAuthStore()
      // 确保 VIP 状态是最新的
      if (authStore.sessdata && !authStore.isVip) {
        console.log('[DownloadStore] Checking VIP status before fetching video info')
        await authStore.checkVipStatus()
      }
      console.log('[DownloadStore] Fetching video info with isVip:', authStore.isVip, 'sessdata present:', !!authStore.sessdata)
      videoInfo.value = await ApiService.fetchVideoInfo(normalizedUrl, authStore.isVip, authStore.sessdata || undefined)
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
    const authStore = useAuthStore()
    let defaultPath = settingsStore.getSetting('defaultDownloadPath')

    // Ensure we have a valid path - use current directory if empty
    if (!defaultPath || defaultPath.trim() === '') {
      defaultPath = './'
    }

    // 选择可用的最高视频质量
    let defaultVideoQuality = 80 // 默认1080P
    if (videoInfo.value?.available_qualities && videoInfo.value.available_qualities.length > 0) {
      // 找到第一个可用的质量（列表已按质量从高到低排序）
      const firstAvailable = videoInfo.value.available_qualities.find(q => q.available)
      if (firstAvailable) {
        defaultVideoQuality = firstAvailable.quality
      }
    }

    // 选择可用的最高音频质量
    let defaultAudioQuality = 30280 // 默认320kbps
    if (videoInfo.value?.available_audio_qualities && videoInfo.value.available_audio_qualities.length > 0) {
      // 找到第一个可用的质量（列表已按质量从高到低排序）
      const firstAvailable = videoInfo.value.available_audio_qualities.find(q => q.available)
      if (firstAvailable) {
        defaultAudioQuality = firstAvailable.quality
      }
    }

    currentConfig.value = {
      url,
      videoQuality: defaultVideoQuality,
      audioQuality: defaultAudioQuality,
      downloadPath: defaultPath,
      withDanmaku: true,
      withSubtitle: true,
      withCover: true,
      batch: false,
      videoOnly: false,
      audioOnly: false,
      sessdata: authStore.sessdata || undefined,
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
