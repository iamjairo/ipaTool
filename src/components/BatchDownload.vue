<template>
  <div class="card">
    <div class="flex flex-wrap items-center justify-between mb-6 gap-4">
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-xl flex items-center justify-center shadow-lg flex-shrink-0">
          <svg
            class="w-6 h-6 text-white"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
            />
          </svg>
        </div>
        <div class="flex-shrink-0">
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">
            Batch Download
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {{ tasks.length }}  batch tasks
          </p>
        </div>
      </div>
      <div class="flex items-center space-x-2 flex-shrink-0">
        <el-button
          type="primary"
          :icon="Plus"
          @click="showCreateDialog = true"
        >
          Create Batch Download
        </el-button>
        <el-button
          :icon="Refresh"
          @click="loadTasks"
        >
          Refresh
        </el-button>
      </div>
    </div>

    <!-- Batch Task List -->
    <div v-if="tasks.length > 0">
      <el-space
        direction="vertical"
        :size="12"
        fill
      >
        <el-card
          v-for="task in tasks"
          :key="task.id"
          shadow="hover"
          class="task-card"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <h3 class="font-semibold text-gray-900 dark:text-white">
                  {{ task.task_name }}
                </h3>
                <el-tag
                  :type="task.status === 'completed' ? 'success' : task.status === 'failed' ? 'danger' : 'warning'"
                  size="small"
                >
                  {{ task.status === 'completed' ? 'Completed' : task.status === 'failed' ? 'Failed' : 'In Progress' }}
                </el-tag>
              </div>

              <div class="flex items-center gap-6 text-sm text-gray-500 dark:text-gray-400 mb-3">
                <span>Total: {{ task.total_count }}</span>
                <span>Completed: {{ task.completed_count }}</span>
                <span>Failed: {{ task.failed_count }}</span>
                <span v-if="task.created_at">{{ formatDate(task.created_at) }}</span>
              </div>

              <!-- Progress bar -->
              <div
                v-if="task.status !== 'completed'"
                class="mb-3"
              >
                <el-progress
                  :percentage="calculateProgress(task)"
                  :status="task.status === 'failed' ? 'exception' : 'success'"
                />
              </div>

              <!-- Action buttons -->
              <div class="flex items-center gap-2">
                <el-button
                  type="primary"
                  size="small"
                  :icon="View"
                  @click="viewDetails(task)"
                >
                  View Details
                </el-button>
                <el-button
                  type="danger"
                  size="small"
                  :icon="Delete"
                  plain
                  @click="deleteTask(task.id)"
                >
                  Delete
                </el-button>
              </div>
            </div>
          </div>
        </el-card>
      </el-space>
    </div>

    <!-- Empty State -->
    <div
      v-else
      class="text-center py-12 text-gray-500 dark:text-gray-400"
    >
      <svg
        class="mx-auto h-16 w-16 mb-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 6h16M4 10h16M4 14h16M4 18h16"
        />
      </svg>
      <p class="text-lg font-medium">
        No batch download tasks
      </p>
      <p class="text-sm mt-2">
        Click "Create Batch Download" to start downloading apps in bulk
      </p>
    </div>

    <!-- Create Batch Download Dialog -->
    <el-dialog
      v-model="showCreateDialog"
      title="Create Batch Download"
      width="500px"
      :close-on-click-modal="false"
    >
      <el-form
        :model="createForm"
        label-width="100px"
      >
        <el-form-item label="Task Name">
          <el-input
            v-model="createForm.taskName"
            placeholder="Enter task name"
          />
        </el-form-item>
        <el-form-item label="App List">
          <div class="text-sm text-gray-500 mb-2">
            Select an account, App ID and version on the Download page, then click "Add to Batch Download"
          </div>
          <div
            v-if="draftItems.length > 0"
            class="w-full space-y-2"
          >
            <div
              v-for="(item, index) in draftItems"
              :key="`${item.app_id}-${item.version || 'latest'}-${item.account_email}`"
              class="flex items-start justify-between gap-3 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 p-3"
            >
              <div class="min-w-0 flex-1">
                <p class="font-medium text-gray-900 dark:text-white truncate">
                  {{ item.app_name || item.app_id }}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  App ID: {{ item.app_id }}
                  <span class="mx-1">|</span>
                  Account: {{ item.account_email }}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  Version: {{ item.version_label || item.version || 'Latest Version' }}
                </p>
              </div>
              <el-button
                type="danger"
                size="small"
                plain
                @click="removeDraftItem(index)"
              >
                Remove
              </el-button>
            </div>
            <div class="flex justify-end">
              <el-button
                size="small"
                plain
                @click="clearDraftItems"
              >
                Clear Drafts
              </el-button>
            </div>
          </div>
          <div
            v-else
            class="text-sm text-gray-400"
          >
No batch download draft items yet
          </div>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showCreateDialog = false">
          Cancel
        </el-button>
        <el-button
          type="primary"
          :loading="creating"
          @click="createBatchTask"
        >
          Create
        </el-button>
      </template>
    </el-dialog>

    <!-- Task Details Dialog -->
    <el-dialog
      v-model="showDetailsDialog"
      title="Batch Download Details"
      width="800px"
    >
      <div v-if="currentTask">
        <div class="mb-4">
          <h3 class="font-semibold mb-2">
            {{ currentTask.task_name }}
          </h3>
          <div class="flex items-center gap-4 text-sm text-gray-500">
            <span>Total: {{ currentTask.total_count }}</span>
            <span>Completed: {{ currentTask.completed_count }}</span>
            <span>Failed: {{ currentTask.failed_count }}</span>
          </div>
        </div>

        <div v-if="taskItems.length > 0">
          <h4 class="font-semibold mb-3">
            Download Items
          </h4>
          <el-space
            direction="vertical"
            :size="8"
            fill
          >
            <div
              v-for="item in taskItems"
              :key="item.id"
              class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
            >
              <div class="flex items-center justify-between">
                <div>
                  <p class="font-medium text-gray-900 dark:text-white">
                    {{ item.app_name || item.app_id }}
                  </p>
                  <p class="text-sm text-gray-500">
                    Version: {{ item.version || 'Unknown' }} | Account: {{ item.account_email }}
                  </p>
                </div>
                <div class="text-right">
                  <el-tag
                    :type="item.status === 'completed' ? 'success' : item.status === 'failed' ? 'danger' : 'warning'"
                    size="small"
                  >
                    {{ item.status === 'completed' ? 'Completed' : item.status === 'failed' ? 'Failed' : 'In Progress' }}
                  </el-tag>
                  <p
                    v-if="item.progress > 0"
                    class="text-sm text-gray-500 mt-1"
                  >
                    {{ item.progress }}%
                  </p>
                </div>
              </div>
              <p
                v-if="item.error"
                class="text-sm text-red-500 mt-2"
              >
                {{ item.error }}
              </p>
            </div>
          </el-space>
        </div>
      </div>

      <template #footer>
        <el-button @click="showDetailsDialog = false">
          Close
        </el-button>
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

// Load batch tasks
const loadTasks = async () => {
  try {
    const response = await fetch(`${API_BASE}/batch-tasks`, { credentials: 'include' })
    const data = await response.json()
    if (data.ok) {
      tasks.value = data.data || []
    }
  } catch (error) {
    console.error('Failed to load batch tasks:', error)
    ElMessage.error('Failed to load batch tasks')
  }
}

// Create batch task
const removeDraftItem = (index) => {
  appStore.removeBatchDraftItem(index)
}

const clearDraftItems = () => {
  appStore.clearBatchDraftItems()
}

const createBatchTask = async () => {
  if (!createForm.value.taskName) {
    ElMessage.warning('Please enter a task name')
    return
  }

  if (draftItems.value.length === 0) {
    ElMessage.warning('Please add at least one item from the Download page first')
    return
  }

  creating.value = true
  try {
    const response = await fetch(`${API_BASE}/batch-download`, {
      credentials: 'include',
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
      ElMessage.success('Batch task created successfully')
      showCreateDialog.value = false
      createForm.value.taskName = ''
      appStore.clearBatchDraftItems()
      await loadTasks()
    } else {
      ElMessage.error(data.error || 'Failed to create batch task')
    }
  } catch (error) {
    console.error('Failed to create batch task:', error)
    ElMessage.error('Failed to create batch task')
  } finally {
    creating.value = false
  }
}

// View task details
const viewDetails = async (task) => {
  currentTask.value = task
  try {
    const response = await fetch(`${API_BASE}/batch-tasks/${task.id}`, { credentials: 'include' })
    const data = await response.json()
    if (data.ok && data.data.items) {
      taskItems.value = data.data.items
    }
    showDetailsDialog.value = true
  } catch (error) {
    console.error('Failed to load task details:', error)
    ElMessage.error('Failed to load task details')
  }
}

// Delete task
const deleteTask = async (id) => {
  try {
    await ElMessageBox.confirm('Are you sure you want to delete this batch task?', 'Confirm Delete', {
      type: 'warning'
    })

    const response = await fetch(`${API_BASE}/batch-tasks/${id}`, {
      credentials: 'include',
      method: 'DELETE'
    })
    const data = await response.json()

    if (data.ok) {
      ElMessage.success('Deleted successfully')
      await loadTasks()
    } else {
      ElMessage.error(data.error || 'Delete failed')
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete task:', error)
      ElMessage.error('Delete failed')
    }
  }
}

// Calculate progress
const calculateProgress = (task) => {
  if (task.total_count === 0) return 0
  return Math.round((task.completed_count / task.total_count) * 100)
}

// Format date
const formatDate = (dateString) => {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toLocaleString()
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
