<script lang="ts" setup>
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const navItems = computed(() => [
  { name: 'download', label: '下载', icon: 'i-carbon-download' },
  { name: 'queue', label: '队列', icon: 'i-carbon-list' },
  { name: 'history', label: '历史', icon: 'i-carbon-time' },
  { name: 'presets', label: '预设', icon: 'i-carbon-save' },
  { name: 'settings', label: '设置', icon: 'i-carbon-settings' },
])

const isActive = (name: string) => route.name === name

function navigateTo(name: string) {
  router.push({ name })
}
</script>

<template>
  <nav class="px-6 border-b border-gray-200 bg-white">
    <div class="flex gap-1">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="text-sm font-medium px-4 py-3 flex gap-2 transition-all duration-200 items-center relative hover:bg-primary-50"
        :class="[
          isActive(item.name)
            ? 'text-primary-500'
            : 'text-text-secondary hover:text-text-primary',
        ]"
        @click="navigateTo(item.name)"
      >
        <div class="text-lg" :class="[item.icon]" />
        <span>{{ item.label }}</span>
        <div
          v-if="isActive(item.name)"
          class="bg-primary-400 h-0.5 bottom-0 left-0 right-0 absolute"
        />
      </button>
    </div>
  </nav>
</template>
