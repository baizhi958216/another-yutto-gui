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
