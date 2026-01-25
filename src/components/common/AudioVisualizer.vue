<script lang="ts" setup>
import { Pause, Play, Volume2, VolumeX } from 'lucide-vue-next'
import { onMounted, onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
  src: string
  title?: string
  cover?: string
}>()

const audioElement = ref<HTMLAudioElement | null>(null)
const canvasElement = ref<HTMLCanvasElement | null>(null)
const isPlaying = ref(false)
const isMuted = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const volume = ref(1)

let audioContext: AudioContext | null = null
let analyser: AnalyserNode | null = null
let source: MediaElementAudioSourceNode | null = null
let animationId: number | null = null
let isAudioContextInitialized = false

// 频谱条数量
const barCount = 64

// 过渡动画相关
const transitionProgress = ref(0) // 0 = idle, 1 = playing
let transitionAnimationId: number | null = null
const transitionDuration = 300 // 过渡时长 ms

// 获取 CSS 变量的实际颜色值
function getCSSColor(varName: string, alpha: number = 1): string {
  const root = document.documentElement
  const rgb = getComputedStyle(root).getPropertyValue(varName).trim()
  if (rgb) {
    return `rgba(${rgb}, ${alpha})`
  }
  // 回退颜色
  return `rgba(99, 102, 241, ${alpha})`
}

function initAudioContext() {
  if (isAudioContextInitialized || !audioElement.value)
    return

  audioContext = new AudioContext()
  analyser = audioContext.createAnalyser()
  analyser.fftSize = 256
  analyser.smoothingTimeConstant = 0.8

  source = audioContext.createMediaElementSource(audioElement.value)
  source.connect(analyser)
  analyser.connect(audioContext.destination)

  isAudioContextInitialized = true
}

// 缓动函数 - easeOutCubic
function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3
}

// 缓动函数 - easeInCubic
function easeInCubic(t: number): number {
  return t ** 3
}

// 开始过渡动画到播放状态
function startTransitionToPlaying() {
  if (transitionAnimationId) {
    cancelAnimationFrame(transitionAnimationId)
  }

  const startValue = transitionProgress.value
  const startTime = performance.now()

  function animate() {
    const elapsed = performance.now() - startTime
    const progress = Math.min(elapsed / transitionDuration, 1)
    transitionProgress.value = startValue + (1 - startValue) * easeOutCubic(progress)

    if (progress < 1) {
      transitionAnimationId = requestAnimationFrame(animate)
    }
    else {
      transitionAnimationId = null
    }
  }

  animate()
}

// 开始过渡动画到暂停状态
function startTransitionToIdle() {
  if (transitionAnimationId) {
    cancelAnimationFrame(transitionAnimationId)
  }

  const startValue = transitionProgress.value
  const startTime = performance.now()

  function animate() {
    const elapsed = performance.now() - startTime
    const progress = Math.min(elapsed / transitionDuration, 1)
    transitionProgress.value = startValue * (1 - easeInCubic(progress))

    if (progress < 1) {
      transitionAnimationId = requestAnimationFrame(animate)
    }
    else {
      transitionAnimationId = null
    }
  }

  animate()
}

function draw() {
  if (!canvasElement.value || !analyser)
    return

  const canvas = canvasElement.value
  const ctx = canvas.getContext('2d')
  if (!ctx)
    return

  const bufferLength = analyser.frequencyBinCount
  const dataArray = new Uint8Array(bufferLength)
  analyser.getByteFrequencyData(dataArray)

  // 获取实际显示尺寸
  const rect = canvas.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  // 设置 canvas 实际像素尺寸
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr

  // 缩放上下文以匹配 CSS 尺寸
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = rect.height

  // 清除画布
  ctx.clearRect(0, 0, width, height)

  // 计算每个条的宽度和间距
  const barWidth = (width / barCount) * 0.7
  const gap = (width / barCount) * 0.3
  const centerY = height / 2

  // 获取当前过渡进度
  const t = transitionProgress.value
  const minBarHeight = 2 // idle 状态的最小高度

  // 绘制频谱条（镜像效果）
  for (let i = 0; i < barCount; i++) {
    // 从频谱数据中采样
    const dataIndex = Math.floor((i / barCount) * bufferLength)
    const value = dataArray[dataIndex]
    const targetBarHeight = (value / 255) * (height * 0.4)

    // 根据过渡进度插值计算实际高度
    const barHeight = minBarHeight + (targetBarHeight - minBarHeight) * t

    const x = i * (barWidth + gap)

    // 根据过渡进度插值计算透明度
    const baseAlpha = 0.4 + 0.6 * t

    // 创建渐变色
    const gradient = ctx.createLinearGradient(0, centerY - barHeight, 0, centerY + barHeight)
    gradient.addColorStop(0, getCSSColor('--color-accent-400-rgb', 0.9 * baseAlpha))
    gradient.addColorStop(0.5, getCSSColor('--color-accent-500-rgb', baseAlpha))
    gradient.addColorStop(1, getCSSColor('--color-accent-400-rgb', 0.9 * baseAlpha))

    ctx.fillStyle = gradient

    // 绘制上半部分
    ctx.beginPath()
    ctx.roundRect(x, centerY - barHeight, barWidth, barHeight, [barWidth / 2, barWidth / 2, 0, 0])
    ctx.fill()

    // 绘制下半部分（镜像）
    ctx.beginPath()
    ctx.roundRect(x, centerY, barWidth, barHeight, [0, 0, barWidth / 2, barWidth / 2])
    ctx.fill()
  }

  // 绘制中心线
  ctx.strokeStyle = getCSSColor('--color-accent-500-rgb', 0.3)
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, centerY)
  ctx.lineTo(width, centerY)
  ctx.stroke()

  animationId = requestAnimationFrame(draw)
}

function drawIdle() {
  if (!canvasElement.value)
    return

  const canvas = canvasElement.value
  const ctx = canvas.getContext('2d')
  if (!ctx)
    return

  const rect = canvas.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = rect.height

  ctx.clearRect(0, 0, width, height)

  const barWidth = (width / barCount) * 0.7
  const gap = (width / barCount) * 0.3
  const centerY = height / 2

  // 绘制静态的小条
  for (let i = 0; i < barCount; i++) {
    const x = i * (barWidth + gap)
    const barHeight = 2

    ctx.fillStyle = getCSSColor('--color-accent-500-rgb', 0.4)

    ctx.beginPath()
    ctx.roundRect(x, centerY - barHeight, barWidth, barHeight, [barWidth / 2, barWidth / 2, 0, 0])
    ctx.fill()

    ctx.beginPath()
    ctx.roundRect(x, centerY, barWidth, barHeight, [0, 0, barWidth / 2, barWidth / 2])
    ctx.fill()
  }

  // 绘制中心线
  ctx.strokeStyle = getCSSColor('--color-accent-500-rgb', 0.3)
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, centerY)
  ctx.lineTo(width, centerY)
  ctx.stroke()
}

function togglePlay() {
  if (!audioElement.value)
    return

  // 首次播放时初始化 AudioContext
  if (!isAudioContextInitialized) {
    initAudioContext()
  }

  // 恢复 AudioContext（如果被暂停）
  if (audioContext?.state === 'suspended') {
    audioContext.resume()
  }

  if (isPlaying.value) {
    audioElement.value.pause()
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }
  }
  else {
    audioElement.value.play()
    draw()
  }
}

function toggleMute() {
  if (!audioElement.value)
    return

  isMuted.value = !isMuted.value
  audioElement.value.muted = isMuted.value
}

function handleVolumeChange(e: Event) {
  const target = e.target as HTMLInputElement
  volume.value = Number.parseFloat(target.value)
  if (audioElement.value) {
    audioElement.value.volume = volume.value
  }
}

function handleSeek(e: Event) {
  const target = e.target as HTMLInputElement
  const time = Number.parseFloat(target.value)
  if (audioElement.value) {
    audioElement.value.currentTime = time
    currentTime.value = time
  }
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function handleTimeUpdate() {
  if (audioElement.value) {
    currentTime.value = audioElement.value.currentTime
  }
}

function handleLoadedMetadata() {
  if (audioElement.value) {
    duration.value = audioElement.value.duration
  }
}

function handlePlay() {
  isPlaying.value = true
  startTransitionToPlaying()
}

function handlePause() {
  isPlaying.value = false
  startTransitionToIdle()
}

function handleEnded() {
  isPlaying.value = false
  if (animationId) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
  startTransitionToIdle()
}

watch(() => props.src, () => {
  // 当 src 改变时重置状态
  isPlaying.value = false
  currentTime.value = 0
  duration.value = 0
  transitionProgress.value = 0
  if (animationId) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
  if (transitionAnimationId) {
    cancelAnimationFrame(transitionAnimationId)
    transitionAnimationId = null
  }
  drawIdle()
})

onMounted(() => {
  drawIdle()
})

onUnmounted(() => {
  if (animationId) {
    cancelAnimationFrame(animationId)
  }
  if (transitionAnimationId) {
    cancelAnimationFrame(transitionAnimationId)
  }
  if (audioContext) {
    audioContext.close()
  }
})
</script>

<template>
  <div class="mx-auto rounded-4 bg-bg-secondary max-w-800px w-full overflow-hidden">
    <!-- 封面和可视化区域 -->
    <div class="flex w-full aspect-16/9 items-center justify-center relative overflow-hidden from-bg-tertiary to-bg-secondary bg-gradient-to-br">
      <!-- 背景模糊效果 -->
      <div
        v-if="cover"
        class="opacity-60 absolute bg-cover bg-center blur-40px brightness-40 -inset-5"
        :style="{ backgroundImage: `url(${cover})` }"
      />

      <!-- 封面图片 -->
      <div class="h-40 w-40 absolute z-1">
        <img
          v-if="cover"
          :src="cover"
          :alt="title"
          class="rounded-full h-full w-full shadow-[0_8px_32px_rgba(0,0,0,0.3)] object-cover"
          referrerpolicy="no-referrer"
        >
        <div v-else class="rounded-full bg-bg-tertiary flex h-full w-full shadow-[0_8px_32px_rgba(0,0,0,0.3)] items-center justify-center">
          <div class="i-carbon:music text-8xl text-text-tertiary" />
        </div>
      </div>

      <!-- 频谱可视化 -->
      <canvas ref="canvasElement" class="h-20 w-full bottom-0 left-0 absolute z-2" />
    </div>

    <!-- 音频元素 -->
    <audio
      ref="audioElement"
      :src="src"
      crossorigin="anonymous"
      preload="metadata"
      @timeupdate="handleTimeUpdate"
      @loadedmetadata="handleLoadedMetadata"
      @play="handlePlay"
      @pause="handlePause"
      @ended="handleEnded"
    />

    <!-- 控制区域 -->
    <div class="px-6 pb-6 pt-4">
      <!-- 进度条 -->
      <div class="mb-4 flex gap-3 items-center">
        <span class="text-xs text-text-tertiary text-center min-w-10 tabular-nums">{{ formatTime(currentTime) }}</span>
        <input
          type="range"
          class="progress-bar appearance-none rounded-sm bg-bg-tertiary flex-1 h-1 cursor-pointer"
          :value="currentTime"
          :max="duration || 0"
          step="0.1"
          @input="handleSeek"
        >
        <span class="text-xs text-text-tertiary text-center min-w-10 tabular-nums">{{ formatTime(duration) }}</span>
      </div>

      <!-- 播放控制 -->
      <div class="flex gap-6 items-center justify-center">
        <button
          class="bg-accent-500 hover:bg-accent-600 text-white rounded-full flex h-14 w-14 cursor-pointer transition-all duration-150 items-center justify-center hover:scale-105"
          @click="togglePlay"
        >
          <Pause v-if="isPlaying" :size="28" />
          <Play v-else :size="28" class="ml-0.5" />
        </button>

        <!-- 音量控制 -->
        <div class="flex gap-2 items-center">
          <button
            class="hover:text-accent-500 text-text-primary border-none bg-transparent flex cursor-pointer transition-all duration-150 items-center justify-center hover:scale-110"
            @click="toggleMute"
          >
            <VolumeX v-if="isMuted" :size="20" />
            <Volume2 v-else :size="20" />
          </button>
          <input
            type="range"
            class="volume-bar appearance-none rounded-sm bg-bg-tertiary h-1 w-20 cursor-pointer"
            :value="volume"
            min="0"
            max="1"
            step="0.01"
            @input="handleVolumeChange"
          >
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Slider thumb 样式无法用原子化 CSS 实现 */
.progress-bar::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--color-accent-500);
  border-radius: 50%;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.progress-bar::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.volume-bar::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 10px;
  height: 10px;
  background: var(--color-accent-500);
  border-radius: 50%;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.volume-bar::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}
</style>
