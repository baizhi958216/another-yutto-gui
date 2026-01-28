export interface VideoInfo {
  title: string
  bvid: string
  aid: number
  thumbnail: string
  duration: number
  description: string
  owner: {
    uid: number
    name: string
    face: string
    sign?: string
    level?: number
    location?: string
  }
  is_favorite?: boolean
  episodes?: Episode[]
  available_qualities?: QualityOption[]
  available_audio_qualities?: AudioQualityOption[]
  comment_count?: number
}

export interface QualityOption {
  quality: number
  description: string
  available?: boolean
  vip_only?: boolean
  login_required?: boolean
}

export interface AudioQualityOption {
  quality: number
  description: string
  available?: boolean
  vip_only?: boolean
  login_required?: boolean
}

export interface Episode {
  id: number
  title: string
  duration: number
  index: number
  available_qualities?: QualityOption[]
  available_audio_qualities?: AudioQualityOption[]
}
