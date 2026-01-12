import { defineStore } from 'pinia'
import { ref } from 'vue'
import { generateId } from '@/utils/helpers'

export interface Toast {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  message: string
  duration?: number
}

export interface ModalState {
  isOpen: boolean
  component: string | null
  props: any
}

export const useUIStore = defineStore('ui', () => {
  // State
  const toasts = ref<Toast[]>([])
  const modal = ref<ModalState>({
    isOpen: false,
    component: null,
    props: null,
  })

  // Toast Actions
  function showToast(type: Toast['type'], message: string, duration = 3000) {
    const id = generateId()
    const toast: Toast = { id, type, message, duration }
    toasts.value.push(toast)

    // 自动移除 toast
    if (duration > 0) {
      setTimeout(() => {
        hideToast(id)
      }, duration)
    }

    return id
  }

  function hideToast(id: string) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) {
      toasts.value.splice(index, 1)
    }
  }

  function clearToasts() {
    toasts.value = []
  }

  // Modal Actions
  function openModal(component: string, props: any = null) {
    modal.value = {
      isOpen: true,
      component,
      props,
    }
  }

  function closeModal() {
    modal.value = {
      isOpen: false,
      component: null,
      props: null,
    }
  }

  return {
    // State
    toasts,
    modal,
    // Actions
    showToast,
    hideToast,
    clearToasts,
    openModal,
    closeModal,
  }
})
