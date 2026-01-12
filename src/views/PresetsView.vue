<script lang="ts" setup>
import type { Preset } from '@/types'
import { ref } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useToast } from '@/composables/useToast'
import { useDownloadStore } from '@/stores/download'
import { usePresetsStore } from '@/stores/presets'

const presetsStore = usePresetsStore()
const downloadStore = useDownloadStore()
const { showSuccess } = useToast()

const isCreating = ref(false)
const editingPreset = ref<Preset | null>(null)

const newPreset = ref({
  name: '',
  description: '',
  videoQuality: 80,
  audioQuality: 30280,
  withDanmaku: true,
  withSubtitle: true,
  withCover: true,
})

function handleCreatePreset() {
  if (!newPreset.value.name) {
    return
  }

  presetsStore.createPreset({
    name: newPreset.value.name,
    description: newPreset.value.description,
    config: {
      videoQuality: newPreset.value.videoQuality,
      audioQuality: newPreset.value.audioQuality,
      withDanmaku: newPreset.value.withDanmaku,
      withSubtitle: newPreset.value.withSubtitle,
      withCover: newPreset.value.withCover,
    },
  })

  showSuccess('预设创建成功')
  isCreating.value = false
  resetForm()
}

function handleApplyPreset(preset: Preset) {
  if (downloadStore.currentConfig) {
    downloadStore.updateConfig(preset.config)
    presetsStore.activatePreset(preset.id)
    showSuccess(`已应用预设: ${preset.name}`)
  }
}

function handleDeletePreset(id: string) {
  if (confirm('确定要删除这个预设吗？')) {
    presetsStore.deletePreset(id)
    showSuccess('预设已删除')
  }
}

function resetForm() {
  newPreset.value = {
    name: '',
    description: '',
    videoQuality: 80,
    audioQuality: 30280,
    withDanmaku: true,
    withSubtitle: true,
    withCover: true,
  }
}

function getQualityLabel(quality: number): string {
  const qualityMap: Record<number, string> = {
    127: '8K 超高清',
    126: '杜比视界',
    125: 'HDR 真彩',
    120: '4K 超清',
    116: '1080P 60帧',
    112: '1080P 高码率',
    80: '1080P 高清',
    64: '720P 高清',
  }
  return qualityMap[quality] || `${quality}P`
}
</script>

<template>
  <div class="page-container">
    <div class="mb-4 flex items-center justify-between">
      <h2 class="text-2xl text-text-primary font-bold">
        预设管理
      </h2>
      <Button variant="primary" @click="isCreating = !isCreating">
        <div class="i-carbon-add mr-1" />
        新建预设
      </Button>
    </div>

    <!-- 创建预设表单 -->
    <Card v-if="isCreating" class="mb-4">
      <h3 class="text-lg text-text-primary font-semibold mb-4">
        创建新预设
      </h3>
      <div class="space-y-3">
        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            预设名称
          </label>
          <Input
            v-model="newPreset.name"
            placeholder="例如：高清画质"
          />
        </div>

        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            描述（可选）
          </label>
          <Input
            v-model="newPreset.description"
            placeholder="例如：1080P 高清视频，适合收藏"
          />
        </div>

        <div>
          <label class="text-sm text-text-primary font-medium mb-2 block">
            视频质量
          </label>
          <select v-model="newPreset.videoQuality" class="input-base">
            <option :value="127">
              8K 超高清
            </option>
            <option :value="126">
              杜比视界
            </option>
            <option :value="125">
              HDR 真彩
            </option>
            <option :value="120">
              4K 超清
            </option>
            <option :value="116">
              1080P 60帧
            </option>
            <option :value="112">
              1080P 高码率
            </option>
            <option :value="80">
              1080P 高清
            </option>
            <option :value="64">
              720P 高清
            </option>
          </select>
        </div>

        <div class="flex gap-4">
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="newPreset.withDanmaku"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载弹幕</span>
          </label>
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="newPreset.withSubtitle"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载字幕</span>
          </label>
          <label class="flex gap-2 cursor-pointer items-center">
            <input
              v-model="newPreset.withCover"
              type="checkbox"
              class="h-4 w-4"
            >
            <span class="text-sm text-text-primary">下载封面</span>
          </label>
        </div>

        <div class="flex gap-2 justify-end">
          <Button variant="secondary" @click="isCreating = false">
            取消
          </Button>
          <Button variant="primary" @click="handleCreatePreset">
            创建
          </Button>
        </div>
      </div>
    </Card>

    <!-- 预设列表 -->
    <div v-if="presetsStore.presets.length === 0 && !isCreating">
      <Card>
        <div class="py-12 text-center">
          <div class="i-carbon-save text-6xl text-text-tertiary mx-auto mb-4" />
          <p class="text-text-secondary mb-4">
            暂无预设，点击上方按钮创建
          </p>
        </div>
      </Card>
    </div>

    <div v-else class="gap-4 grid grid-cols-1 lg:grid-cols-3 md:grid-cols-2">
      <Card
        v-for="preset in presetsStore.presets"
        :key="preset.id"
        class="card-hover"
      >
        <div class="mb-3 flex items-start justify-between">
          <div>
            <h3 class="text-lg text-text-primary font-semibold">
              {{ preset.name }}
            </h3>
            <p v-if="preset.description" class="text-sm text-text-secondary mt-1">
              {{ preset.description }}
            </p>
          </div>
          <div
            v-if="presetsStore.activePresetId === preset.id"
            class="text-xs text-primary-600 px-2 py-1 rounded bg-primary-100"
          >
            使用中
          </div>
        </div>

        <div class="text-sm text-text-secondary mb-4 space-y-2">
          <div class="flex gap-2 items-center">
            <div class="i-carbon-video" />
            <span>{{ getQualityLabel(preset.config.videoQuality) }}</span>
          </div>
          <div class="flex gap-3">
            <span v-if="preset.config.withDanmaku">✓ 弹幕</span>
            <span v-if="preset.config.withSubtitle">✓ 字幕</span>
            <span v-if="preset.config.withCover">✓ 封面</span>
          </div>
        </div>

        <div class="flex gap-2">
          <Button
            variant="primary"
            class="flex-1"
            @click="handleApplyPreset(preset)"
          >
            应用
          </Button>
          <Button variant="secondary" @click="handleDeletePreset(preset.id)">
            <div class="i-carbon-trash-can" />
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>
