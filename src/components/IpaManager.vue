<template>
  <div class="space-y-6">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 bg-gradient-to-br from-violet-500 to-indigo-500 rounded-xl flex items-center justify-center shadow-lg">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 16V4m0 12l-4-4m4 4l4-4M5 20h14" />
          </svg>
        </div>
        <div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">IPA 管理</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">管理服务器上的 IPA 文件</p>
        </div>
      </div>
      <el-button :loading="loading" plain @click="loadArtifacts">刷新</el-button>
    </div>

    <div v-if="artifacts.length > 0" class="space-y-3">
      <div v-for="item in artifacts" :key="item.id" class="artifact-row">
        <AppArtwork :src="item.artworkUrl" :alt="item.appName" :label="item.appName || item.fileName" />
        <div class="artifact-main">
          <div class="artifact-top">
            <div class="min-w-0">
              <div class="artifact-title">{{ item.appName || item.fileName }}</div>
              <div class="artifact-meta">
                <span>{{ item.artistName || '未知开发者' }}</span>
                <span>版本 {{ item.version || '未知' }}</span>
                <span>账号 {{ item.accountEmail || '未知账号' }}</span>
                <span>{{ formatFileSize(item.fileSize) }}</span>
              </div>
            </div>
            <el-tag size="small" type="info">{{ formatDate(item.modifiedAt) }}</el-tag>
          </div>
          <div class="artifact-path">{{ item.filePath }}</div>
          <div class="artifact-actions">
            <el-button type="primary" size="small" @click="download(item.downloadUrl)">下载</el-button>
            <el-button type="success" size="small" :disabled="!item.installUrl" @click="item.installUrl && install(item.installUrl)">安装</el-button>
            <el-button type="danger" size="small" plain @click="removeArtifact(item)">删除</el-button>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-14 text-gray-500 dark:text-gray-400">
      <svg class="mx-auto h-16 w-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
      </svg>
      <p class="text-lg font-medium">暂无 IPA 文件</p>
      <p class="text-sm mt-2">下载完成后会出现在这里</p>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import AppArtwork from './AppArtwork.vue'

const API_BASE = '/api'
const artifacts = ref([])
const loading = ref(false)

const loadArtifacts = async () => {
  loading.value = true
  try {
    const response = await fetch(`${API_BASE}/ipa-files`, { credentials: 'include' })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || '加载失败')
    artifacts.value = data.data || []
  } catch (error) {
    ElMessage.error(error.message || '加载失败')
  } finally {
    loading.value = false
  }
}

const download = (url) => window.open(url, '_blank', 'noopener')
const install = (url) => { window.location.href = url }

const removeArtifact = async (item) => {
  try {
    await ElMessageBox.confirm(`确定删除 ${item.fileName} 吗？`, '确认删除', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消'
    })
    const response = await fetch(`${API_BASE}/ipa-files/${item.id}`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || '删除失败')
    ElMessage.success('IPA 已删除')
    await loadArtifacts()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败')
    }
  }
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
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

onMounted(loadArtifacts)
</script>

<style scoped>
.artifact-row {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 16px;
  border-radius: 16px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: rgba(255, 255, 255, 0.88);
}

.dark .artifact-row {
  background: rgba(17, 24, 39, 0.72);
  border-color: rgba(71, 85, 105, 0.45);
}

.artifact-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.artifact-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.artifact-title {
  font-size: 15px;
  font-weight: 600;
  color: #111827;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dark .artifact-title {
  color: #f9fafb;
}

.artifact-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 14px;
  font-size: 12px;
  color: #6b7280;
}

.dark .artifact-meta {
  color: #9ca3af;
}

.artifact-path {
  font-size: 12px;
  color: #64748b;
  word-break: break-all;
}

.dark .artifact-path {
  color: #94a3b8;
}

.artifact-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

@media (max-width: 767px) {
  .artifact-row {
    padding: 14px;
  }

  .artifact-top {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
