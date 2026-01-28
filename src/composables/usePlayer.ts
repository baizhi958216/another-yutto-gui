import type { Ref } from 'vue'
import * as PlyrNamespace from 'plyr'
import { onUnmounted } from 'vue'
import 'plyr/dist/plyr.css'

const Plyr = (PlyrNamespace as any).default || PlyrNamespace

export function usePlayer(videoElement: Ref<HTMLVideoElement | null>) {
  let player: any = null

  function initPlayer() {
    if (videoElement.value) {
      player = new Plyr(videoElement.value, {
        controls: ['play-large', 'play', 'progress', 'current-time', 'mute', 'volume', 'settings', 'fullscreen'],
        settings: ['quality', 'speed'],
      })
    }
  }

  function destroyPlayer() {
    player?.destroy()
    player = null
  }

  onUnmounted(() => {
    destroyPlayer()
  })

  return {
    initPlayer,
    destroyPlayer,
    getPlayer: () => player,
  }
}
