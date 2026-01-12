<script lang="ts" setup>
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
    <div class="mb-4 flex items-center justify-between">
      <h2 class="text-2xl text-text-primary font-bold">
        下载历史
      </h2>
      <div class="flex gap-2 items-center">
        <span class="text-sm text-text-secondary">
          共 {{ historyStore.totalCount }} 条记录
        </span>
        <Button
          v-if="historyStore.totalCount > 0"
          variant="secondary"
          @click="handleClearHistory"
        >
          清空历史
        </Button>
      </div>
    </div>

    <!-- 搜索和筛选 -->
    <Card class="mb-4">
      <div class="flex gap-3">
        <Input
          v-model="searchQuery"
          placeholder="搜索标题或链接..."
          class="flex-1"
          @keyup.enter="handleSearch"
        />
        <select
          v-model="historyStore.sortBy"
          class="input-base w-32"
          @change="historyStore.setSortBy(historyStore.sortBy)"
        >
          <option
            v-for="option in sortOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </select>
        <Button variant="primary" @click="handleSearch">
          搜索
        </Button>
      </div>
    </Card>

    <!-- 空状态 -->
    <Card v-if="historyStore.filteredEntries.length === 0">
      <div class="py-12 text-center">
        <div class="i-carbon-time text-6xl text-text-tertiary mx-auto mb-4" />
        <p class="text-text-secondary">
          {{ searchQuery ? '没有找到匹配的记录' : '暂无下载历史' }}
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
            class="rounded-lg h-20 w-32 object-cover"
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
              <div class="i-carbon-trash-can" />
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
