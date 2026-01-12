import type { Preset } from '@/types'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { generateId } from '@/utils/helpers'

export const usePresetsStore = defineStore('presets', () => {
  // State
  const presets = ref<Preset[]>([])
  const activePresetId = ref<string | null>(null)

  // Getters
  const activePreset = computed(() =>
    presets.value.find(preset => preset.id === activePresetId.value) || null,
  )

  // Actions
  function createPreset(preset: Omit<Preset, 'id' | 'createdAt'>) {
    const newPreset: Preset = {
      ...preset,
      id: generateId(),
      createdAt: Date.now(),
    }
    presets.value.push(newPreset)
    return newPreset
  }

  function updatePreset(id: string, updates: Partial<Omit<Preset, 'id' | 'createdAt'>>) {
    const preset = presets.value.find(p => p.id === id)
    if (preset) {
      Object.assign(preset, updates)
    }
  }

  function deletePreset(id: string) {
    const index = presets.value.findIndex(preset => preset.id === id)
    if (index > -1) {
      presets.value.splice(index, 1)
      if (activePresetId.value === id) {
        activePresetId.value = null
      }
    }
  }

  function activatePreset(id: string) {
    const preset = presets.value.find(p => p.id === id)
    if (preset) {
      activePresetId.value = id
    }
  }

  function deactivatePreset() {
    activePresetId.value = null
  }

  // 初始化默认预设
  function initDefaultPresets() {
    if (presets.value.length === 0) {
      createPreset({
        name: '高清画质',
        description: '1080P 高清视频，适合收藏',
        config: {
          videoQuality: 80,
          audioQuality: 30280,
          withDanmaku: true,
          withSubtitle: true,
          withCover: true,
        },
      })

      createPreset({
        name: '标清画质',
        description: '720P 标清视频，节省空间',
        config: {
          videoQuality: 64,
          audioQuality: 30280,
          withDanmaku: false,
          withSubtitle: false,
          withCover: false,
        },
      })
    }
  }

  return {
    // State
    presets,
    activePresetId,
    // Getters
    activePreset,
    // Actions
    createPreset,
    updatePreset,
    deletePreset,
    activatePreset,
    deactivatePreset,
    initDefaultPresets,
  }
}, {
  persist: true,
})
