export interface HistoryEntry {
  id: string
  title: string
  url: string
  thumbnail: string
  downloadDate: number
  filePath: string
  videoQuality: number
  audioQuality: number
  videoOnly?: boolean
  audioOnly?: boolean
  size: number
  commentFilePath?: string
}
