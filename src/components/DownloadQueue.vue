<template>
  <div class="space-y-6">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 bg-gradient-to-br from-orange-500 to-red-500 rounded-xl flex items-center justify-center shadow-lg">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <line x1="8" y1="6" x2="21" y2="6" />
            <line x1="8" y1="12" x2="21" y2="12" />
            <line x1="8" y1="18" x2="21" y2="18" />
            <line x1="3" y1="6" x2="3.01" y2="6" />
            <line x1="3" y1="12" x2="3.01" y2="12" />
            <line x1="3" y1="18" x2="3.01" y2="18" />
          </svg>
        </div>
        <div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">下载队列</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">{{ queue.length }} 个任务 · {{ records.length }} 条记录</p>
        </div>
      </div>
      <div class="flex gap-2">
        <el-button size="small" plain @click="loadRecords">刷新</el-button>
        <el-button size="small" type="warning" plain @click="cleanupServerFiles">清理服务器文件</el-button>
      </div>
    </div>

    <section v-if="queue.length > 0" class="space-y-3">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-white">当前任务</h3>
      <div v-for="task in queue" :key="task.id" class="queue-row">
        <img :src="task.artworkUrl || 'https://via.placeholder.com/56'" :alt="task.appName" class="row-artwork">
        <div class="row-main">
          <div class="row-top">
            <div class="min-w-0">
              <div class="row-title">{{ task.appName }}</div>
              <div class="row-meta">{{ task.artistName || '未知开发者' }} · 版本 {{ task.version || '未知' }}</div>
            </div>
            <el-tag :type="statusTagType(task.status)" size="small">{{ statusLabel(task.status) }}</el-tag>
          </div>
          <div class="row-info">
            <span v-if="task.fileSize">大小 {{ formatFileSize(task.fileSize) }}</span>
            <span v-if="task.progress !== undefined">进度 {{ task.progress }}%</span>
            <span v-if="task.stage">阶段 {{ task.stage }}</span>
          </div>
          <el-progress v-if="task.status !== 'completed' && task.status !== 'failed' && task.progress !== undefined" :percentage="task.progress" :stroke-width="6" />
          <div v-if="task.error" class="row-error">{{ task.error }}</div>
          <div class="row-actions">
            <el-button v-if="task.status === 'completed' && task.downloadUrl" type="primary" size="small" @click="download(task.downloadUrl)">下载</el-button>
            <el-button v-if="task.status === 'completed' && task.installUrl" type="success" size="small" @click="install(task.installUrl)">安装</el-button>
            <el-button size="small" type="danger" plain @click="removeTask(task.id)">{{ task.status === 'completed' || task.status === 'failed' ? '移除' : '取消' }}</el-button>
          </div>
        </div>
      </div>
    </section>

    <section v-if="records.length > 0" class="space-y-3">
      <div class="flex items-center justify-between gap-3">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">下载记录</h3>
        <el-button size="small" type="danger" plain @click="clearAllRecords">清空记录</el-button>
      </div>
      <div v-for="record in records" :key="record.id" class="queue-row">
        <img :src="record.artworkUrl || 'https://via.placeholder.com/56'" :alt="record.appName" class="row-artwork">
        <div class="row-main">
          <div class="row-top">
            <div class="min-w-0">
              <div class="row-title">{{ record.appName || '未命名 IPA' }}</div>
              <div class="row-meta">{{ record.artistName || record.accountEmail || '未知来源' }} · 版本 {{ record.version || '未知' }}</div>
            </div>
            <el-tag :type="statusTagType(record.status)" size="small">{{ statusLabel(record.status) }}</el-tag>
          </div>
          <div class="row-info">
            <span v-if="record.fileSize">大小 {{ formatFileSize(record.fileSize) }}</span>
            <span>{{ formatDate(record.downloadDate || record.createdAt) }}</span>
            <span>{{ record.fileExists ? '文件在服务器' : '文件缺失' }}</span>
          </div>
          <div v-if="record.error" class="row-error">{{ record.error }}</div>
          <div class="row-actions">
            <el-button v-if="record.downloadUrl && record.fileExists" type="primary" size="small" @click="download(record.downloadUrl)">下载</el-button>
            <el-button v-if="record.installUrl && record.fileExists" type="success" size="small" @click="install(record.installUrl)">安装</el-button>
            <el-button size="small" type="danger" plain @click="removeRecord(record.id)">删除记录</el-button>
          </div>
        </div>
      </div>
    </section>

    <div v-if="queue.length === 0 && records.length === 0" class="text-center py-12 text-gray-500 dark:text-gray-400">
      <svg class="mx-auto h-16 w-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <p class="text-lg font-medium">暂无下载任务和记录</p>
      <p class="text-sm mt-2">完成后可在这里查看状态与操作</p>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

const API_BASE = '/api'

defineProps({
  queue: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['remove-item'])
const records = ref([])

const loadRecords = async () => {
  try {
    const response = await fetch(`${API_BASE}/download-records`, { credentials: 'include' })
    const data = await response.json()
    if (data.ok) {
      records.value = data.data || []
    } else {
      ElMessage.error(data.error || '加载记录失败')
    }
  } catch (error) {
    console.error('Failed to load download records:', error)
    ElMessage.error('加载记录失败')
  }
}

const removeRecord = async (id) => {
  try {
    await ElMessageBox.confirm('确定删除这条记录吗？', '确认删除', { type: 'warning' })
    const response = await fetch(`${API_BASE}/download-records/${id}`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || '删除失败')
    ElMessage.success('记录已删除')
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败')
    }
  }
}

const clearAllRecords = async () => {
  try {
    await ElMessageBox.confirm('确定清空全部下载记录吗？', '确认清空', {
      type: 'warning',
      confirmButtonText: '清空',
      cancelButtonText: '取消'
    })
    const response = await fetch(`${API_BASE}/download-records`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || '清空失败')
    ElMessage.success('记录已清空')
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '清空失败')
    }
  }
}

const cleanupServerFiles = async () => {
  try {
    await ElMessageBox.confirm('确定清理服务器上的下载目录吗？', '确认清理', {
      type: 'warning',
      confirmButtonText: '清理',
      cancelButtonText: '取消'
    })
    const response = await fetch(`${API_BASE}/cleanup-downloads`, {
      method: 'POST',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || '清理失败')
    ElMessage.success(`已释放 ${formatFileSize(data.data?.freed_bytes || 0)}`)
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '清理失败')
    }
  }
}

const removeTask = (id) => emit('remove-item', id)
const download = (url) => window.open(url, '_blank', 'noopener')
const install = (url) => { window.location.href = url }

const statusTagType = (status) => {
  if (status === 'completed' || status === 'ready') return 'success'
  if (status === 'failed' || status === 'error') return 'danger'
  return 'warning'
}

const statusLabel = (status) => {
  if (status === 'completed' || status === 'ready') return '已完成'
  if (status === 'failed' || status === 'error') return '失败'
  return '进行中'
}

const formatFileSize = (bytes) => {
  if (!bytes) return '未知'
  const units = ['B', 'KB', 'MB', 'GB']
  let value = bytes
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return `${value.toFixed(value >= 100 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}

const formatDate = (value) => {
  if (!value) return '未知时间'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

onMounted(loadRecords)
</script>

<style scoped>
.queue-row {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 16px;
  border-radius: 16px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: rgba(255, 255, 255, 0.88);
}

.dark .queue-row {
  background: rgba(17, 24, 39, 0.72);
  border-color: rgba(71, 85, 105, 0.45);
}

.row-artwork {
  width: 56px;
  height: 56px;
  flex-shrink: 0;
  border-radius: 14px;
  object-fit: cover;
  box-shadow: 0 8px 18px rgba(15, 23, 42, 0.12);
}

.row-main {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.row-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.row-title {
  font-size: 15px;
  font-weight: 600;
  color: #111827;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dark .row-title {
  color: #f9fafb;
}

.row-meta,
.row-info {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 14px;
  font-size: 12px;
  color: #6b7280;
}

.dark .row-meta,
.dark .row-info {
  color: #9ca3af;
}

.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.row-error {
  font-size: 12px;
  color: #dc2626;
}

@media (max-width: 767px) {
  .queue-row {
    padding: 14px;
  }

  .row-top {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
