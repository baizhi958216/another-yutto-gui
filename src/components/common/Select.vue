<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps<{
  modelValue: string | number
  options: Array<{ label: string, value: string | number, disabled?: boolean }>
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const isOpen = ref(false)
const selectRef = ref<HTMLDivElement>()
const dropdownRef = ref<HTMLDivElement>()

const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue)
})

function toggleDropdown() {
  if (props.disabled)
    return
  isOpen.value = !isOpen.value
}

function selectOption(value: string | number) {
  if (props.options.find(opt => opt.value === value)?.disabled)
    return
  emit('update:modelValue', value)
  isOpen.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (selectRef.value && !selectRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div ref="selectRef" class="w-full relative">
    <!-- 触发器 -->
    <div
      class="px-4 py-2.5 border border-border-primary rounded-xl bg-bg-secondary flex w-full cursor-pointer shadow-sm transition-all items-center justify-between"
      :class="{
        'border-teal-500 ring-2 ring-teal-100': isOpen,
        'hover:border-teal-500 hover:ring-2 hover:ring-teal-100': !disabled,
        'opacity-50 cursor-not-allowed bg-bg-tertiary': disabled,
      }"
      @click="toggleDropdown"
    >
      <span class="text-sm text-text-primary flex-1">
        {{ selectedOption?.label || '请选择' }}
      </span>
      <div
        class="text-text-secondary flex h-5 w-5 transition-transform duration-200 items-center justify-center"
        :class="{ 'rotate-180': isOpen }"
      >
        <div class="i-carbon-chevron-down" />
      </div>
    </div>

    <!-- 下拉菜单 -->
    <Transition name="dropdown">
      <div
        v-if="isOpen"
        ref="dropdownRef"
        class="p-1 border border-border-primary rounded-xl bg-bg-secondary max-h-68 shadow-lg left-0 right-0 top-[calc(100%+0.5rem)] absolute z-1000 overflow-y-auto"
      >
        <div
          v-for="option in options"
          :key="option.value"
          class="text-sm text-text-primary my-2px px-3 py-2.5 rounded-lg flex cursor-pointer transition-all items-center justify-between"
          :class="{
            'bg-teal-50 text-teal-500 font-medium': option.value === modelValue,
            'hover:bg-teal-50 hover:text-teal-500': !option.disabled && option.value !== modelValue,
            'opacity-40 cursor-not-allowed text-text-tertiary': option.disabled,
          }"
          @click="selectOption(option.value)"
        >
          <span
            :class="{
              'text-teal-500': option.value === modelValue,
            }"
          >
            {{ option.label }}
          </span>
          <div v-if="option.value === modelValue" class="i-carbon-checkmark text-teal-500" />
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-0.5rem);
}

/* 自定义滚动条 */
.overflow-y-auto::-webkit-scrollbar {
  width: 0.375rem;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: var(--color-border-primary);
  border-radius: 0.25rem;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-secondary);
}
</style>
