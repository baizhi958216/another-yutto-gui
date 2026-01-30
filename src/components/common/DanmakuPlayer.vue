<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import ASS from 'assjs'
import { nextTick, onUnmounted, ref, watch } from 'vue'

interface Props {
  videoElement: HTMLVideoElement | null
  videoFilePath: string
  visible?: boolean
}

const props = defineProps<Props>()

let ass: ASS | null = null
const canvasVisible = ref(props.visible !== false)

// 加载 ASS 字幕文件
async function loadAssFile() {
  if (!props.videoFilePath || !props.videoElement)
    return

  try {
    // 获取视频文件名（不含扩展名）
    const videoFileName = props.videoFilePath.split(/[/\\]/).pop() || ''
    const baseName = videoFileName.replace(/\.[^/.]+$/, '')
    // 构建 ASS 文件路径（与视频同目录）
    const assFilePath = props.videoFilePath.replace(/\.[^/.]+$/, '.ass')

    // 通过 Tauri 读取 ASS 文件内容
    const assContent = await invoke<string>('read_csv_file', { filePath: assFilePath })

    // 初始化 ASS 渲染器
    ass = new ASS(assContent, props.videoElement)

    // 等待 DOM 更新后设置初始可见性
    await nextTick()
    // 使用 setTimeout 确保 ASS 完全渲染完成
    setTimeout(() => {
      updateCanvasVisibility()
    }, 100)
  }
  catch (error) {
    console.warn('无法加载 ASS 字幕文件:', error)
  }
}

// 更新 canvas 可见性
function updateCanvasVisibility() {
  if (!ass)
    return

  // 使用 ASS.js 的 API 来控制显示/隐藏
  if (canvasVisible.value) {
    ass.show()
  }
  else {
    ass.hide()
  }
}

// 监听视频元素变化
watch(() => props.videoElement, (newElement) => {
  if (newElement) {
    loadAssFile()
  }
}, { immediate: true })

// 监听 visible 属性变化
watch(() => props.visible, (newVal) => {
  canvasVisible.value = newVal !== false
  updateCanvasVisibility()
})

onUnmounted(() => {
  ass?.destroy()
})
</script>

<template>
  <!-- ASS 会自动创建 canvas 覆盖在 video 上，不需要手动创建 -->
  <div class="danmaku-placeholder" />
</template>

<style scoped>
.danmaku-placeholder {
  display: none;
}
</style>
