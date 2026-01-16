import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const useAuthStore = defineStore('auth', () => {
  // State
  const sessdata = ref<string | null>(null)
  const isLoggedIn = ref(false)
  const isValidating = ref(false)
  const isVip = ref(false)

  // Actions
  async function login() {
    try {
      // Open login window
      await invoke('open_login_window')

      // Listen for login-success event
      const unlisten = await listen<string>('login-success', async (event) => {
        sessdata.value = event.payload
        isLoggedIn.value = true
        // Check VIP status after login
        await checkVipStatus()
        unlisten()
      })
    } catch (error) {
      console.error('Failed to open login window:', error)
      throw error
    }
  }

  async function loadSessdata() {
    try {
      const result = await invoke<string | null>('get_sessdata')
      if (result) {
        sessdata.value = result
        isLoggedIn.value = true
        // Check VIP status after loading SESSDATA
        await checkVipStatus()
      } else {
        sessdata.value = null
        isLoggedIn.value = false
        isVip.value = false
      }
    } catch (error) {
      console.error('Failed to load SESSDATA:', error)
      sessdata.value = null
      isLoggedIn.value = false
      isVip.value = false
    }
  }

  async function validateAuth() {
    if (!sessdata.value) {
      isLoggedIn.value = false
      return false
    }

    isValidating.value = true
    try {
      const isValid = await invoke<boolean>('validate_sessdata', {
        sessdata: sessdata.value,
      })

      if (!isValid) {
        // Auto-logout if invalid
        await logout()
      }

      return isValid
    } catch (error) {
      console.error('Failed to validate SESSDATA:', error)
      return false
    } finally {
      isValidating.value = false
    }
  }

  async function logout() {
    try {
      await invoke('clear_auth')
      sessdata.value = null
      isLoggedIn.value = false
      isVip.value = false
    } catch (error) {
      console.error('Failed to logout:', error)
      throw error
    }
  }

  async function checkVipStatus() {
    if (!sessdata.value) {
      isVip.value = false
      return false
    }

    try {
      const vipStatus = await invoke<boolean>('check_vip_status', {
        sessdata: sessdata.value,
      })
      isVip.value = vipStatus
      return vipStatus
    } catch (error) {
      console.error('Failed to check VIP status:', error)
      isVip.value = false
      return false
    }
  }

  return {
    // State
    sessdata,
    isLoggedIn,
    isValidating,
    isVip,
    // Actions
    login,
    loadSessdata,
    validateAuth,
    logout,
    checkVipStatus,
  }
})
