<script lang="ts" setup>
import { onMounted } from 'vue'
import ToastContainer from '@/components/common/ToastContainer.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

// Load auth state on app mount
onMounted(async () => {
  await authStore.loadSessdata()
})
</script>

<template>
  <div class="bg-white flex h-screen overflow-hidden">
    <Sidebar />
    <div class="flex flex-1 flex-col min-w-0">
      <TitleBar />
      <main class="p-8 flex-1 relative overflow-x-hidden overflow-y-auto">
        <div class="mx-auto h-full max-w-4xl">
          <RouterView v-slot="{ Component }">
            <Transition name="fade" mode="out-in">
              <component :is="Component" />
            </Transition>
          </RouterView>
        </div>
      </main>
    </div>
    <ToastContainer />
  </div>
</template>

<style lang="scss">
* {
  border: none;
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Quicksand', sans-serif;
}

/* 自定义滚动条样式 */
/* Webkit 浏览器 (Chrome, Edge, Safari) */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 10px;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 10px;
  transition: background 0.2s ease;
}

::-webkit-scrollbar-thumb:hover {
  background: #14b8a6;
}

::-webkit-scrollbar-thumb:active {
  background: #0d9488;
}

/* Firefox */
* {
  scrollbar-width: thin;
  scrollbar-color: #cbd5e1 transparent;
}

input[type='checkbox'] {
  appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid #cbd5e1;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

input[type='checkbox']:hover {
  border-color: #14b8a6;
}

input[type='checkbox']:checked {
  background: #14b8a6;
  border-color: #14b8a6;
}

input[type='checkbox']:checked::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

input[type='checkbox']:focus {
  outline: none;
  box-shadow: 0 0 0 3px rgba(20, 184, 166, 0.1);
}

input[type='checkbox']:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 页面过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
