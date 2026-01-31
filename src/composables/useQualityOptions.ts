import type { Ref } from 'vue'
import type { AudioQualityOption, QualityOption } from '@/types'
import { computed, unref } from 'vue'

interface QualityOptionWithDisplay extends QualityOption {
  displayText: string
}

interface AudioQualityOptionWithDisplay extends AudioQualityOption {
  displayText: string
}

/**
 * Composable for managing quality options display
 */
type MaybeRef<T> = T | Ref<T>

export function useQualityOptions(
  availableQualities: MaybeRef<QualityOption[] | undefined>,
  availableAudioQualities: MaybeRef<AudioQualityOption[] | undefined>,
) {
  // Helper function to generate display text for video quality
  function getQualityDisplayText(quality: QualityOption): string {
    if (quality.available) {
      return quality.description
    }
    if (quality.vip_only) {
      return `${quality.description} (需要大会员)`
    }
    if (quality.login_required) {
      return `${quality.description} (需要登录)`
    }
    return quality.description
  }

  // Helper function to generate display text for audio quality
  function getAudioQualityDisplayText(quality: AudioQualityOption): string {
    if (quality.available) {
      return quality.description
    }
    if (quality.vip_only) {
      return `${quality.description} (需要大会员)`
    }
    if (quality.login_required) {
      return `${quality.description} (需要登录)`
    }
    return quality.description
  }

  // Default quality options if not available from backend
  const defaultQualityOptions: QualityOptionWithDisplay[] = [
    { quality: 127, description: '8K 超高清', available: false, vip_only: true, login_required: false, displayText: '8K 超高清 (需要大会员)' },
    { quality: 126, description: '杜比视界', available: false, vip_only: true, login_required: false, displayText: '杜比视界 (需要大会员)' },
    { quality: 125, description: 'HDR 真彩', available: false, vip_only: true, login_required: false, displayText: 'HDR 真彩 (需要大会员)' },
    { quality: 120, description: '4K 超清', available: false, vip_only: true, login_required: false, displayText: '4K 超清 (需要大会员)' },
    { quality: 116, description: '1080P 60帧', available: false, vip_only: true, login_required: false, displayText: '1080P 60帧 (需要大会员)' },
    { quality: 112, description: '1080P 高码率', available: false, vip_only: true, login_required: false, displayText: '1080P 高码率 (需要大会员)' },
    { quality: 80, description: '1080P 高清', available: false, vip_only: false, login_required: true, displayText: '1080P 高清 (需要登录)' },
    { quality: 64, description: '720P 高清', available: false, vip_only: false, login_required: true, displayText: '720P 高清 (需要登录)' },
  ]

  // Default audio quality options if not available from backend
  const defaultAudioQualityOptions: AudioQualityOptionWithDisplay[] = [
    { quality: 30251, description: 'Hi-Res无损', available: false, vip_only: true, login_required: false, displayText: 'Hi-Res无损 (需要大会员)' },
    { quality: 30255, description: '杜比音效', available: false, vip_only: true, login_required: false, displayText: '杜比音效 (需要大会员)' },
    { quality: 30250, description: '杜比全景声', available: false, vip_only: true, login_required: false, displayText: '杜比全景声 (需要大会员)' },
    { quality: 30280, description: '320kbps', available: false, vip_only: false, login_required: true, displayText: '320kbps (需要登录)' },
    { quality: 30232, description: '132kbps', available: false, vip_only: false, login_required: true, displayText: '132kbps (需要登录)' },
    { quality: 30216, description: '64kbps', available: false, vip_only: false, login_required: true, displayText: '64kbps (需要登录)' },
  ]

  // Compute available quality options from videoInfo or use defaults
  const qualityOptions = computed<QualityOptionWithDisplay[]>(() => {
    const qualities = unref(availableQualities)
    if (qualities && qualities.length > 0) {
      const options = qualities.map(q => ({
        ...q,
        displayText: getQualityDisplayText(q),
      }))
      console.log('[useQualityOptions] Video quality options:', options)
      return options
    }
    return defaultQualityOptions
  })

  // Compute available audio quality options from videoInfo or use defaults
  const audioQualityOptions = computed<AudioQualityOptionWithDisplay[]>(() => {
    const qualities = unref(availableAudioQualities)
    if (qualities && qualities.length > 0) {
      const options = qualities.map(q => ({
        ...q,
        displayText: getAudioQualityDisplayText(q),
      }))
      console.log('[useQualityOptions] Audio quality options:', options)
      return options
    }
    return defaultAudioQualityOptions
  })

  return {
    qualityOptions,
    audioQualityOptions,
    getQualityDisplayText,
    getAudioQualityDisplayText,
  }
}
