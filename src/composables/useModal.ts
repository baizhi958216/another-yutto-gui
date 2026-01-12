import { computed } from 'vue'
import { useUIStore } from '@/stores/ui'

/**
 * Modal 对话框组合式函数
 */
export function useModal() {
  const uiStore = useUIStore()

  const isOpen = computed(() => uiStore.modal.isOpen)
  const component = computed(() => uiStore.modal.component)
  const props = computed(() => uiStore.modal.props)

  /**
   * 打开 Modal
   */
  function openModal(modalComponent: string, modalProps: any = null): void {
    uiStore.openModal(modalComponent, modalProps)
  }

  /**
   * 关闭 Modal
   */
  function closeModal(): void {
    uiStore.closeModal()
  }

  return {
    isOpen,
    component,
    props,
    openModal,
    closeModal,
  }
}
