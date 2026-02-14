<script lang="ts" setup>
import type { SortType } from '@/composables/useComments'
import type { Comment } from '@/types'
import { X, ZoomIn, ZoomOut } from 'lucide-vue-next'
import { ref } from 'vue'
import Button from '@/components/common/Button.vue'
import SmartImage from '@/components/common/SmartImage.vue'
import { formatRelativeTime } from '@/utils/format'

interface Props {
  comments: Comment[]
  totalComments: number
  sortType: SortType
  currentPage: number
  totalPages: number
  hasNextPage: boolean
  hasPrevPage: boolean
  loading: boolean
  error: string | null
}

defineProps<Props>()

const emit = defineEmits<{
  changeSortType: [type: SortType]
  nextPage: []
  prevPage: []
  goToPage: [page: number]
}>()

// 将秒级时间戳转换为毫秒级时间戳
const formatCommentTime = (timestamp: number) => formatRelativeTime(timestamp * 1000)

const expandedReplyIds = ref<Set<number>>(new Set())
const replyPageMap = ref<Record<number, number>>({})
const repliesPageSize = 10
const previewImageUrl = ref('')
const previewImageAlt = ref('')
const previewZoom = ref(1)
const maxPreviewZoom = 3
const minPreviewZoom = 0.5

interface ContentSegment {
  type: 'text' | 'emote'
  value?: string
  url?: string
  alt?: string
}

function getReplies(comment: Comment): Comment[] {
  return comment.replies || []
}

function getEmotes(comment: Comment): { text: string, url: string }[] {
  return comment.emotes || []
}

function normalizeEmoteKey(text: string): string {
  if (text.startsWith('[') && text.endsWith(']')) {
    return text
  }
  return `[${text}]`
}

function buildEmoteMap(comment: Comment): Map<string, string> {
  const map = new Map<string, string>()

  for (const emote of getEmotes(comment)) {
    if (!emote.url)
      continue

    const normalizedKey = normalizeEmoteKey(emote.text)
    map.set(normalizedKey, emote.url)
    map.set(emote.text, emote.url)
  }

  return map
}

function splitContentByEmotes(comment: Comment): ContentSegment[] {
  const content = comment.content || ''
  if (!content) {
    return [{ type: 'text', value: '' }]
  }

  const emoteMap = buildEmoteMap(comment)
  const segments: ContentSegment[] = []
  const emotePattern = /\[[^\]\r\n]+\]/g
  let lastIndex = 0

  for (const match of content.matchAll(emotePattern)) {
    const token = match[0]
    const index = match.index ?? 0

    if (index > lastIndex) {
      segments.push({
        type: 'text',
        value: content.slice(lastIndex, index),
      })
    }

    const emoteUrl = emoteMap.get(token)
    if (emoteUrl) {
      segments.push({
        type: 'emote',
        url: emoteUrl,
        alt: token,
      })
    }
    else {
      segments.push({
        type: 'text',
        value: token,
      })
    }

    lastIndex = index + token.length
  }

  if (lastIndex < content.length) {
    segments.push({
      type: 'text',
      value: content.slice(lastIndex),
    })
  }

  if (segments.length === 0) {
    segments.push({
      type: 'text',
      value: content,
    })
  }

  return segments
}

function getReplyCount(comment: Comment): number {
  const replies = getReplies(comment)
  if (replies.length > 0) {
    return replies.length
  }
  return comment.reply_count || 0
}

function hasReplies(comment: Comment): boolean {
  return getReplyCount(comment) > 0
}

function isRepliesExpanded(commentRpid: number): boolean {
  return expandedReplyIds.value.has(commentRpid)
}

function getReplyTotalPages(comment: Comment): number {
  const total = getReplies(comment).length
  return Math.max(1, Math.ceil(total / repliesPageSize))
}

function getCurrentReplyPage(commentRpid: number): number {
  return replyPageMap.value[commentRpid] || 1
}

function setCurrentReplyPage(commentRpid: number, page: number): void {
  replyPageMap.value = {
    ...replyPageMap.value,
    [commentRpid]: page,
  }
}

function getPagedReplies(comment: Comment): Comment[] {
  const replies = getReplies(comment)
  const totalPages = getReplyTotalPages(comment)
  const currentPage = Math.min(getCurrentReplyPage(comment.rpid), totalPages)
  const start = (currentPage - 1) * repliesPageSize
  const end = start + repliesPageSize
  return replies.slice(start, end)
}

function goToPrevReplyPage(comment: Comment): void {
  const currentPage = getCurrentReplyPage(comment.rpid)
  if (currentPage > 1) {
    setCurrentReplyPage(comment.rpid, currentPage - 1)
  }
}

function goToNextReplyPage(comment: Comment): void {
  const currentPage = getCurrentReplyPage(comment.rpid)
  const totalPages = getReplyTotalPages(comment)
  if (currentPage < totalPages) {
    setCurrentReplyPage(comment.rpid, currentPage + 1)
  }
}

function toggleReplies(commentRpid: number): void {
  const next = new Set(expandedReplyIds.value)
  if (next.has(commentRpid)) {
    next.delete(commentRpid)
  }
  else {
    next.add(commentRpid)
    if (!replyPageMap.value[commentRpid]) {
      setCurrentReplyPage(commentRpid, 1)
    }
  }
  expandedReplyIds.value = next
}

function openImagePreview(url: string, alt: string): void {
  previewImageUrl.value = url
  previewImageAlt.value = alt
  previewZoom.value = 1
}

function closeImagePreview(): void {
  previewImageUrl.value = ''
  previewImageAlt.value = ''
  previewZoom.value = 1
}

function zoomInPreview(): void {
  previewZoom.value = Math.min(maxPreviewZoom, previewZoom.value + 0.25)
}

function zoomOutPreview(): void {
  previewZoom.value = Math.max(minPreviewZoom, previewZoom.value - 0.25)
}
</script>

<template>
  <div class="comments-section">
    <div class="mb-4 flex items-center justify-between">
      <h2 class="text-xl text-text-primary font-bold flex gap-2 items-center">
        评论 {{ totalComments > 0 ? `(${totalComments})` : '' }}
      </h2>

      <!-- 排序选项 -->
      <div class="flex gap-2">
        <button
          class="text-sm px-3 py-1.5 rounded transition-colors"
          :class="sortType === 'time' ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
          @click="emit('changeSortType', 'time')"
        >
          按时间
        </button>
        <button
          class="text-sm px-3 py-1.5 rounded transition-colors"
          :class="sortType === 'likes' ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
          @click="emit('changeSortType', 'likes')"
        >
          按点赞
        </button>
      </div>
    </div>

    <div v-if="error" class="text-sm text-error p-4 rounded bg-bg-secondary">
      {{ error }}
    </div>

    <div v-if="loading" class="py-8 text-center">
      <div class="text-text-tertiary">
        加载评论中...
      </div>
    </div>

    <div v-else-if="totalComments === 0" class="py-8 text-center">
      <div class="text-text-tertiary">
        暂无评论
      </div>
    </div>

    <div v-else>
      <TransitionGroup name="comment-list" tag="div" class="space-y-4">
        <div
          v-for="comment in comments"
          :key="comment.rpid"
          class="p-4 rounded-lg bg-bg-secondary transition-colors hover:bg-bg-tertiary"
        >
          <div class="flex gap-3">
            <!-- 用户头像 -->
            <SmartImage
              :src="comment.avatar"
              :alt="comment.uname"
              class="rounded-full flex-shrink-0 h-10 w-10"
            />
            <div class="flex-1 min-w-0">
              <!-- 用户信息 -->
              <div class="mb-2 flex gap-2 items-center">
                <span class="text-sm text-text-primary font-medium">
                  {{ comment.uname }}
                </span>
                <span class="text-xs text-text-tertiary">
                  LV{{ comment.current_level }}
                </span>
                <span class="text-xs text-text-tertiary">
                  {{ formatCommentTime(comment.ctime) }}
                </span>
                <span v-if="comment.location" class="text-xs text-text-tertiary">
                  {{ comment.location }}
                </span>
              </div>
              <!-- 评论内容 -->
              <div class="text-sm text-text-primary leading-relaxed whitespace-pre-wrap break-words">
                <template
                  v-for="(segment, segmentIdx) in splitContentByEmotes(comment)"
                  :key="`${comment.rpid}-segment-${segmentIdx}`"
                >
                  <span v-if="segment.type === 'text'">{{ segment.value }}</span>
                  <SmartImage
                    v-else
                    :src="segment.url"
                    :alt="segment.alt"
                    :title="segment.alt"
                    class="mx-0.5 align-text-bottom h-6 w-6 inline-block object-contain"
                  />
                </template>
              </div>
              <!-- 评论图片 -->
              <div v-if="comment.pictures && comment.pictures.length > 0" class="mt-2 flex flex-wrap gap-2">
                <SmartImage
                  v-for="(pic, idx) in comment.pictures"
                  :key="idx"
                  :src="pic.img_src"
                  class="rounded max-h-32 cursor-zoom-in transition-opacity object-cover hover:opacity-90"
                  @click="openImagePreview(pic.img_src, `${comment.uname} 的评论图片`)"
                />
              </div>
              <!-- 点赞数 -->
              <div class="text-xs text-text-tertiary mt-2 flex gap-1 w-fit items-center justify-center">
                <div class="i-carbon:thumbs-up-filled mt--0.5" />
                {{ comment.like }}
              </div>

              <button
                v-if="hasReplies(comment)"
                class="text-xs text-primary mt-2 transition-opacity hover:opacity-80"
                type="button"
                @click="toggleReplies(comment.rpid)"
              >
                {{ isRepliesExpanded(comment.rpid) ? '收起回复' : `查看回复 (${getReplyCount(comment)})` }}
              </button>

              <div
                v-if="isRepliesExpanded(comment.rpid)"
                class="mt-3 pl-3 border-l border-bg-tertiary space-y-3"
              >
                <div
                  v-if="getReplies(comment).length === 0"
                  class="text-xs text-text-tertiary"
                >
                  暂无可展示的回复
                </div>

                <div
                  v-for="reply in getPagedReplies(comment)"
                  :key="reply.rpid"
                  class="p-3 rounded bg-bg-primary"
                >
                  <div class="flex gap-2 items-center">
                    <SmartImage
                      :src="reply.avatar"
                      :alt="reply.uname"
                      class="rounded-full flex-shrink-0 h-6 w-6"
                    />
                    <span class="text-xs text-text-primary font-medium">
                      {{ reply.uname }}
                    </span>
                    <span class="text-xs text-text-tertiary">
                      {{ formatCommentTime(reply.ctime) }}
                    </span>
                    <span v-if="reply.location" class="text-xs text-text-tertiary">
                      {{ reply.location }}
                    </span>
                  </div>
                  <div class="text-xs text-text-primary leading-relaxed mt-1 whitespace-pre-wrap break-words">
                    <template
                      v-for="(segment, segmentIdx) in splitContentByEmotes(reply)"
                      :key="`${reply.rpid}-segment-${segmentIdx}`"
                    >
                      <span v-if="segment.type === 'text'">{{ segment.value }}</span>
                      <SmartImage
                        v-else
                        :src="segment.url"
                        :alt="segment.alt"
                        :title="segment.alt"
                        class="mx-0.5 align-text-bottom h-5 w-5 inline-block object-contain"
                      />
                    </template>
                  </div>
                  <div
                    v-if="reply.pictures && reply.pictures.length > 0"
                    class="mt-2 flex flex-wrap gap-2"
                  >
                    <SmartImage
                      v-for="(pic, picIdx) in reply.pictures"
                      :key="`${reply.rpid}-pic-${picIdx}`"
                      :src="pic.img_src"
                      class="rounded max-h-24 cursor-zoom-in transition-opacity object-cover hover:opacity-90"
                      @click="openImagePreview(pic.img_src, `${reply.uname} 的回复图片`)"
                    />
                  </div>
                  <div class="text-xs text-text-tertiary mt-1 flex gap-1 items-center">
                    <div class="i-carbon:thumbs-up-filled mt--0.5" />
                    {{ reply.like }}
                  </div>
                </div>

                <div
                  v-if="getReplies(comment).length > repliesPageSize"
                  class="pt-1 flex gap-2 items-center"
                >
                  <button
                    class="text-xs text-text-primary px-2 py-1 rounded bg-bg-secondary transition-colors hover:bg-bg-tertiary disabled:opacity-50 disabled:cursor-not-allowed"
                    :disabled="getCurrentReplyPage(comment.rpid) <= 1"
                    type="button"
                    @click="goToPrevReplyPage(comment)"
                  >
                    上一页
                  </button>

                  <span class="text-xs text-text-tertiary">
                    第 {{ getCurrentReplyPage(comment.rpid) }} / {{ getReplyTotalPages(comment) }} 页
                  </span>

                  <button
                    class="text-xs text-text-primary px-2 py-1 rounded bg-bg-secondary transition-colors hover:bg-bg-tertiary disabled:opacity-50 disabled:cursor-not-allowed"
                    :disabled="getCurrentReplyPage(comment.rpid) >= getReplyTotalPages(comment)"
                    type="button"
                    @click="goToNextReplyPage(comment)"
                  >
                    下一页
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </TransitionGroup>

      <!-- 分页控件 -->
      <div v-if="totalPages > 1" class="mt-6 flex gap-2 items-center justify-center">
        <Button
          variant="secondary"
          :disabled="!hasPrevPage"
          @click="emit('prevPage')"
        >
          上一页
        </Button>

        <div class="flex gap-1">
          <!-- 第一页 -->
          <button
            v-if="currentPage > 3"
            class="text-sm px-3 py-1 rounded transition-colors"
            :class="currentPage === 1 ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
            @click="emit('goToPage', 1)"
          >
            1
          </button>
          <span v-if="currentPage > 4" class="text-text-tertiary px-2 py-1">...</span>

          <!-- 当前页附近的页码 -->
          <button
            v-for="page in [currentPage - 2, currentPage - 1, currentPage, currentPage + 1, currentPage + 2].filter(p => p >= 1 && p <= totalPages)"
            :key="page"
            class="text-sm px-3 py-1 rounded transition-colors"
            :class="page === currentPage ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
            @click="emit('goToPage', page)"
          >
            {{ page }}
          </button>

          <!-- 最后一页 -->
          <span v-if="currentPage < totalPages - 3" class="text-text-tertiary px-2 py-1">...</span>
          <button
            v-if="currentPage < totalPages - 2"
            class="text-sm px-3 py-1 rounded transition-colors"
            :class="currentPage === totalPages ? 'bg-primary text-white' : 'bg-bg-secondary text-text-primary hover:bg-bg-tertiary'"
            @click="emit('goToPage', totalPages)"
          >
            {{ totalPages }}
          </button>
        </div>

        <Button
          variant="secondary"
          :disabled="!hasNextPage"
          @click="emit('nextPage')"
        >
          下一页
        </Button>

        <span class="text-sm text-text-tertiary ml-4">
          第 {{ currentPage }} / {{ totalPages }} 页
        </span>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="image-preview">
        <div
          v-if="previewImageUrl"
          class="p-4 bg-black/85 flex items-center inset-0 justify-center fixed z-80"
          @click="closeImagePreview"
        >
          <div class="flex gap-2 items-center right-4 top-4 absolute">
            <button
              type="button"
              class="text-white rounded-full bg-black/50 flex h-9 w-9 transition-colors items-center justify-center hover:bg-black/70 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="previewZoom <= minPreviewZoom"
              @click.stop="zoomOutPreview"
            >
              <ZoomOut :size="18" />
            </button>
            <button
              type="button"
              class="text-white rounded-full bg-black/50 flex h-9 w-9 transition-colors items-center justify-center hover:bg-black/70 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="previewZoom >= maxPreviewZoom"
              @click.stop="zoomInPreview"
            >
              <ZoomIn :size="18" />
            </button>
            <button
              type="button"
              class="text-white rounded-full bg-black/50 flex h-9 w-9 transition-colors items-center justify-center hover:bg-black/70"
              @click.stop="closeImagePreview"
            >
              <X :size="18" />
            </button>
          </div>

          <div class="h-full w-full overflow-auto" @click.stop>
            <div class="p-8 flex min-h-full min-w-full items-center justify-center">
              <SmartImage
                :src="previewImageUrl"
                :alt="previewImageAlt"
                class="max-h-[85vh] max-w-[92vw] transition-transform duration-200 object-contain"
                :style="{ transform: `scale(${previewZoom})`, transformOrigin: 'center center' }"
              />
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.comment-list-move,
.comment-list-enter-active,
.comment-list-leave-active {
  transition: all 0.3s ease;
}

.comment-list-enter-from,
.comment-list-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.comment-list-leave-active {
  position: absolute;
}

.image-preview-enter-active,
.image-preview-leave-active {
  transition: opacity 0.2s ease;
}

.image-preview-enter-from,
.image-preview-leave-to {
  opacity: 0;
}
</style>
