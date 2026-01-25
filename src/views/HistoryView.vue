<script lang="ts" setup>
import { Search, Trash2 } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useHistoryStore } from '@/stores/history'
import { formatFileSize, formatRelativeTime } from '@/utils/format'
import { formatAudioQuality, formatVideoQuality } from '@/utils/quality'

const historyStore = useHistoryStore()
const router = useRouter()

const searchQuery = ref('')

// 获取历史记录的标签
function getHistoryTags(entry: any): string[] {
  const tags: string[] = []

  if (entry.videoOnly) {
    tags.push('仅视频')
  }
  else if (entry.audioOnly) {
    tags.push('仅音频')
  }

  // 添加质量信息
  if (!entry.audioOnly) {
    tags.push(formatVideoQuality(entry.videoQuality))
  }
  if (!entry.videoOnly) {
    tags.push(formatAudioQuality(entry.audioQuality))
  }

  // 添加评论标签
  if (entry.commentFilePath) {
    tags.push('评论')
  }

  return tags
}

// Load history on mount
onMounted(async () => {
  await historyStore.loadHistory()
})

function handleSearch() {
  historyStore.setFilter(searchQuery.value)
}

async function handleDeleteEntry(id: string) {
  await historyStore.removeEntry(id)
}

function handlePreview(id: string) {
  router.push(`/preview/${id}`)
}
</script>

<template>
  <div class="page-container">
    <!-- 搜索和筛选 -->
    <div class="mb-5 flex gap-3">
      <div class="flex-1 relative">
        <Input
          v-model="searchQuery"
          placeholder="搜索标题或链接..."
          class="pl-10 flex-1"
          @keyup.enter="handleSearch"
        />
        <Search :size="18" class="text-text-tertiary left-3 top-1/2 absolute -translate-y-1/2" />
      </div>
      <Button variant="primary" @click="handleSearch">
        搜索
      </Button>
    </div>

    <!-- 空状态 -->
    <div v-if="historyStore.filteredEntries.length === 0">
      <div class="text-center opacity-50 flex flex-col min-h-[30vh] items-center justify-center">
        <div class="mb-6 rounded-full bg-bg-tertiary flex h-32 w-32 items-center justify-center">
          <span class="text-5xl">🕰️</span>
        </div>
        <h2 class="text-xl text-text-secondary font-bold">
          {{ searchQuery ? '没有找到匹配的记录' : '暂无下载历史' }}
        </h2>
        <p class="text-sm text-text-tertiary mt-2">
          下载完成的视频会出现在这里～
        </p>
      </div>
    </div>

    <!-- 历史列表 -->
    <div v-else class="space-y-3">
      <Card
        v-for="entry in historyStore.filteredEntries"
        :key="entry.id"
        class="cursor-pointer transition-shadow hover:shadow-md"
        @click="handlePreview(entry.id)"
      >
        <div class="flex gap-4">
          <!-- 缩略图 -->
          <img
            :src="entry.thumbnail"
            :alt="entry.title"
            class="rounded h-20 w-32 object-cover"
          >

          <!-- 信息 -->
          <div class="flex-1 min-w-0">
            <h3 class="text-base text-text-primary font-semibold mb-1 truncate">
              {{ entry.title }}
            </h3>

            <!-- 质量标签 -->
            <div class="mb-2 flex flex-wrap gap-1">
              <span
                v-for="tag in getHistoryTags(entry)"
                :key="tag"
                class="text-xs text-text-secondary px-2 py-0.5 rounded bg-bg-tertiary"
              >
                {{ tag }}
              </span>
            </div>

            <div class="text-sm text-text-secondary flex gap-4 items-center">
              <span>{{ formatFileSize(entry.size) }}</span>
              <span>{{ formatRelativeTime(entry.downloadDate) }}</span>
            </div>
            <p class="text-xs text-text-tertiary mt-1 truncate">
              {{ entry.filePath }}
            </p>
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2 items-center" @click.stop>
            <Button variant="secondary" @click="handleDeleteEntry(entry.id)">
              <Trash2 :size="16" />
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
