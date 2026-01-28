import type { Ref } from 'vue'
import { onUnmounted, ref } from 'vue'
import { clampAlpha, easeInCubic, easeOutCubic, getCSSColor, getVisualizerMetrics } from '@/utils/audio-helpers'

/**
 * Composable for managing canvas visualization
 */
export function useVisualizerCanvas(
  canvasElement: Ref<HTMLCanvasElement | null>,
  analyser: AnalyserNode | null,
  barCount: number = 96,
) {
  let animationId: number | null = null
  let transitionAnimationId: number | null = null
  const transitionProgress = ref(0) // 0 = idle, 1 = playing
  const transitionDuration = 300 // ms

  /**
   * Start transition animation to playing state
   */
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

  /**
   * Start transition animation to idle state
   */
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

  /**
   * Draw frequency bars on canvas
   */
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

    const { barWidth, gap, baseline, maxBarHeight, minBarHeight } = getVisualizerMetrics(width, height, barCount)
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

    animationId = requestAnimationFrame(draw)
  }

  /**
   * Draw idle state (no audio playing)
   */
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

    const { barWidth, gap, baseline, minBarHeight } = getVisualizerMetrics(width, height, barCount)
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
      const edge = Math.abs(pos - 0.5) * 2
      const edgeFade = 0.7 + 0.3 * (1 - edge * edge)
      const barHeight = minBarHeight
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
    }

    animationId = requestAnimationFrame(drawIdle)
  }

  /**
   * Start drawing animation
   */
  function startDrawing(isPlaying: boolean) {
    if (animationId) {
      cancelAnimationFrame(animationId)
    }
    if (isPlaying) {
      draw()
    }
    else {
      drawIdle()
    }
  }

  /**
   * Stop drawing animation
   */
  function stopDrawing() {
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }
    if (transitionAnimationId) {
      cancelAnimationFrame(transitionAnimationId)
      transitionAnimationId = null
    }
  }

  onUnmounted(() => {
    stopDrawing()
  })

  return {
    transitionProgress,
    startTransitionToPlaying,
    startTransitionToIdle,
    startDrawing,
    stopDrawing,
  }
}
