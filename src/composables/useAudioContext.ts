import type { Ref } from 'vue'
import { onUnmounted } from 'vue'

/**
 * Composable for managing AudioContext and AnalyserNode
 */
export function useAudioContext(audioElement: Ref<HTMLAudioElement | null>) {
  let audioContext: AudioContext | null = null
  let analyser: AnalyserNode | null = null
  let source: MediaElementAudioSourceNode | null = null
  let isInitialized = false

  /**
   * Initialize AudioContext and connect to audio element
   */
  function initAudioContext() {
    if (isInitialized || !audioElement.value)
      return

    audioContext = new AudioContext()
    analyser = audioContext.createAnalyser()
    analyser.fftSize = 512
    analyser.smoothingTimeConstant = 0.85

    source = audioContext.createMediaElementSource(audioElement.value)
    source.connect(analyser)
    analyser.connect(audioContext.destination)

    isInitialized = true
  }

  /**
   * Cleanup AudioContext on unmount
   */
  function cleanup() {
    if (source) {
      source.disconnect()
      source = null
    }
    if (analyser) {
      analyser.disconnect()
      analyser = null
    }
    if (audioContext) {
      audioContext.close()
      audioContext = null
    }
    isInitialized = false
  }

  onUnmounted(() => {
    cleanup()
  })

  return {
    audioContext,
    analyser,
    initAudioContext,
    cleanup,
  }
}
