<template>
  <div class="card">
    <div class="flex flex-wrap items-center justify-between mb-6 gap-4">
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-xl flex items-center justify-center shadow-lg flex-shrink-0">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </div>
        <div class="flex-shrink-0">
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">批量下载</h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">{{ tasks.length }} 个批量任务</p>
        </div>
      </div>
      <div class="flex items-center space-x-2 flex-shrink-0">
        <el-button @click="showCreateDialog = true" type="primary" :icon="Plus">创建批量下载</el-button>
        <el-button @click="loadTasks" :icon="Refresh">刷新</el-button>
      </div>
    </div>

    <!-- 批量任务列表 -->
    <div v-if="tasks.length > 0">
      <el-space direction="vertical" :size="12" fill>
        <el-card
          v-for="task in tasks"
          :key="task.id"
          shadow="hover"
          class="task-card"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <h3 class="font-semibold text-gray-900 dark:text-white">{{ task.task_name }}</h3>
                <el-tag
                  :type="task.status === 'completed' ? 'success' : task.status === 'failed' ? 'danger' : 'warning'"
                  size="small"
                >
                  {{ task.status === 'completed' ? '已完成' : task.status === 'failed' ? '失败' : '进行中' }}
                </el-tag>
              </div>

              <div class="flex items-center gap-6 text-sm text-gray-500 dark:text-gray-400 mb-3">
                <span>总数: {{ task.total_count }}</span>
                <span>完成: {{ task.completed_count }}</span>
                <span>失败: {{ task.failed_count }}</span>
                <span v-if="task.created_at">{{ formatDate(task.created_at) }}</span>
              </div>

              <!-- 进度条 -->
              <div v-if="task.status !== 'completed'" class="mb-3">
                <el-progress
                  :percentage="calculateProgress(task)"
                  :status="task.status === 'failed' ? 'exception' : 'success'"
                />
              </div>

              <!-- 操作按钮 -->
              <div class="flex items-center gap-2">
                <el-button @click="viewDetails(task)" type="primary" size="small" :icon="View">查看详情</el-button>
                <el-button @click="deleteTask(task.id)" type="danger" size="small" :icon="Delete" plain>删除</el-button>
              </div>
            </div>
          </div>
        </el-card>
      </el-space>
    </div>

    <!-- 空状态 -->
    <div v-else class="text-center py-12 text-gray-500 dark:text-gray-400">
      <svg class="mx-auto h-16 w-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
      </svg>
      <p class="text-lg font-medium">暂无批量下载任务</p>
      <p class="text-sm mt-2">点击"创建批量下载"开始批量下载应用</p>
    </div>

    <!-- 创建批量下载对话框 -->
    <el-dialog
      v-model="showCreateDialog"
      title="创建批量下载"
      width="500px"
      :close-on-click-modal="false"
    >
      <el-form :model="createForm" label-width="100px">
        <el-form-item label="任务名称">
          <el-input v-model="createForm.taskName" placeholder="输入任务名称" />
        </el-form-item>
        <el-form-item label="应用列表">
          <div class="text-sm text-gray-500 mb-2">
            在下载页选择账号、APPID 和版本后，点击"添加到批量下载"
          </div>
          <div v-if="draftItems.length > 0" class="w-full space-y-2">
            <div
              v-for="(item, index) in draftItems"
              :key="`${item.app_id}-${item.version || 'latest'}-${item.account_email}`"
              class="flex items-start justify-between gap-3 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 p-3"
            >
              <div class="min-w-0 flex-1">
                <p class="font-medium text-gray-900 dark:text-white truncate">{{ item.app_name || item.app_id }}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  App ID: {{ item.app_id }}
                  <span class="mx-1">|</span>
                  账号: {{ item.account_email }}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  版本: {{ item.version_label || item.version || '最新版本' }}
                </p>
              </div>
              <el-button @click="removeDraftItem(index)" type="danger" size="small" plain>移除</el-button>
            </div>
            <div class="flex justify-end">
              <el-button @click="clearDraftItems" size="small" plain>清空草稿</el-button>
            </div>
          </div>
          <div v-else class="text-sm text-gray-400">
            还没有批量下载草稿项
          </div>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="createBatchTask" :loading="creating">创建</el-button>
      </template>
    </el-dialog>

    <!-- 任务详情对话框 -->
    <el-dialog
      v-model="showDetailsDialog"
      title="批量下载详情"
      width="800px"
    >
      <div v-if="currentTask">
        <div class="mb-4">
          <h3 class="font-semibold mb-2">{{ currentTask.task_name }}</h3>
          <div class="flex items-center gap-4 text-sm text-gray-500">
            <span>总数: {{ currentTask.total_count }}</span>
            <span>完成: {{ currentTask.completed_count }}</span>
            <span>失败: {{ currentTask.failed_count }}</span>
          </div>
        </div>

        <div v-if="taskItems.length > 0">
          <h4 class="font-semibold mb-3">下载项目</h4>
          <el-space direction="vertical" :size="8" fill>
            <div
              v-for="item in taskItems"
              :key="item.id"
              class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
            >
              <div class="flex items-center justify-between">
                <div>
                  <p class="font-medium text-gray-900 dark:text-white">{{ item.app_name || item.app_id }}</p>
                  <p class="text-sm text-gray-500">
                    版本: {{ item.version || '未知' }} | 账号: {{ item.account_email }}
                  </p>
                </div>
                <div class="text-right">
                  <el-tag
                    :type="item.status === 'completed' ? 'success' : item.status === 'failed' ? 'danger' : 'warning'"
                    size="small"
                  >
                    {{ item.status === 'completed' ? '已完成' : item.status === 'failed' ? '失败' : '进行中' }}
                  </el-tag>
                  <p v-if="item.progress > 0" class="text-sm text-gray-500 mt-1">{{ item.progress }}%</p>
                </div>
              </div>
              <p v-if="item.error" class="text-sm text-red-500 mt-2">{{ item.error }}</p>
            </div>
          </el-space>
        </div>
      </div>

      <template #footer>
        <el-button @click="showDetailsDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { computed, ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Refresh, View, Delete } from '@element-plus/icons-vue'
import { useAppStore } from '../stores/app'

const API_BASE = '/api'
const appStore = useAppStore()

const tasks = ref([])
const showCreateDialog = ref(false)
const showDetailsDialog = ref(false)
const creating = ref(false)
const currentTask = ref(null)
const taskItems = ref([])

const createForm = ref({
  taskName: '',
  items: []
})

const draftItems = computed(() => appStore.batchDraftItems)

// 加载批量任务
const loadTasks = async () => {
  try {
    const response = await fetch(`${API_BASE}/batch-tasks`)
    const data = await response.json()
    if (data.ok) {
      tasks.value = data.data || []
    }
  } catch (error) {
    console.error('Failed to load batch tasks:', error)
    ElMessage.error('加载批量任务失败')
  }
}

// 创建批量任务
const removeDraftItem = (index) => {
  appStore.removeBatchDraftItem(index)
}

const clearDraftItems = () => {
  appStore.clearBatchDraftItems()
}

const createBatchTask = async () => {
  if (!createForm.value.taskName) {
    ElMessage.warning('请输入任务名称')
    return
  }

  if (draftItems.value.length === 0) {
    ElMessage.warning('请先从下载页添加至少一个批量下载项')
    return
  }

  creating.value = true
  try {
    const response = await fetch(`${API_BASE}/batch-download`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        task_name: createForm.value.taskName,
        items: draftItems.value.map(item => ({
          app_id: item.app_id,
          app_name: item.app_name,
          version: item.version,
          account_email: item.account_email
        }))
      })
    })
    const data = await response.json()

    if (data.ok) {
      ElMessage.success('批量任务创建成功')
      showCreateDialog.value = false
      createForm.value.taskName = ''
      appStore.clearBatchDraftItems()
      await loadTasks()
    } else {
      ElMessage.error(data.error || '创建批量任务失败')
    }
  } catch (error) {
    console.error('Failed to create batch task:', error)
    ElMessage.error('创建批量任务失败')
  } finally {
    creating.value = false
  }
}

// 查看任务详情
const viewDetails = async (task) => {
  currentTask.value = task
  try {
    const response = await fetch(`${API_BASE}/batch-tasks/${task.id}`)
    const data = await response.json()
    if (data.ok && data.data.items) {
      taskItems.value = data.data.items
    }
    showDetailsDialog.value = true
  } catch (error) {
    console.error('Failed to load task details:', error)
    ElMessage.error('加载任务详情失败')
  }
}

// 删除任务
const deleteTask = async (id) => {
  try {
    await ElMessageBox.confirm('确定要删除这个批量任务吗？', '确认删除', {
      type: 'warning'
    })

    const response = await fetch(`${API_BASE}/batch-tasks/${id}`, {
      method: 'DELETE'
    })
    const data = await response.json()

    if (data.ok) {
      ElMessage.success('删除成功')
      await loadTasks()
    } else {
      ElMessage.error(data.error || '删除失败')
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete task:', error)
      ElMessage.error('删除失败')
    }
  }
}

// 计算进度
const calculateProgress = (task) => {
  if (task.total_count === 0) return 0
  return Math.round((task.completed_count / task.total_count) * 100)
}

// 格式化日期
const formatDate = (dateString) => {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN')
}

onMounted(() => {
  loadTasks()
})
</script>

<style scoped>
.card {
  background: white;
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.dark .card {
  background: #1f2937;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.task-card {
  transition: all 0.2s ease;
}

.task-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
</style>
