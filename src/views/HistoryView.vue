<script lang="ts" setup>
import { Search, Trash2 } from 'lucide-vue-next'
import { ref } from 'vue'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import { useHistoryStore } from '@/stores/history'
import { formatFileSize, formatRelativeTime } from '@/utils/format'

const historyStore = useHistoryStore()

const searchQuery = ref('')

const sortOptions = [
  { value: 'date', label: '按日期' },
  { value: 'title', label: '按标题' },
  { value: 'size', label: '按大小' },
]

function handleSearch() {
  historyStore.setFilter(searchQuery.value)
}

function handleClearHistory() {
  if (confirm('确定要清空所有历史记录吗？')) {
    historyStore.clearHistory()
  }
}

function handleDeleteEntry(id: string) {
  historyStore.removeEntry(id)
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
        <Search :size="18" class="text-gray-400 left-3 top-1/2 absolute -translate-y-1/2" />
      </div>
      <Button variant="primary" @click="handleSearch">
        搜索
      </Button>
    </div>

    <!-- 空状态 -->
    <Card v-if="historyStore.filteredEntries.length === 0">
      <div class="text-center opacity-50 flex flex-col min-h-[30vh] items-center justify-center">
        <div class="mb-6 rounded-full bg-gray-50 flex h-32 w-32 items-center justify-center">
          <span class="text-5xl">🕰️</span>
        </div>
        <h2 class="text-xl text-gray-500 font-bold">
          {{ searchQuery ? '没有找到匹配的记录' : '暂无下载历史' }}
        </h2>
        <p class="text-sm text-gray-400 mt-2">
          下载完成的视频会出现在这里～
        </p>
      </div>
    </Card>

    <!-- 历史列表 -->
    <div v-else class="space-y-3">
      <Card
        v-for="entry in historyStore.filteredEntries"
        :key="entry.id"
        class="transition-shadow hover:shadow-md"
      >
        <div class="flex gap-4">
          <!-- 缩略图 -->
          <img
            :src="entry.thumbnail"
            :alt="entry.title"
            class="rounded-2xl h-20 w-32 object-cover"
          >

          <!-- 信息 -->
          <div class="flex-1 min-w-0">
            <h3 class="text-base text-text-primary font-semibold mb-1 truncate">
              {{ entry.title }}
            </h3>
            <div class="text-sm text-text-secondary flex gap-4 items-center">
              <span>{{ entry.quality }}</span>
              <span>{{ formatFileSize(entry.size) }}</span>
              <span>{{ formatRelativeTime(entry.downloadDate) }}</span>
            </div>
            <p class="text-xs text-text-tertiary mt-1 truncate">
              {{ entry.filePath }}
            </p>
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2 items-center">
            <Button variant="secondary" @click="handleDeleteEntry(entry.id)">
              <Trash2 :size="16" />
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
