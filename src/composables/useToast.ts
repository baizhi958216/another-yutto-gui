import type { Toast } from '@/stores/ui'
import { useUIStore } from '@/stores/ui'

/**
 * Toast 通知组合式函数
 */
export function useToast() {
  const uiStore = useUIStore()

  /**
   * 显示 Toast 通知
   */
  function showToast(
    type: Toast['type'],
    message: string,
    duration?: number,
  ): string {
    return uiStore.showToast(type, message, duration)
  }

  /**
   * 显示成功通知
   */
  function showSuccess(message: string, duration?: number): string {
    return showToast('success', message, duration)
  }

  /**
   * 显示错误通知
   */
  function showError(message: string, duration?: number): string {
    return showToast('error', message, duration)
  }

  /**
   * 显示警告通知
   */
  function showWarning(message: string, duration?: number): string {
    return showToast('warning', message, duration)
  }

  /**
   * 显示信息通知
   */
  function showInfo(message: string, duration?: number): string {
    return showToast('info', message, duration)
  }

  /**
   * 隐藏指定 Toast
   */
  function hideToast(id: string): void {
    uiStore.hideToast(id)
  }

  /**
   * 清除所有 Toast
   */
  function clearToasts(): void {
    uiStore.clearToasts()
  }

  return {
    showToast,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    hideToast,
    clearToasts,
  }
}
