<script lang="ts" setup>
import { onMounted } from 'vue'
import AppHeader from '@/components/layout/AppHeader.vue'
import AppNav from '@/components/layout/AppNav.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

// Load auth state on app mount
onMounted(async () => {
  await authStore.loadSessdata()
})
</script>

<template>
  <div class="bg-bg-secondary flex flex-col h-screen">
    <!-- Custom Title Bar -->
    <TitleBar />

    <!-- Header -->
    <AppHeader />

    <!-- Navigation -->
    <AppNav />

    <!-- Main Content with Router View -->
    <main class="flex-1 overflow-auto">
      <RouterView v-slot="{ Component }">
        <Transition name="fade" mode="out-in">
          <component :is="Component" />
        </Transition>
      </RouterView>
    </main>
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
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
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
