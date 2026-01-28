import type { DownloadConfig, VideoInfo } from '@/types'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiService } from '@/services/api'
import { normalizeBilibiliUrl, validateBilibiliUrl } from '@/utils/validate'
import { useAuthStore } from './auth'
import { useSettingsStore } from './settings'

export const useDownloadStore = defineStore('download', () => {
  // State
  const currentUrl = ref<string>('')
  const currentConfig = ref<DownloadConfig | null>(null)
  const videoInfo = ref<VideoInfo | null>(null)
  const isLoading = ref<boolean>(false)
  const error = ref<string | null>(null)
  const selectedEpisodes = ref<Set<number>>(new Set())
  const episodeQualities = ref<Record<number, { videoQuality: number, audioQuality: number }>>({})

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

      // 自动全选所有剧集
      if (videoInfo.value?.episodes && videoInfo.value.episodes.length > 0) {
        selectedEpisodes.value = new Set(
          videoInfo.value.episodes.map(ep => ep.index),
        )
        console.log('[DownloadStore] Auto-selected all episodes:', selectedEpisodes.value)

        // 初始化每个剧集的质量配置
        const defaultVideoQuality = videoInfo.value.available_qualities?.find(q => q.available)?.quality || 80
        const defaultAudioQuality = videoInfo.value.available_audio_qualities?.find(q => q.available)?.quality || 30280

        episodeQualities.value = {}
        videoInfo.value.episodes.forEach((ep) => {
          episodeQualities.value[ep.index] = {
            videoQuality: defaultVideoQuality,
            audioQuality: defaultAudioQuality,
          }
        })
        console.log('[DownloadStore] Initialized episode qualities:', episodeQualities.value)
      }
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
    const yuttoCliPath = settingsStore.getSetting('yuttoCliPath')
    const normalizedYuttoPath = yuttoCliPath && yuttoCliPath.trim() !== '' ? yuttoCliPath : undefined

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
      yuttoCliPath: normalizedYuttoPath,
      withDanmaku: false,
      withSubtitle: false,
      withCover: false,
      withComments: false,
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
    selectedEpisodes.value.clear()
    episodeQualities.value = {}
  }

  function toggleEpisode(index: number) {
    if (selectedEpisodes.value.has(index)) {
      selectedEpisodes.value.delete(index)
    }
    else {
      selectedEpisodes.value.add(index)
    }
    // 触发响应式更新
    selectedEpisodes.value = new Set(selectedEpisodes.value)
    updateEpisodesConfig()
  }

  function selectAllEpisodes() {
    if (!videoInfo.value?.episodes)
      return
    selectedEpisodes.value = new Set(
      videoInfo.value.episodes.map(ep => ep.index),
    )
    updateEpisodesConfig()
  }

  function deselectAllEpisodes() {
    selectedEpisodes.value.clear()
    updateEpisodesConfig()
  }

  function updateEpisodesConfig() {
    if (!currentConfig.value)
      return
    const episodesStr = convertSelectedToYuttoFormat(selectedEpisodes.value)
    currentConfig.value.episodes = episodesStr || undefined
    currentConfig.value.episodeQualities = episodeQualities.value
  }

  function updateEpisodeQuality(index: number, videoQuality: number, audioQuality: number) {
    episodeQualities.value[index] = { videoQuality, audioQuality }
    updateEpisodesConfig()
  }

  function convertSelectedToYuttoFormat(selected: Set<number>): string {
    if (selected.size === 0)
      return ''

    const sorted = Array.from(selected).sort((a, b) => a - b)
    const ranges: string[] = []
    let rangeStart = sorted[0]
    let rangeEnd = sorted[0]

    for (let i = 1; i <= sorted.length; i++) {
      if (i < sorted.length && sorted[i] === rangeEnd + 1) {
        rangeEnd = sorted[i]
      }
      else {
        if (rangeStart === rangeEnd) {
          ranges.push(String(rangeStart))
        }
        else if (rangeEnd === rangeStart + 1) {
          ranges.push(String(rangeStart))
          ranges.push(String(rangeEnd))
        }
        else {
          ranges.push(`${rangeStart}~${rangeEnd}`)
        }
        if (i < sorted.length) {
          rangeStart = sorted[i]
          rangeEnd = sorted[i]
        }
      }
    }

    return ranges.join(',')
  }

  return {
    // State
    currentUrl,
    currentConfig,
    videoInfo,
    isLoading,
    error,
    selectedEpisodes,
    episodeQualities,
    // Actions
    fetchVideoInfo,
    updateConfig,
    initConfig,
    clearError,
    reset,
    toggleEpisode,
    selectAllEpisodes,
    deselectAllEpisodes,
    updateEpisodeQuality,
  }
})
