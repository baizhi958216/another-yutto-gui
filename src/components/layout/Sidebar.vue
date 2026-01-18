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
  <nav data-tauri-drag-region class="py-8 border-r border-gray-100 bg-[#F8FBFA] flex flex-col w-20 items-center md:w-64">
    <!-- Logo Section -->
    <div class="mb-8 flex flex-col gap-3 items-center">
      <div class="flex w-12 transition-all duration-300 items-center justify-center md:h-34 md:w-34 hover:rotate-6">
        <div class="text-teal-500 flex items-center justify-center">
          <img src="/logo.png" alt="logo" class="cursor-pointer">
        </div>
      </div>
      <!-- <div class="text-center hidden md:block">
        <h1 class="text-lg text-gray-800 font-bold">
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
            ? 'bg-teal-500 text-white shadow-lg shadow-teal-200/50'
            : 'text-gray-500 hover:bg-teal-50 hover:text-teal-600',
        ]"
        @click="navigateTo(item.name)"
      >
        <component :is="item.icon" :size="20" class="flex-shrink-0" />
        <span class="text-sm font-medium hidden md:block">{{ item.label }}</span>
      </button>
    </div>
  </nav>
</template>
