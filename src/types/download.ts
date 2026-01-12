export interface DownloadConfig {
  url: string
  videoQuality: number
  audioQuality: number
  downloadPath: string
  withDanmaku: boolean
  withSubtitle: boolean
  withCover: boolean
  batch: boolean
  videoOnly?: boolean
  audioOnly?: boolean
  episodes?: string
  sessdata?: string
}

export interface DownloadTask {
  id: string
  config: DownloadConfig
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error'
  progress: number
  speed: string
  eta: string
  error?: string
  videoInfo?: {
    title: string
    thumbnail: string
  }
}

export interface DownloadProgress {
  downloadId: string
  progress: number
  speed: string
  eta: string
}
