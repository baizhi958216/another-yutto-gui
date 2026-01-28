/**
 * Get CSS variable color value with alpha
 */
export function getCSSColor(varName: string, alpha: number = 1): string {
  const root = document.documentElement
  const rgb = getComputedStyle(root).getPropertyValue(varName).trim()
  if (rgb) {
    return `rgba(${rgb}, ${alpha})`
  }
  // Fallback color
  return `rgba(99, 102, 241, ${alpha})`
}

/**
 * Calculate visualizer metrics based on canvas dimensions
 */
export function getVisualizerMetrics(width: number, height: number, barCount: number = 96) {
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

/**
 * Clamp alpha value between 0 and 1
 */
export function clampAlpha(value: number): number {
  return Math.min(1, Math.max(0, value))
}

/**
 * Easing function - easeOutCubic
 */
export function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3
}

/**
 * Easing function - easeInCubic
 */
export function easeInCubic(t: number): number {
  return t ** 3
}

/**
 * Format time in seconds to MM:SS format
 */
export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}
