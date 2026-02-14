export interface Comment {
  rpid: number
  oid: number
  mid: number
  root: number
  uname: string
  avatar: string
  sex: string
  content: string
  ctime: number
  like: number
  reply_count: number
  current_level: number
  location: string
  parent: number
  pictures: Picture[]
  emotes: Emote[]
  replies?: Comment[]
}

export interface Picture {
  img_src: string
}

export interface Emote {
  text: string
  url: string
}
