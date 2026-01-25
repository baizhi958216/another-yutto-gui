import { defineStore } from 'pinia'
import { ref } from 'vue'

export type TitleBarActionHandler = () => void | Promise<void>

export interface TitleBarAction {
  id: string
  label: string
  onClick?: TitleBarActionHandler
  disabled?: boolean
  hidden?: boolean
}

export interface TitleBarBranding {
  visible: boolean
  title: string
  actions: TitleBarAction[]
}

const DEFAULT_BRANDING: TitleBarBranding = {
  visible: false,
  title: '',
  actions: [],
}

export const useTitleBarStore = defineStore('titleBar', () => {
  const branding = ref<TitleBarBranding>({
    ...DEFAULT_BRANDING,
    actions: [],
  })

  function setBranding(update: Partial<TitleBarBranding>) {
    branding.value = {
      ...branding.value,
      ...update,
    }

    if (update.visible === undefined) {
      branding.value.visible = true
    }
  }

  function hideBranding() {
    branding.value.visible = false
  }

  function resetBranding() {
    branding.value = {
      ...DEFAULT_BRANDING,
      actions: [],
    }
  }

  return {
    branding,
    setBranding,
    hideBranding,
    resetBranding,
  }
})
