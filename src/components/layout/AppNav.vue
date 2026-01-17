<script lang="ts" setup>
import { Download, History, ListOrdered, Settings } from 'lucide-vue-next'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

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
</script>

<template>
  <nav class="px-6 border-b border-gray-200 bg-white">
    <div class="flex gap-1">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="text-sm font-medium px-4 py-3 flex gap-2 transition-all duration-200 items-center relative hover:bg-teal-50"
        :class="[
          isActive(item.name)
            ? 'text-teal-500'
            : 'text-text-secondary hover:text-text-primary',
        ]"
        @click="navigateTo(item.name)"
      >
        <component :is="item.icon" :size="18" />
        <span>{{ item.label }}</span>
        <div
          v-if="isActive(item.name)"
          class="bg-teal-400 h-0.5 bottom-0 left-0 right-0 absolute"
        />
      </button>
    </div>
  </nav>
</template>
