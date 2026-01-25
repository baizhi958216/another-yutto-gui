import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/download',
    },
    {
      path: '/download',
      name: 'download',
      component: () => import('@/views/DownloadView.vue'),
      meta: {
        title: '下载',
        icon: 'i-carbon-download',
      },
    },
    {
      path: '/queue',
      name: 'queue',
      component: () => import('@/views/QueueView.vue'),
      meta: {
        title: '队列',
        icon: 'i-carbon-list',
      },
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('@/views/HistoryView.vue'),
      meta: {
        title: '历史',
        icon: 'i-carbon-time',
      },
    },
    {
      path: '/preview/:id',
      name: 'preview',
      component: () => import('@/views/VideoPreviewView.vue'),
      meta: {
        title: '视频预览',
        icon: 'i-carbon-play',
      },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: {
        title: '设置',
        icon: 'i-carbon-settings',
      },
    },
  ],
})

export default router
