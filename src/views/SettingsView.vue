<script lang="ts" setup>
import { isTauri } from '@tauri-apps/api/core'
import { Folder, ShieldCheck, Zap } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import { useToast } from '@/composables/useToast'
import { openInBrowser, selectFile, selectFolder } from '@/services/tauri'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { showSuccess, showError, showInfo } = useToast()
const runningInTauri = isTauri()

const localSettings = ref({ ...settingsStore.settings })
const showLogoutConfirm = ref(false)
const showResetConfirm = ref(false)

async function handleSelectDownloadPath() {
  if (!runningInTauri) {
    showInfo('下载目录选择仅在桌面应用中可用')
    return
  }

  const path = await selectFolder()
  if (path) {
    localSettings.value.defaultDownloadPath = path
  }
}

async function handleSelectYuttoPath() {
  if (!runningInTauri) {
    showInfo('Yutto 路径选择仅在桌面应用中可用')
    return
  }

  const path = await selectFile()
  if (path) {
    localSettings.value.yuttoCliPath = path
  }
}

function handleSave() {
  const { theme: _theme, ...rest } = localSettings.value
  settingsStore.updateSettings(rest)
  showSuccess('设置已保存')
}

function handleReset() {
  showResetConfirm.value = true
}

function confirmReset() {
  settingsStore.resetSettings()
  localSettings.value = { ...settingsStore.settings }
  showSuccess('设置已重置')
  showResetConfirm.value = false
}

async function handleLogin() {
  if (!runningInTauri) {
    showInfo('Bilibili 登录仅在桌面应用中可用')
    return
  }

  try {
    await authStore.login()
  }
  catch (error) {
    showError('登录失败，请重试')
  }
}

async function handleLogout() {
  showLogoutConfirm.value = true
}

async function confirmLogout() {
  try {
    await authStore.logout()
    showSuccess('已退出登录')
    showLogoutConfirm.value = false
  }
  catch (error) {
    showError('退出登录失败')
  }
}

function handleOpenUrl(url: string) {
  openInBrowser(url)
}

onMounted(async () => {
  await authStore.loadSessdata()
})
</script>

<template>
  <div class="page-container">
    <div class="space-y-4">
      <!-- 下载设置 -->
      <Card>
        <div class="mb-4 flex gap-3 items-center">
          <div class="rounded-lg bg-teal-100 flex h-8 w-8 items-center justify-center">
            <Folder :size="18" class="text-teal-600" />
          </div>
          <h3 class="text-lg text-text-primary font-semibold">
            下载设置
          </h3>
        </div>
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
      <Card>
        <div class="mb-4 flex gap-3 items-center">
          <div class="rounded-lg bg-teal-100 flex h-8 w-8 items-center justify-center">
            <Zap :size="18" class="text-teal-600" />
          </div>
          <h3 class="text-lg text-text-primary font-semibold">
            Yutto CLI 设置
          </h3>
        </div>
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
      <Card>
        <div class="mb-4 flex gap-3 items-center">
          <div class="rounded-lg bg-teal-100 flex h-8 w-8 items-center justify-center">
            <ShieldCheck :size="18" class="text-teal-600" />
          </div>
          <h3 class="text-lg text-text-primary font-semibold">
            账号设置
          </h3>
        </div>
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
              <div class="flex gap-2 items-center">
                <span class="text-sm text-green-600 font-medium">✓ 已登录 Bilibili</span>
              </div>
              <div class="flex gap-2 items-center">
                <span class="text-sm text-text-secondary">大会员状态:</span>
                <span v-if="authStore.isVip" class="text-sm text-purple-600 font-medium">已开通</span>
                <span v-else class="text-sm text-text-tertiary">未开通</span>
              </div>
              <div>
                <label class="text-xs text-text-tertiary mb-1 block">
                  SESSDATA
                </label>
                <div class="text-xs text-text-secondary font-mono p-2 rounded bg-bg-secondary break-all">
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

      <!-- 语言设置 -->
      <Card>
        <h3 class="text-lg text-text-primary font-semibold mb-4">
          语言设置
        </h3>
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            界面语言
          </label>
          <Select
            v-model="localSettings.language"
            :options="[
              { label: '简体中文', value: 'zh-CN' },
              { label: 'English (开发中)', value: 'en-US' },
            ]"
          />
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

      <div class="flex gap-12">
        <div class="flex gap-2 w-fit cursor-pointer transition-all items-center justify-center hover:text-teal-500" @click="handleOpenUrl('https://github.com/yutto-dev/yutto')">
          <img src="/logo.png" alt="yutto" class="h-8 w-auto">
          Yutto
        </div>
        <div class="flex gap-2 w-fit cursor-pointer transition-all items-center justify-center hover:text-teal-500" @click="handleOpenUrl('https://github.com/baizhi958216/another-yutto-gui')">
          <div class="i-mdi:github text-8" />
          Another Yutto GUI
        </div>
      </div>
    </div>

    <!-- 确认对话框 -->
    <ConfirmDialog
      :show="showLogoutConfirm"
      title="退出登录"
      message="确定要退出登录吗？"
      confirm-text="退出"
      cancel-text="取消"
      variant="warning"
      @confirm="confirmLogout"
      @cancel="showLogoutConfirm = false"
    />

    <ConfirmDialog
      :show="showResetConfirm"
      title="重置设置"
      message="确定要重置所有设置吗？此操作不可撤销。"
      confirm-text="重置"
      cancel-text="取消"
      variant="danger"
      @confirm="confirmReset"
      @cancel="showResetConfirm = false"
    />
  </div>
</template>
