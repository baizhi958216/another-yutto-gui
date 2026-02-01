export interface DownloadConfig {
  url: string
  videoQuality: number
  audioQuality: number
  downloadPath: string
  yuttoCliPath?: string
  withDanmaku: boolean
  withSubtitle: boolean
  withCover: boolean
  withComments: boolean
  batch: boolean
  videoOnly?: boolean
  audioOnly?: boolean
  episodes?: string
  sessdata?: string
  episodeQualities?: Record<number, { videoQuality: number, audioQuality: number }>
}

export interface DownloadTask {
  id: string
  config: DownloadConfig
  status: 'pending' | 'downloading' | 'paused' | 'completed' | 'error'
  progress: number
  speed: string
  eta: string
  error?: string
  warning?: string
  videoInfo?: {
    title: string
    thumbnail: string
  }
  totalSize: number // DEPRECATED: use totalBytes

  // NEW: Precise byte-level fields from JSON output
  downloadedBytes: number
  totalBytes: number
  speedBytesPerSec: number
  etaSeconds?: number
  filesCount?: number

  savedFilePath?: string
  commentFilePath?: string
  commentDownloadProgress?: string
  isDownloadingComments: boolean
  startTime: number
  processId?: number
  pausedAtProgress?: number
  pausedAtSpeed?: string
}

export interface DownloadProgress {
  downloadId: string
  progress: number
  speed: string
  eta: string
}
