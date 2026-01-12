export interface Preset {
  id: string
  name: string
  description?: string
  config: {
    videoQuality: number
    audioQuality: number
    withDanmaku: boolean
    withSubtitle: boolean
    withCover: boolean
  }
  createdAt: number
}
