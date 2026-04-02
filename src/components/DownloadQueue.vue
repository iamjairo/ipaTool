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
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">Download Queue</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">{{ currentTasks.length }}  active tasks · {{ records.length }}  records · Storage used: {{ formatStorageM(totalStorageBytes) }}</p>
        </div>
      </div>
      <div class="flex gap-2">
        <el-button size="small" plain @click="loadRecords">Refresh</el-button>
        <el-button size="small" type="warning" plain @click="cleanupServerFiles">Clean Server Files</el-button>
      </div>
    </div>

    <section v-if="currentTasks.length > 0" class="space-y-3">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Active Tasks</h3>
      <div v-for="task in currentTasks" :key="task.id" class="queue-row">
        <AppArtwork :src="task.artworkUrl" :alt="task.appName" :label="task.appName" />
        <div class="row-main">
          <div class="row-top">
            <div class="min-w-0">
              <div class="row-title">{{ task.appName }}</div>
              <div class="row-meta">
                <span>{{ task.artistName || 'Unknown Developer' }}</span>
                <span>Version {{ task.version || 'Unknown' }}</span>
                <span>Account {{ task.accountEmail || task.account?.email || 'Unknown Account' }}</span>
              </div>
            </div>
            <el-tag :type="statusTagType(task.status)" size="small">{{ statusLabel(task.status) }}</el-tag>
          </div>
          <div class="row-info">
            <span v-if="task.fileSize">Size {{ formatFileSize(task.fileSize) }}</span>
            <span v-if="task.progress !== undefined">Progress {{ task.progress }}%</span>
            <span v-if="task.stage">Stage {{ task.stage }}</span>
          </div>
          <el-progress v-if="task.status !== 'completed' && task.status !== 'failed' && task.progress !== undefined" :percentage="task.progress" :stroke-width="6" />
          <div v-if="task.error" class="row-error">{{ task.error }}</div>
          <div class="row-actions">
            <el-button v-if="task.status === 'completed' && task.downloadUrl" type="primary" size="small" @click="download(task.downloadUrl)">Download</el-button>
            <el-button v-if="task.status === 'completed' && task.otaInstallable && task.installUrl" type="success" size="small" @click="install(task.installUrl)">Install</el-button>
            <el-tag v-else-if="task.status === 'completed' && task.installMethod === 'download_only'" size="small" type="info">Download Only</el-tag>
            <el-button size="small" type="danger" plain @click="removeTask(task.id)">{{ task.status === 'completed' || task.status === 'failed' ? 'Remove' : 'Cancel' }}</el-button>
          </div>
        </div>
      </div>
    </section>

    <section v-if="records.length > 0" class="space-y-3">
      <div class="flex items-center justify-between gap-3">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Download History</h3>
        <el-button size="small" type="danger" plain @click="clearAllRecords">Clear History</el-button>
      </div>
      <div v-for="record in records" :key="record.id" class="queue-row">
        <AppArtwork :src="record.artworkUrl" :alt="record.appName" :label="record.appName || 'IPA'" />
        <div class="row-main">
          <div class="row-top">
            <div class="min-w-0">
              <div class="row-title">{{ record.appName || 'Unnamed IPA' }}</div>
              <div class="row-meta">
                <span>{{ record.artistName || 'Unknown Developer' }}</span>
                <span>Version {{ record.version || 'Unknown' }}</span>
                <span>Account {{ record.accountEmail || 'Unknown Account' }}</span>
              </div>
            </div>
            <el-tag :type="statusTagType(record.status)" size="small">{{ statusLabel(record.status) }}</el-tag>
          </div>
          <div class="row-info">
            <span v-if="record.fileSize">Size {{ formatFileSize(record.fileSize) }}</span>
            <span>{{ formatDate(record.downloadDate || record.createdAt) }}</span>
            <span>{{ record.fileExists ? 'File on server' : 'File missing' }}</span>
          </div>
          <div v-if="record.error" class="row-error">{{ record.error }}</div>
          <div class="row-actions">
            <el-button v-if="record.downloadUrl && record.fileExists" type="primary" size="small" @click="download(record.downloadUrl)">Download</el-button>

            <el-button v-if="record.fileExists && record.otaInstallable && record.installUrl" type="success" size="small" @click="install(record.installUrl)">Install</el-button>
            <el-tooltip v-else-if="record.fileExists && record.installMethod === 'download_only'" :content="record.inspection?.summary || ''" :disabled="!record.inspection?.summary" placement="top">
              <span>
                <el-tag size="small" type="info">Download Only</el-tag>
              </span>
            </el-tooltip>

            <el-button v-if="record.fileExists" size="small" type="warning" plain @click="cleanupRecordFile(record)">Clean Package</el-button>
            <el-button size="small" type="danger" plain @click="removeRecord(record.id)">Delete Record</el-button>
          </div>
        </div>
      </div>
    </section>

    <div v-if="currentTasks.length === 0 && records.length === 0" class="text-center py-12 text-gray-500 dark:text-gray-400">
      <svg class="mx-auto h-16 w-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <p class="text-lg font-medium">No download tasks or records</p>
      <p class="text-sm mt-2">Completed tasks will appear here</p>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import AppArtwork from './AppArtwork.vue'
import { useAppStore } from '../stores/app'

const API_BASE = '/api'
const props = defineProps({
  queue: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['remove-item'])
const appStore = useAppStore()
const records = ref([])
const pollTimers = new Map()
const currentTasks = computed(() => props.queue.filter(task => !['completed', 'ready'].includes(task?.status)))
const totalStorageBytes = computed(() => records.value.reduce((sum, item) => {
  if (!item?.fileExists) return sum
  return sum + Number(item.fileSize || 0)
}, 0))

const loadRecords = async () => {
  try {
    const response = await fetch(`${API_BASE}/download-records`, { credentials: 'include' })
    const data = await response.json()
    if (data.ok) {
      records.value = data.data || []
    } else {
      ElMessage.error(data.error || 'Failed to load records')
    }
  } catch (error) {
    console.error('Failed to load download records:', error)
    ElMessage.error('Failed to load records')
  }
}

const removeRecord = async (id) => {
  try {
    await ElMessageBox.confirm('Are you sure you want to delete this record?', 'Confirm Delete', { type: 'warning' })
    const response = await fetch(`${API_BASE}/download-records/${id}`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || 'Delete failed')
    ElMessage.success('Record deleted')
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || 'Delete failed')
    }
  }
}

const clearAllRecords = async () => {
  try {
    await ElMessageBox.confirm('Are you sure you want to clear all download records?', 'Confirm Clear', {
      type: 'warning',
      confirmButtonText: 'Clear',
      cancelButtonText: 'Cancel'
    })
    const response = await fetch(`${API_BASE}/download-records`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || 'Failed to clear records')
    ElMessage.success('Records cleared')
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || 'Failed to clear records')
    }
  }
}

const cleanupRecordFile = async (record) => {
  try {
    await ElMessageBox.confirm(`Are you sure you want to clean ${record.appName || record.filePath || 'this package'}?`, 'Confirm Cleanup', {
      type: 'warning',
      confirmButtonText: 'Clean Package',
      cancelButtonText: 'Cancel'
    })
    const response = await fetch(`${API_BASE}/download-records/${record.id}/file`, {
      method: 'DELETE',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || 'Cleanup failed')
    ElMessage.success(`Cleaned ${formatStorageM(data.data?.freed_bytes || 0)}`)
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || 'Cleanup failed')
    }
  }
}

const cleanupServerFiles = async () => {
  try {
    await ElMessageBox.confirm('Are you sure you want to clean the server download directory?', 'Confirm Cleanup', {
      type: 'warning',
      confirmButtonText: 'Clean',
      cancelButtonText: 'Cancel'
    })
    const response = await fetch(`${API_BASE}/cleanup-downloads`, {
      method: 'POST',
      credentials: 'include'
    })
    const data = await response.json()
    if (!data.ok) throw new Error(data.error || 'Cleanup failed')
    ElMessage.success(`Freed ${formatFileSize(data.data?.freed_bytes || 0)}`)
    await loadRecords()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || 'Cleanup failed')
    }
  }
}

const isFinalStatus = (status) => ['completed', 'ready', 'failed', 'error'].includes(status)

const stopTaskPolling = (taskId) => {
  const timer = pollTimers.get(taskId)
  if (timer) {
    clearInterval(timer)
    pollTimers.delete(taskId)
  }
}

const markTaskInterrupted = (taskId, message = 'Task is no longer valid, possibly due to server restart or page navigation. Please re-initiate the download.') => {
  stopTaskPolling(taskId)
  appStore.updateQueueItem(taskId, {
    status: 'failed',
    stage: 'interrupted',
    error: message
  })
}

const syncTaskSnapshot = async (taskId, snapshot) => {
  const updates = {
    stage: snapshot.stage || '',
    progress: snapshot.progress ?? 0,
    error: snapshot.error || '',
    status: snapshot.status === 'ready' ? 'completed' : snapshot.status,
    packageKind: snapshot.packageKind,
    otaInstallable: snapshot.otaInstallable,
    installMethod: snapshot.installMethod,
    inspection: snapshot.inspection
  }

  if (snapshot.status === 'ready') {
    updates.progress = 100
    updates.downloadUrl = snapshot.downloadUrl
    updates.installUrl = snapshot.installUrl
    updates.fileSize = snapshot.fileSize || 0
    stopTaskPolling(taskId)
    await loadRecords()
  } else if (snapshot.status === 'failed') {
    stopTaskPolling(taskId)
  }

  appStore.updateQueueItem(taskId, updates)
}

const pollTaskStatus = async (taskId) => {
  try {
    const response = await fetch(`${API_BASE}/job-info?jobId=${encodeURIComponent(taskId)}`, {
      credentials: 'include'
    })

    if (response.status === 404) {
      markTaskInterrupted(taskId)
      return
    }

    const data = await response.json()
    if (!response.ok || !data.ok || !data.data) {
      if (response.status >= 400) {
        markTaskInterrupted(taskId, data?.error || 'Failed to get task status, please re-initiate the download')
      }
      return
    }

    await syncTaskSnapshot(taskId, data.data)
  } catch (error) {
    console.error('Failed to poll task status:', error)
  }
}

const ensureTaskPolling = (task) => {
  if (!task?.id || isFinalStatus(task.status) || pollTimers.has(task.id)) return

  pollTaskStatus(task.id)
  const timer = setInterval(() => pollTaskStatus(task.id), 1500)
  pollTimers.set(task.id, timer)
}

const syncActiveTaskPolling = () => {
  const activeIds = new Set()

  for (const task of props.queue) {
    if (task?.id && !isFinalStatus(task.status)) {
      activeIds.add(task.id)
      ensureTaskPolling(task)
    }
  }

  for (const taskId of pollTimers.keys()) {
    if (!activeIds.has(taskId)) {
      stopTaskPolling(taskId)
    }
  }
}

const removeTask = (id) => {
  stopTaskPolling(id)
  emit('remove-item', id)
}
const download = (url) => window.open(url, '_blank', 'noopener')

const rewriteToCurrentOrigin = (rawUrl) => {
  const url = new URL(rawUrl, window.location.origin)
  url.protocol = window.location.protocol
  url.host = window.location.host
  return url.toString()
}

const buildInstallUrl = (installUrl) => {
  if (!installUrl) return null

  try {
    if (installUrl.startsWith('itms-services://')) {
      const itmsMatch = installUrl.match(/itms-services:\/\/\?action=download-manifest&url=(.+)/)
      if (!itmsMatch) return installUrl

      const manifestUrl = rewriteToCurrentOrigin(decodeURIComponent(itmsMatch[1]))
      return `itms-services://?action=download-manifest&url=${encodeURIComponent(manifestUrl)}`
    }

    const url = new URL(installUrl, window.location.origin)
    if (url.pathname === '/api/public/install' || url.pathname === '/api/install') {
      const manifest = url.searchParams.get('manifest')
      if (manifest) {
        const rewrittenManifest = rewriteToCurrentOrigin(manifest)
        return `itms-services://?action=download-manifest&url=${encodeURIComponent(rewrittenManifest)}`
      }
      return installUrl
    }

    return rewriteToCurrentOrigin(installUrl)
  } catch {
    return installUrl
  }
}

const install = (installUrl) => {
  const url = buildInstallUrl(installUrl)
  if (url) {
    window.location.href = url
  }
}

const statusTagType = (status) => {
  if (status === 'completed' || status === 'ready') return 'success'
  if (status === 'failed' || status === 'error') return 'danger'
  return 'warning'
}

const statusLabel = (status) => {
  if (status === 'completed' || status === 'ready') return 'Completed'
  if (status === 'failed' || status === 'error') return 'Failed'
  return 'In Progress'
}

const formatFileSize = (bytes) => {
  if (!bytes) return 'Unknown'
  const units = ['B', 'KB', 'MB', 'GB']
  let value = bytes
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return `${value.toFixed(value >= 100 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}

const formatStorageM = (bytes) => `${(Number(bytes || 0) / 1024 / 1024).toFixed(1)} M`

const formatDate = (value) => {
  if (!value) return 'Unknown'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

watch(
  () => props.queue.map(task => `${task?.id}:${task?.status}:${task?.progress}:${task?.stage}`),
  () => {
    syncActiveTaskPolling()
  },
  { immediate: true }
)

onMounted(() => {
  loadRecords()
  syncActiveTaskPolling()
})

onBeforeUnmount(() => {
  for (const taskId of [...pollTimers.keys()]) {
    stopTaskPolling(taskId)
  }
})
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
  align-items: center;
}

.row-actions :deep(.el-button) {
  margin: 0;
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
