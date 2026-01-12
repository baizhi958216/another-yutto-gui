import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'

/**
 * 主题组合式函数
 */
export function useTheme() {
  const settingsStore = useSettingsStore()

  const theme = computed(() => settingsStore.settings.theme)
  const isDark = computed(() => theme.value === 'dark')

  /**
   * 切换主题
   */
  function toggleTheme(): void {
    const newTheme = theme.value === 'light' ? 'dark' : 'light'
    settingsStore.updateSettings({ theme: newTheme })
  }

  /**
   * 设置主题
   */
  function setTheme(newTheme: 'light' | 'dark'): void {
    settingsStore.updateSettings({ theme: newTheme })
  }

  return {
    theme,
    isDark,
    toggleTheme,
    setTheme,
  }
}
