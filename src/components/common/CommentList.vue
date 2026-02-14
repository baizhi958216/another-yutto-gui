<script lang="ts" setup>
import type { SortType } from '@/composables/useComments'
import type { Comment } from '@/types'
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
              <div class="text-sm text-text-primary whitespace-pre-wrap break-words">
                {{ comment.content }}
              </div>
              <!-- 评论图片 -->
              <div v-if="comment.pictures && comment.pictures.length > 0" class="mt-2 flex flex-wrap gap-2">
                <SmartImage
                  v-for="(pic, idx) in comment.pictures"
                  :key="idx"
                  :src="pic.img_src"
                  class="rounded max-h-32 object-cover"
                />
              </div>
              <!-- 点赞数 -->
              <div class="text-xs text-text-tertiary mt-2 flex gap-1 w-fit items-center justify-center">
                <div class="i-carbon:thumbs-up-filled mt--0.5" />
                {{ comment.like }}
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
</style>
