<script lang="ts" setup>
import { Download, History, ListOrdered, Moon, Settings, Sun } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'

const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()

const navItems = computed(() => [
  { name: 'download', label: '下载', icon: Download },
  { name: 'queue', label: '队列', icon: ListOrdered },
  { name: 'history', label: '历史', icon: History },
  { name: 'settings', label: '设置', icon: Settings },
])

const isActive = (name: string) => route.name === name

function navigateTo(name: string) {
  router.push({ name })
}

const isTransitioning = ref(false)

function handleThemeToggle(event?: MouseEvent) {
  if (isTransitioning.value) {
    return
  }

  if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) {
    toggleTheme()
    return
  }

  const x = typeof event?.clientX === 'number' ? event.clientX : window.innerWidth / 2
  const y = typeof event?.clientY === 'number' ? event.clientY : window.innerHeight / 2
  const maxX = Math.max(x, window.innerWidth - x)
  const maxY = Math.max(y, window.innerHeight - y)
  const radius = Math.hypot(maxX, maxY)
  const root = document.documentElement
  root.style.setProperty('--theme-transition-x', `${x}px`)
  root.style.setProperty('--theme-transition-y', `${y}px`)
  root.style.setProperty('--theme-transition-radius', `${radius}px`)

  const startViewTransition = (
    document as Document & {
      startViewTransition?: (callback: () => void) => { finished: Promise<void> }
    }
  ).startViewTransition

  if (startViewTransition) {
    isTransitioning.value = true
    const transition = startViewTransition.call(document, () => {
      toggleTheme()
    })
    transition.finished.finally(() => {
      isTransitioning.value = false
    })
    return
  }

  toggleTheme()
}
</script>

<template>
  <nav data-tauri-drag-region class="py-8 border-r border-border-primary bg-bg-secondary flex flex-col w-20 items-center md:w-64">
    <!-- Logo Section -->
    <div class="mb-8 flex flex-col gap-3 items-center">
      <div class="flex w-12 transition-all duration-300 items-center justify-center md:h-34 md:w-34 hover:rotate-6">
        <div class="text-teal-500 flex items-center justify-center">
          <img src="/logo.png" alt="logo" class="cursor-pointer" @click="router.push('/')">
        </div>
      </div>
      <!-- <div class="text-center hidden md:block">
        <h1 class="text-lg text-text-primary font-bold">
          yutto GUI
        </h1>
      </div> -->
    </div>

    <!-- Navigation Buttons -->
    <div class="px-3 flex flex-col gap-2 w-full">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="px-4 py-3.5 rounded-2xl flex gap-3 w-full transition-all duration-300 items-center"
        :class="[
          isActive(item.name)
            ? 'bg-teal-500 text-white shadow-lg shadow-teal-200'
            : 'text-text-secondary hover:bg-bg-tertiary hover:text-teal-600',
        ]"
        @click="navigateTo(item.name)"
      >
        <component :is="item.icon" :size="20" class="flex-shrink-0" />
        <span class="text-sm font-medium hidden md:block">{{ item.label }}</span>
      </button>
    </div>

    <div class="mt-auto px-3 pb-6 w-full">
      <button
        class="p-2.5 border border-border-primary rounded-2xl bg-bg-secondary flex flex-col gap-2 w-full transition-all hover:bg-bg-tertiary md:flex-row md:items-center md:justify-between"
        :aria-pressed="isDark"
        aria-label="Toggle theme"
        @click="handleThemeToggle"
      >
        <div class="flex gap-3 items-center">
          <div class="text-teal-600 border border-border-primary rounded-xl bg-bg-tertiary flex h-8 w-8 items-center justify-center">
            <Sun v-if="!isDark" :size="18" />
            <Moon v-else :size="18" />
          </div>
          <span class="text-sm text-text-secondary font-medium hidden md:block">主题</span>
        </div>
        <div
          class="rounded-full h-5 w-9 transition-colors relative"
          :class="isDark ? 'bg-teal-500' : 'bg-border-secondary'"
        >
          <span
            class="rounded-full bg-white h-4 w-4 transition-transform left-0.5 top-0.5 absolute"
            :class="isDark ? 'translate-x-4' : 'translate-x-0'"
          />
        </div>
      </button>
    </div>
  </nav>
</template>
