export interface Comment {
  rpid: number
  oid: number
  mid: number
  uname: string
  avatar: string
  sex: string
  content: string
  ctime: number
  like: number
  current_level: number
  location: string
  parent: number
  pictures: Picture[]
}

export interface Picture {
  img_src: string
}
