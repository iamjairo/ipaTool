<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center space-x-3">
        <div class="w-12 h-12 bg-gradient-to-br from-green-500 to-teal-500 rounded-xl flex items-center justify-center shadow-lg flex-shrink-0">
          <svg
            class="w-6 h-6 text-white"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
        </div>
        <div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-white">
            App Subscriptions & Updates
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {{ subscriptions.length }}  subscriptions | {{ updateCount }}  updates
          </p>
        </div>
      </div>
      <div class="flex items-center space-x-2 flex-shrink-0">
        <el-button
          type="primary"
          :icon="Refresh"
          :loading="checking"
          @click="checkUpdates"
        >
          Check for Updates
        </el-button>
        <el-button
          :icon="Plus"
          @click="showSubscribeDialog = true"
        >
          Add Subscription
        </el-button>
      </div>
    </div>

    <!-- Update Notifications -->
    <div
      v-if="updates.length > 0"
      class="mb-6"
    >
      <div class="flex items-center gap-2 mb-3">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          Updates Found
        </h3>
        <el-badge
          :value="updates.length"
          type="danger"
        />
      </div>
      <el-space
        direction="vertical"
        :size="12"
        fill
      >
        <el-alert
          v-for="update in updates"
          :key="update.app_id"
          type="success"
          :closable="false"
          show-icon
          class="update-alert"
        >
          <template #title>
            <div class="flex items-center gap-3">
              <el-image
                :src="update.artwork_url || 'https://via.placeholder.com/40'"
                class="w-10 h-10 rounded-lg"
                fit="cover"
              />
              <div class="flex-1">
                <p class="font-medium text-gray-900 dark:text-white">
                  {{ update.app_name }}
                </p>
                <p class="text-sm text-gray-500">
                  {{ update.current_version }} → {{ update.latest_version }}
                </p>
              </div>
              <el-button
                type="primary"
                size="small"
                @click="downloadUpdate(update)"
              >
                Download New Version
              </el-button>
            </div>
          </template>
        </el-alert>
      </el-space>
    </div>

    <!-- Subscription List -->
    <div v-if="subscriptions.length > 0">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          My Subscriptions
        </h3>
      </div>
      <el-space
        direction="vertical"
        :size="12"
        fill
      >
        <el-card
          v-for="sub in subscriptions"
          :key="sub.id"
          shadow="hover"
          class="sub-card"
        >
          <div class="flex items-start gap-4">
            <el-image
              :src="sub.artwork_url || 'https://via.placeholder.com/60'"
              class="w-12 h-12 rounded-lg shadow-md flex-shrink-0"
              fit="cover"
            />
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between gap-2">
                <h3 class="font-semibold text-gray-900 dark:text-white truncate">
                  {{ sub.app_name }}
                </h3>
                <el-button
                  type="danger"
                  size="small"
                  :icon="Delete"
                  plain
                  circle
                  @click="removeSubscription(sub)"
                />
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                {{ sub.artist_name || 'Unknown Developer' }}
              </p>
              <div class="flex items-center gap-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
                <span v-if="sub.current_version">Version: {{ sub.current_version }}</span>
                <span v-if="sub.last_checked">Checked: {{ formatDate(sub.last_checked) }}</span>
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
          d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
        />
      </svg>
      <p class="text-lg font-medium">
        No subscriptions
      </p>
      <p class="text-sm mt-2">
        Add app subscriptions to receive update notifications
      </p>
    </div>

    <!-- Add Subscription Dialog -->
    <el-dialog
      v-model="showSubscribeDialog"
      title="Add Subscription"
      width="500px"
      :close-on-click-modal="false"
    >
      <el-form
        :model="subscribeForm"
        label-width="100px"
      >
        <el-form-item label="App ID">
          <el-input
            v-model="subscribeForm.app_id"
            placeholder="Enter the app's Bundle ID or Track ID"
          />
        </el-form-item>
        <el-form-item label="App Name">
          <el-input
            v-model="subscribeForm.app_name"
            placeholder="Enter app name"
          />
        </el-form-item>
        <el-form-item label="Account Email">
          <el-input
            v-model="subscribeForm.account_email"
            placeholder="Enter the account email used for downloading"
          />
        </el-form-item>
        <el-form-item label="Region">
          <el-select
            v-model="subscribeForm.account_region"
            placeholder="Select region"
          >
            <el-option
              label="United States"
              value="US"
            />
            <el-option
              label="China"
              value="CN"
            />
            <el-option
              label="Japan"
              value="JP"
            />
            <el-option
              label="United Kingdom"
              value="GB"
            />
            <el-option
              label="Germany"
              value="DE"
            />
          </el-select>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showSubscribeDialog = false">
          Cancel
        </el-button>
        <el-button
          type="primary"
          :loading="subscribing"
          @click="addSubscription"
        >
          Add
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Refresh, Delete } from '@element-plus/icons-vue'
import { useNotifications } from '../composables/useNotifications'

const notifications = useNotifications()

const API_BASE = '/api'

const subscriptions = ref([])
const updates = ref([])
const showSubscribeDialog = ref(false)
const checking = ref(false)
const subscribing = ref(false)

const subscribeForm = ref({
  app_id: '',
  app_name: '',
  account_email: '',
  account_region: 'US',
  artwork_url: '',
  artist_name: ''
})

const updateCount = computed(() => updates.value.length)

// Load subscription list
const loadSubscriptions = async () => {
  try {
    const response = await fetch(`${API_BASE}/subscriptions`, { credentials: 'include' })
    const data = await response.json()
    if (data.ok) {
      subscriptions.value = data.data || []
    }
  } catch (error) {
    console.error('Failed to load subscriptions:', error)
    ElMessage.error('Failed to load subscriptions')
  }
}

// Check for updates
const checkUpdates = async () => {
  checking.value = true
  try {
    const response = await fetch(`${API_BASE}/check-updates`, { credentials: 'include' })
    const data = await response.json()

    if (data.ok) {
      updates.value = data.data.updates || []
      if (updates.value.length > 0) {
        ElMessage.success(`Found ${updates.value.length} update(s)`)
        // Send browser notifications one by one
        for (const update of updates.value) {
          notifications.notifyVersionUpdate(
            update.app_name,
            update.current_version,
            update.latest_version
          )
        }
      } else {
        ElMessage.info('All apps are up to date')
      }
    } else {
      ElMessage.error(data.error || 'Failed to check for updates')
    }
  } catch (error) {
    console.error('Failed to check updates:', error)
    ElMessage.error('Failed to check for updates')
  } finally {
    checking.value = false
  }
}

// Add subscription
const addSubscription = async () => {
  if (!subscribeForm.value.app_id || !subscribeForm.value.app_name || !subscribeForm.value.account_email) {
    ElMessage.warning('Please fill in all required fields')
    return
  }

  subscribing.value = true
  try {
    const response = await fetch(`${API_BASE}/subscriptions`, {
      credentials: 'include',
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(subscribeForm.value)
    })
    const data = await response.json()

    if (data.ok) {
      ElMessage.success('Subscription added successfully')
      showSubscribeDialog.value = false
      Object.assign(subscribeForm.value, {
        app_id: '',
        app_name: '',
        account_email: '',
        account_region: 'US',
        artwork_url: '',
        artist_name: ''
      })
      await loadSubscriptions()
    } else {
      ElMessage.error(data.error || 'Failed to add subscription')
    }
  } catch (error) {
    console.error('Failed to add subscription:', error)
    ElMessage.error('Failed to add subscription')
  } finally {
    subscribing.value = false
  }
}

// Remove subscription
const removeSubscription = async (sub) => {
  try {
    await ElMessageBox.confirm(`Are you sure you want to unsubscribe from "${sub.app_name}"?`, 'Confirm Unsubscribe', {
      type: 'warning'
    })

    const response = await fetch(`${API_BASE}/subscriptions?app_id=${sub.app_id}&account_email=${sub.account_email}`, {
      credentials: 'include',
      method: 'DELETE'
    })
    const data = await response.json()

    if (data.ok) {
      ElMessage.success('Unsubscribed successfully')
      await loadSubscriptions()
    } else {
      ElMessage.error(data.error || 'Failed to unsubscribe')
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to remove subscription:', error)
      ElMessage.error('Failed to unsubscribe')
    }
  }
}

// Download update
const downloadUpdate = (update) => {
  ElMessage.info(`Starting download of update for ${update.app_name}...`)
  // Download logic can be triggered here
}

// Format date
const formatDate = (dateString) => {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toLocaleString()
}

onMounted(() => {
  loadSubscriptions()
  // Auto-check for updates
  checkUpdates()
})
</script>

<style scoped>
.sub-card {
  transition: all 0.2s ease;
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.update-alert :deep(.el-alert__content) {
  padding: 0;
}
</style>
