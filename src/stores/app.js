import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 下载任务状态
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

  // 下载任务队列
  const taskQueue = ref([])

  // 批量下载草稿项
  const batchDraftItems = ref([])

  // 当前激活的页面标签
  const activeTab = ref('download')

  // 账号更新计数器
  const accountsUpdateCounter = ref(0)

  // 管理员登录态
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
      let msg = '登录失败'
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

  // 设置选中的应用
  const setSelectedApp = (app) => {
    downloadState.value.selectedApp = app
    if (app && app.trackId) {
      downloadState.value.appId = String(app.trackId)
    }
  }

  // 更新下载状态
  const updateDownloadState = (key, value) => {
    if (key in downloadState.value) {
      downloadState.value[key] = value
    }
  }

  // 添加任务到队列
  const addToQueue = (item) => {
    const existingIndex = taskQueue.value.findIndex(q => q.id === item.id)
    if (existingIndex >= 0) {
      // 更新现有任务
      taskQueue.value[existingIndex] = { ...taskQueue.value[existingIndex], ...item }
    } else {
      // 添加新任务
      taskQueue.value.push(item)
    }
  }

  // 更新队列任务
  const updateQueueItem = (id, updates) => {
    const index = taskQueue.value.findIndex(q => q.id === id)
    if (index >= 0) {
      taskQueue.value[index] = { ...taskQueue.value[index], ...updates }
    }
  }

  // 从队列移除任务
  const removeFromQueue = (index) => {
    taskQueue.value.splice(index, 1)
  }

  // 清空任务队列
  const clearQueue = () => {
    taskQueue.value = []
  }

  // 添加批量下载草稿项
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

  // 触发账号更新
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
