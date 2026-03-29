<template>
  <div
    id="app"
    :class="{ 'dark': isDark }"
    class="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 transition-colors duration-300"
  >
    <!-- Auth loading -->
    <div
      v-if="authState === 'loading'"
      class="min-h-screen flex items-center justify-center"
    >
      <div class="text-gray-400">
        加载中...
      </div>
    </div>

    <!-- Login page -->
    <Login
      v-else-if="authState === 'unauthenticated'"
      @login-success="onLoginSuccess"
    />

    <!-- Main app (authenticated) -->
    <template v-else>
      <header class="sticky top-0 z-50 glass-card border-b border-gray-200/50 dark:border-gray-700/50">
        <div class="container mx-auto px-4 py-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <div class="w-10 h-10 bg-gradient-to-br from-primary-500 to-purple-600 rounded-xl flex items-center justify-center shadow-lg">
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
                    d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                  />
                </svg>
              </div>
              <div>
                <h1 class="text-2xl font-bold gradient-text">
                  IPA Tool
                </h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">
                  IPA管理工具
                </p>
              </div>
            </div>
          
            <div class="flex items-center space-x-2">
              <button 
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors" 
                :title="isDark ? '切换到浅色模式' : '切换到深色模式'"
                @click="toggleDark"
              >
                <svg
                  v-if="isDark"
                  class="w-5 h-5 text-yellow-400"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"
                    clip-rule="evenodd"
                  />
                </svg>
                <svg
                  v-else
                  class="w-5 h-5 text-gray-700"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
                </svg>
              </button>
              <button
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                title="设置"
                @click="showSettings = true"
              >
                <svg
                  class="w-5 h-5 text-gray-600 dark:text-gray-300"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                </svg>
              </button>
              <button
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                title="退出登录"
                @click="handleLogout"
              >
                <svg
                  class="w-5 h-5 text-gray-600 dark:text-gray-300"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                  />
                </svg>
              </button>
              <a 
                href="https://github.com/ruanrrn/ipaTool" 
                target="_blank"
                class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                title="查看源代码"
              >
                <svg
                  class="w-5 h-5"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                </svg>
              </a>
            </div>
          </div>
        </div>
      </header>

      <!-- Settings drawer -->
      <el-drawer
        v-model="showSettings"
        title="设置"
        direction="rtl"
        size="400px"
        :append-to-body="true"
      >
        <div class="px-2 space-y-8">
          <!-- Account security section -->
          <section>
            <h4 class="text-sm font-semibold text-gray-900 dark:text-white mb-4 pb-2 border-b border-gray-200 dark:border-gray-700">
              🔒 账号安全
            </h4>
            <el-form label-position="top" class="space-y-0">
              <el-form-item label="新用户名（留空则不修改）">
                <el-input
                  v-model="settingsForm.new_username"
                  autocomplete="off"
                  placeholder="输入新用户名"
                />
              </el-form-item>
              <el-form-item label="当前密码">
                <el-input
                  v-model="settingsForm.current_password"
                  type="password"
                  show-password
                  autocomplete="current-password"
                  placeholder="输入当前密码以确认修改"
                />
              </el-form-item>
              <el-form-item label="新密码（留空则不修改）">
                <el-input
                  v-model="settingsForm.new_password"
                  type="password"
                  show-password
                  autocomplete="new-password"
                  placeholder="输入新密码"
                />
              </el-form-item>
              <el-button
                type="primary"
                :loading="settingsLoading"
                @click="handleSaveSettings"
              >
                保存修改
              </el-button>
            </el-form>
          </section>

          <!-- Notification settings section -->
          <section>
            <h4 class="text-sm font-semibold text-gray-900 dark:text-white mb-4 pb-2 border-b border-gray-200 dark:border-gray-700">
              🔔 通知设置
            </h4>

            <!-- Permission -->
            <div
              v-if="notifications.permission.value !== 'granted'"
              class="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-xl"
            >
              <p class="text-sm text-yellow-800 dark:text-yellow-300 mb-2">
                浏览器通知未授权，请先开启权限。
              </p>
              <el-button
                type="warning"
                size="small"
                @click="handleRequestNotificationPermission"
              >
                开启通知权限
              </el-button>
            </div>
            <div
              v-else
              class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl"
            >
              <p class="text-sm text-green-700 dark:text-green-300">
                ✅ 浏览器通知已授权
              </p>
            </div>

            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-700 dark:text-gray-300">新版本检测</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">订阅应用有更新时通知</p>
                </div>
                <el-switch
                  :model-value="notifications.settings.value.versionUpdate"
                  @change="(v) => handleToggleNotification('versionUpdate', v)"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-700 dark:text-gray-300">下载完成</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">IPA 下载成功时通知</p>
                </div>
                <el-switch
                  :model-value="notifications.settings.value.downloadComplete"
                  @change="(v) => handleToggleNotification('downloadComplete', v)"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-700 dark:text-gray-300">下载失败</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">IPA 下载出错时通知</p>
                </div>
                <el-switch
                  :model-value="notifications.settings.value.downloadFailed"
                  @change="(v) => handleToggleNotification('downloadFailed', v)"
                />
              </div>
            </div>
          </section>
        </div>
      </el-drawer>

      <main class="container mx-auto px-4 py-8">
        <TabLayout 
          @app-selected="handleAppSelected"
          @download-started="handleDownloadStarted"
          @accounts-updated="handleAccountsUpdated"
          @remove-item="handleRemoveItem"
          @clear-queue="handleClearQueue"
        />
      </main>

      <footer class="mt-16 py-8 border-t border-gray-200 dark:border-gray-700">
        <div class="container mx-auto px-4 text-center pb-safe-mobile">
          <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-xl">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-2">
              💡 使用提示
            </h3>
            <div class="text-xs text-gray-600 dark:text-gray-400 space-y-1">
              <p><strong class="text-blue-600 dark:text-blue-400">直链下载</strong>：使用浏览器原生下载器，适合小文件快速下载</p>
              <p><strong class="text-blue-600 dark:text-blue-400">下载并安装</strong>：后端处理大文件，显示实时进度和日志，支持自动签名</p>
            </div>
          </div>
          <p class="text-gray-600 dark:text-gray-400 mb-3">
            Made with ❤️ by <a
              href="https://github.com/ruanrrn"
              class="text-primary-600 hover:underline font-medium"
              target="_blank"
            >ruanrrn</a>
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-500 mb-4">
            © {{ new Date().getFullYear() }} IPA Tool. 现代化 IPA 下载与签名工具
          </p>
          <div class="flex justify-center space-x-6 text-sm">
            <a
              href="https://github.com/ruanrrn/ipaTool"
              class="text-primary-600 hover:text-primary-700 dark:hover:text-primary-500 transition-colors flex items-center space-x-1"
              target="_blank"
            >
              <svg
                class="w-4 h-4"
                fill="currentColor"
                viewBox="0 0 24 24"
              ><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" /></svg>
              <span>GitHub</span>
            </a>
            <a
              href="https://github.com/ruanrrn/ipaTool/issues"
              class="text-primary-600 hover:text-primary-700 dark:hover:text-primary-500 transition-colors"
              target="_blank"
            >
              反馈问题
            </a>
            <a
              href="https://github.com/ruanrrn/ipaTool"
              class="text-primary-600 hover:text-primary-700 dark:hover:text-primary-500 transition-colors"
              target="_blank"
            >
              Star 支持
            </a>
          </div>
        </div>
      </footer>
    </template>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, watch, ref } from 'vue'
import { useDark } from './composables/useDark'
import { useAppStore } from './stores/app'
import { useNotifications } from './composables/useNotifications'
import TabLayout from './components/TabLayout.vue'
import Login from './components/Login.vue'
import { ElMessage } from 'element-plus'

const { isDark, toggleDark } = useDark()
const appStore = useAppStore()
const notifications = useNotifications()

// Auth state: 'loading' | 'unauthenticated' | 'authenticated'
const authState = ref('loading')

const API_BASE = '/api'

// Settings dialog
const showSettings = ref(false)
const settingsLoading = ref(false)
const settingsForm = ref({
  new_username: '',
  current_password: '',
  new_password: ''
})

async function handleSaveSettings() {
  const { new_username, current_password, new_password } = settingsForm.value
  if (!current_password) {
    ElMessage.warning('请输入当前密码')
    return
  }
  if (!new_username.trim() && !new_password) {
    ElMessage.warning('请至少填写一项要修改的内容')
    return
  }

  settingsLoading.value = true
  try {
    const res = await fetch(`${API_BASE}/auth/change-password`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({
        current_password,
        new_password: new_password || current_password,
        new_username: new_username.trim() || undefined
      })
    })
    const json = await res.json()
    if (!res.ok || !json.ok) {
      throw new Error(json?.error || '修改失败')
    }
    ElMessage.success('修改成功，请重新登录')
    showSettings.value = false
    settingsForm.value = { new_username: '', current_password: '', new_password: '' }
    // force re-login
    authState.value = 'unauthenticated'
    appStore.authState.user = null
  } catch (e) {
    ElMessage.error(e?.message || '修改失败')
  } finally {
    settingsLoading.value = false
  }
}

async function checkAuth() {
  try {
    const res = await fetch(`${API_BASE}/auth/me`, { credentials: 'same-origin' })
    const data = await res.json()
    if (data.ok && data.data) {
      authState.value = 'authenticated'
    } else {
      authState.value = 'unauthenticated'
    }
  } catch {
    authState.value = 'unauthenticated'
  }
}

function onLoginSuccess() {
  authState.value = 'authenticated'
}

async function handleRequestNotificationPermission() {
  const result = await notifications.requestPermission()
  if (result === 'granted') {
    ElMessage.success('通知权限已开启')
  } else {
    ElMessage.warning('通知权限被拒绝，可在浏览器设置中手动开启')
  }
}

function handleToggleNotification(type, value) {
  notifications.toggle(type, value)
  // Start/stop version polling accordingly
  if (type === 'versionUpdate') {
    if (value) {
      notifications.startVersionPolling()
    } else {
      notifications.stopVersionPolling()
    }
  }
}

async function handleLogout() {
  try {
    await fetch(`${API_BASE}/auth/logout`, { method: 'POST', credentials: 'same-origin' })
  } catch { /* ignore */ }
  authState.value = 'unauthenticated'
  ElMessage.success('已退出登录')
}

const handleAppSelected = (app) => {
  appStore.setSelectedApp(app)
}

const handleDownloadStarted = (task) => {
  appStore.addToQueue(task)
  appStore.activeTab = 'queue'
}

const handleRemoveItem = (index) => {
  appStore.removeFromQueue(index)
}

const handleClearQueue = () => {
  appStore.clearQueue()
}

const handleAccountsUpdated = () => {
  appStore.triggerAccountsUpdate()
}

const updateDarkClass = () => {
  const html = document.documentElement
  const body = document.body
  if (isDark.value) {
    html.classList.add('dark')
    body.classList.add('dark')
  } else {
    html.classList.remove('dark')
    body.classList.remove('dark')
  }
}

onMounted(() => {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  isDark.value = prefersDark
  updateDarkClass()
  checkAuth()
  notifications.init()
})

onUnmounted(() => {
  notifications.stopVersionPolling()
})

watch(isDark, () => {
  updateDarkClass()
})
</script>
