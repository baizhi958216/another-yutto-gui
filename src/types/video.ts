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
  }
  episodes?: Episode[]
}

export interface Episode {
  id: number
  title: string
  duration: number
  index: number
}
