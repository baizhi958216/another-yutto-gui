<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useToast } from '@/composables/useToast'
import { selectFile, selectFolder } from '@/services/tauri'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'

const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { showSuccess, showError } = useToast()

const localSettings = ref({ ...settingsStore.settings })

async function handleSelectDownloadPath() {
  const path = await selectFolder()
  if (path) {
    localSettings.value.defaultDownloadPath = path
  }
}

async function handleSelectYuttoPath() {
  const path = await selectFile()
  if (path) {
    localSettings.value.yuttoCliPath = path
  }
}

function handleSave() {
  settingsStore.updateSettings(localSettings.value)
  showSuccess('设置已保存')
}

function handleReset() {
  if (confirm('确定要重置所有设置吗？')) {
    settingsStore.resetSettings()
    localSettings.value = { ...settingsStore.settings }
    showSuccess('设置已重置')
  }
}

async function handleLogin() {
  try {
    await authStore.login()
  }
  catch (error) {
    showError('登录失败，请重试')
  }
}

async function handleLogout() {
  if (confirm('确定要退出登录吗？')) {
    try {
      await authStore.logout()
      showSuccess('已退出登录')
    }
    catch (error) {
      showError('退出登录失败')
    }
  }
}

onMounted(async () => {
  await authStore.loadSessdata()
})
</script>

<template>
  <div class="page-container">
    <h2 class="text-2xl text-text-primary font-bold mb-4">
      设置
    </h2>

    <div class="space-y-4">
      <!-- 下载设置 -->
      <Card title="下载设置">
        <div class="space-y-4">
          <div>
            <label class="text-sm text-text-primary font-medium mb-2 block">
              默认下载路径
            </label>
            <div class="flex gap-2">
              <Input
                v-model="localSettings.defaultDownloadPath"
                placeholder="选择默认下载文件夹"
                class="flex-1"
                readonly
              />
              <Button variant="secondary" @click="handleSelectDownloadPath">
                浏览
              </Button>
            </div>
          </div>

          <div>
            <label class="text-sm text-text-primary font-medium mb-2 block">
              最大并发下载数
            </label>
            <Input
              v-model.number="localSettings.maxConcurrentDownloads"
              type="number"
              min="1"
              max="10"
              placeholder="3"
            />
            <p class="text-xs text-text-tertiary mt-1">
              同时进行的最大下载任务数（1-10）
            </p>
          </div>
        </div>
      </Card>

      <!-- Yutto CLI 设置 -->
      <Card title="Yutto CLI 设置">
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            Yutto CLI 路径
          </label>
          <div class="flex gap-2">
            <Input
              v-model="localSettings.yuttoCliPath"
              placeholder="选择 yutto 可执行文件"
              class="flex-1"
              readonly
            />
            <Button variant="secondary" @click="handleSelectYuttoPath">
              浏览
            </Button>
          </div>
          <p class="text-xs text-text-tertiary mt-1">
            如果未设置，将使用系统 PATH 中的 yutto
          </p>
        </div>
      </Card>

      <!-- 账号设置 -->
      <Card title="账号设置">
        <div class="space-y-3">
          <p class="text-sm text-text-secondary">
            登录 Bilibili 账号以下载会员专享内容
          </p>

          <div v-if="!authStore.isLoggedIn">
            <Button variant="primary" @click="handleLogin">
              登录 Bilibili
            </Button>
          </div>

          <div v-else class="space-y-3">
            <div class="space-y-2">
              <div class="flex items-center gap-2">
                <span class="text-sm text-green-600 font-medium">✓ 已登录 Bilibili</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-sm text-text-secondary">大会员状态:</span>
                <span v-if="authStore.isVip" class="text-sm text-purple-600 font-medium">已开通</span>
                <span v-else class="text-sm text-text-tertiary">未开通</span>
              </div>
              <div>
                <label class="text-xs text-text-tertiary mb-1 block">
                  SESSDATA
                </label>
                <div class="bg-bg-secondary p-2 rounded text-xs font-mono text-text-secondary break-all">
                  {{ authStore.sessdata || '加载中...' }}
                </div>
              </div>
            </div>
            <div class="flex gap-2">
              <Button variant="primary" @click="handleLogin">
                重新登录
              </Button>
              <Button variant="secondary" @click="handleLogout">
                退出登录
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <!-- 外观设置 -->
      <Card title="外观设置">
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            主题
          </label>
          <select v-model="localSettings.theme" class="input-base">
            <option value="light">
              浅色
            </option>
            <option value="dark">
              深色（开发中）
            </option>
          </select>
        </div>
      </Card>

      <!-- 语言设置 -->
      <Card title="语言设置">
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            界面语言
          </label>
          <select v-model="localSettings.language" class="input-base">
            <option value="zh-CN">
              简体中文
            </option>
            <option value="en-US">
              English (开发中)
            </option>
          </select>
        </div>
      </Card>

      <!-- 操作按钮 -->
      <div class="flex gap-2 justify-end">
        <Button variant="secondary" @click="handleReset">
          重置设置
        </Button>
        <Button variant="primary" @click="handleSave">
          保存设置
        </Button>
      </div>
    </div>
  </div>
</template>
