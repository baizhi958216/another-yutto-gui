<script lang="ts" setup>
import { Pause, Play, Volume2, VolumeX } from 'lucide-vue-next'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import SmartImage from '@/components/common/SmartImage.vue'
import { normalizeImageUrl } from '@/utils/image'

const props = defineProps<{
  src: string
  title?: string
  cover?: string
}>()

const normalizedCover = computed(() => normalizeImageUrl(props.cover))

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
const barCount = 96

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

function getVisualizerMetrics(width: number, height: number) {
  const step = width / barCount
  const barWidth = step * 0.55
  const gap = step - barWidth
  const topPadding = Math.max(6, height * 0.12)
  const bottomPadding = Math.max(8, height * 0.18)
  const baseline = height - bottomPadding
  const maxBarHeight = Math.max(4, baseline - topPadding)
  const minBarHeight = Math.max(1.5, maxBarHeight * 0.04)
  return { barWidth, gap, baseline, maxBarHeight, minBarHeight }
}

function clampAlpha(value: number): number {
  return Math.min(1, Math.max(0, value))
}

function initAudioContext() {
  if (isAudioContextInitialized || !audioElement.value)
    return

  audioContext = new AudioContext()
  analyser = audioContext.createAnalyser()
  analyser.fftSize = 512
  analyser.smoothingTimeConstant = 0.85

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

  const rect = canvas.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)

  const width = rect.width
  const height = rect.height

  ctx.clearRect(0, 0, width, height)

  const { barWidth, gap, baseline, maxBarHeight, minBarHeight } = getVisualizerMetrics(width, height)
  const t = transitionProgress.value
  const offset = gap / 2
  const isDarkTheme = document.documentElement.classList.contains('dark')
  const alphaBoost = isDarkTheme ? 1 : 1.45
  const strongAccent = isDarkTheme ? '--color-accent-400-rgb' : '--color-accent-600-rgb'
  const softAccent = '--color-accent-500-rgb'

  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'

  for (let i = 0; i < barCount; i++) {
    const pos = i / (barCount - 1)
    const dataIndex = Math.floor(pos ** 1.6 * (bufferLength - 1))
    const value = dataArray[dataIndex] ?? 0
    const normalized = (value / 255) ** 1.5
    const edge = Math.abs(pos - 0.5) * 2
    const edgeFade = 0.7 + 0.3 * (1 - edge * edge)
    const barHeight = minBarHeight + (maxBarHeight * normalized * edgeFade) * t
    const x = offset + i * (barWidth + gap)
    const y = baseline - barHeight
    const baseAlpha = clampAlpha((0.2 + 0.75 * t) * edgeFade * alphaBoost)

    const gradient = ctx.createLinearGradient(0, y, 0, baseline)
    gradient.addColorStop(0, getCSSColor(strongAccent, 0.95 * baseAlpha))
    gradient.addColorStop(1, getCSSColor(softAccent, 0.25 * baseAlpha))

    ctx.fillStyle = gradient
    const radius = Math.min(barWidth / 2, barHeight / 2)
    ctx.beginPath()
    ctx.roundRect(x, y, barWidth, barHeight, radius)
    ctx.fill()

    if (barHeight > 4) {
      ctx.strokeStyle = getCSSColor(strongAccent, 0.6 * baseAlpha)
      ctx.lineWidth = 1
      ctx.beginPath()
      ctx.moveTo(x, y + 0.5)
      ctx.lineTo(x + barWidth, y + 0.5)
      ctx.stroke()
    }
  }

  const baselineAlpha = isDarkTheme ? (0.12 + 0.18 * t) : (0.24 + 0.24 * t)
  ctx.strokeStyle = getCSSColor(strongAccent, baselineAlpha)
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, baseline + 0.5)
  ctx.lineTo(width, baseline + 0.5)
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
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)

  const width = rect.width
  const height = rect.height

  ctx.clearRect(0, 0, width, height)

  const { barWidth, gap, baseline, maxBarHeight, minBarHeight } = getVisualizerMetrics(width, height)
  const offset = gap / 2
  const idleLevel = 0.08
  const isDarkTheme = document.documentElement.classList.contains('dark')
  const alphaBoost = isDarkTheme ? 1 : 1.7
  const strongAccent = isDarkTheme ? '--color-accent-400-rgb' : '--color-accent-600-rgb'
  const softAccent = '--color-accent-500-rgb'

  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'

  for (let i = 0; i < barCount; i++) {
    const pos = i / (barCount - 1)
    const edge = Math.abs(pos - 0.5) * 2
    const edgeFade = 0.7 + 0.3 * (1 - edge * edge)
    const barHeight = minBarHeight + maxBarHeight * idleLevel * edgeFade
    const x = offset + i * (barWidth + gap)
    const y = baseline - barHeight
    const baseAlpha = clampAlpha(0.26 * edgeFade * alphaBoost)

    const gradient = ctx.createLinearGradient(0, y, 0, baseline)
    gradient.addColorStop(0, getCSSColor(strongAccent, 0.7 * baseAlpha))
    gradient.addColorStop(1, getCSSColor(softAccent, 0.2 * baseAlpha))

    ctx.fillStyle = gradient
    const radius = Math.min(barWidth / 2, barHeight / 2)
    ctx.beginPath()
    ctx.roundRect(x, y, barWidth, barHeight, radius)
    ctx.fill()
  }

  const baselineAlpha = isDarkTheme ? 0.12 : 0.22
  ctx.strokeStyle = getCSSColor(strongAccent, baselineAlpha)
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, baseline + 0.5)
  ctx.lineTo(width, baseline + 0.5)
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
      <SmartImage
        v-if="normalizedCover"
        :src="normalizedCover"
        alt=""
        class="opacity-60 h-full w-full scale-110 absolute object-cover blur-40px brightness-40"
        aria-hidden="true"
      />

      <!-- 封面图片 -->
      <div class="h-40 w-40 absolute z-1">
        <SmartImage
          v-if="normalizedCover"
          :src="normalizedCover"
          :alt="title"
          class="rounded-full h-full w-full shadow-[0_8px_32px_rgba(0,0,0,0.3)] object-cover"
        />
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
          class="bg-accent-500 hover:bg-accent-600 text-primary rounded-full flex h-14 w-14 cursor-pointer transition-all duration-150 items-center justify-center hover:scale-105"
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
