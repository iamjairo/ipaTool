import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  // Download task state
  const downloadState = ref({
    selectedApp: null,
    appId: '',
    appVersionId: '',
    selectedAccountIndex: '',
    availableVersions: [],
    selectedVersionId: '',
    versionsLoaded: false,
    showProgressPanel: false,
    progressPercentage: 0,
    progressMessage: '',
    progressLogs: ''
  })

  // Download task queue
  const taskQueue = ref([])

  // Batch download draft items
  const batchDraftItems = ref([])

  // Currently active tab
  const activeTab = ref('download')

  // Account update counter
  const accountsUpdateCounter = ref(0)

  // Admin login state
  const authState = ref({
    checked: false,
    loading: false,
    user: null
  })

  const setAuthUser = (user) => {
    authState.value.user = user
  }

  const checkAuth = async () => {
    authState.value.loading = true
    try {
      const res = await fetch('/api/auth/me', {
        method: 'GET',
        credentials: 'include'
      })

      if (!res.ok) {
        authState.value.user = null
        return false
      }

      const json = await res.json()
      authState.value.user = json?.data || null
      return !!authState.value.user
    } catch {
      authState.value.user = null
      return false
    } finally {
      authState.value.checked = true
      authState.value.loading = false
    }
  }

  const loginAdmin = async (username, password) => {
    const res = await fetch('/api/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify({ username, password })
    })

    if (!res.ok) {
      let msg = 'Login failed'
      try {
        const json = await res.json()
        msg = json?.error || msg
      } catch {}
      throw new Error(msg)
    }

    const json = await res.json()
    authState.value.user = json?.data || null
    authState.value.checked = true
    return authState.value.user
  }

  const logoutAdmin = async () => {
    try {
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include'
      })
    } catch {}
    authState.value.user = null
    authState.value.checked = true
  }

  // Set selected app
  const setSelectedApp = (app) => {
    downloadState.value.selectedApp = app
    if (app && app.trackId) {
      downloadState.value.appId = String(app.trackId)
    }
  }

  // Update download state
  const updateDownloadState = (key, value) => {
    if (key in downloadState.value) {
      downloadState.value[key] = value
    }
  }

  // Add task to queue
  const addToQueue = (item) => {
    const existingIndex = taskQueue.value.findIndex(q => q.id === item.id)
    if (existingIndex >= 0) {
      // Update existing task
      taskQueue.value[existingIndex] = { ...taskQueue.value[existingIndex], ...item }
    } else {
      // Add new task
      taskQueue.value.push(item)
    }
  }

  // Update queue task
  const updateQueueItem = (id, updates) => {
    const index = taskQueue.value.findIndex(q => q.id === id)
    if (index >= 0) {
      taskQueue.value[index] = { ...taskQueue.value[index], ...updates }
    }
  }

  // Remove task from queue
  const removeFromQueue = (idOrIndex) => {
    if (typeof idOrIndex === 'number' && idOrIndex >= 0 && idOrIndex < taskQueue.value.length) {
      taskQueue.value.splice(idOrIndex, 1)
      return
    }

    const index = taskQueue.value.findIndex(item => item.id === idOrIndex)
    if (index >= 0) {
      taskQueue.value.splice(index, 1)
    }
  }

  // Clear task queue
  const clearQueue = () => {
    taskQueue.value = []
  }

  // Add batch download draft item
  const addBatchDraftItem = (item) => {
    const key = `${item.app_id}:${item.version || 'latest'}:${item.account_email}`
    const existingIndex = batchDraftItems.value.findIndex(
      draft => `${draft.app_id}:${draft.version || 'latest'}:${draft.account_email}` === key
    )

    if (existingIndex >= 0) {
      batchDraftItems.value[existingIndex] = { ...batchDraftItems.value[existingIndex], ...item }
      return { added: false, updated: true }
    }

    batchDraftItems.value.push(item)
    return { added: true, updated: false }
  }

  const removeBatchDraftItem = (index) => {
    batchDraftItems.value.splice(index, 1)
  }

  const clearBatchDraftItems = () => {
    batchDraftItems.value = []
  }

  // Trigger account update
  const triggerAccountsUpdate = () => {
    accountsUpdateCounter.value++
  }

  return {
    downloadState,
    taskQueue,
    batchDraftItems,
    activeTab,
    accountsUpdateCounter,
    authState,
    setSelectedApp,
    updateDownloadState,
    addToQueue,
    updateQueueItem,
    removeFromQueue,
    clearQueue,
    addBatchDraftItem,
    removeBatchDraftItem,
    clearBatchDraftItems,
    triggerAccountsUpdate,
    setAuthUser,
    checkAuth,
    loginAdmin,
    logoutAdmin
  }
})
