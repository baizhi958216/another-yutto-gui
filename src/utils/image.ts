import type { Comment, HistoryEntry, VideoInfo } from '@/types'

const ABSOLUTE_PROTOCOL_RE = /^[a-z][a-z\d+.-]*:/i
const DOMAIN_LIKE_RE = /^(?:[a-z0-9-]+\.)+[a-z]{2,}(?::\d+)?(?:\/|\?)/i

/**
 * Normalize external image URLs for cross-platform WebView compatibility.
 * - `//example.com/a.jpg` -> `https://example.com/a.jpg`
 * - `http://...` -> `https://...`
 * - `i0.hdslb.com/...` -> `https://i0.hdslb.com/...`
 */
export function normalizeImageUrl(url: string | null | undefined): string {
  if (!url) {
    return ''
  }

  const trimmedUrl = url.trim()
  if (!trimmedUrl) {
    return ''
  }

  if (trimmedUrl.startsWith('//')) {
    return `https:${trimmedUrl}`
  }

  if (trimmedUrl.startsWith('http://')) {
    return `https://${trimmedUrl.slice('http://'.length)}`
  }

  if (ABSOLUTE_PROTOCOL_RE.test(trimmedUrl)) {
    return trimmedUrl
  }

  if (DOMAIN_LIKE_RE.test(trimmedUrl)) {
    return `https://${trimmedUrl}`
  }

  return trimmedUrl
}

export function normalizeVideoInfoImages(videoInfo: VideoInfo): VideoInfo {
  return {
    ...videoInfo,
    thumbnail: normalizeImageUrl(videoInfo.thumbnail),
    owner: {
      ...videoInfo.owner,
      face: normalizeImageUrl(videoInfo.owner.face),
    },
  }
}

export function normalizeHistoryEntryImages(entry: HistoryEntry): HistoryEntry {
  return {
    ...entry,
    thumbnail: normalizeImageUrl(entry.thumbnail),
  }
}

export function normalizeCommentImages(comment: Comment): Comment {
  return {
    ...comment,
    avatar: normalizeImageUrl(comment.avatar),
    pictures: comment.pictures.map(picture => ({
      ...picture,
      img_src: normalizeImageUrl(picture.img_src),
    })),
    replies: comment.replies?.map(normalizeCommentImages),
  }
}
