<template>
  <div class="space-y-6">
    <!-- Account Management (Apple accounts) -->
    <AccountManager @accounts-updated="(v) => emit('accounts-updated', v)" />

    <!-- Admin Account Security -->
    <div class="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
      <div class="flex items-center space-x-3 mb-6">
        <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-500 rounded-lg flex items-center justify-center shadow">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
            账号安全
          </h3>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            修改管理员登录凭据
          </p>
        </div>
      </div>

      <!-- Current info -->
      <div class="mb-5 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-3">
            <div class="w-9 h-9 bg-gradient-to-br from-blue-400 to-indigo-500 rounded-full flex items-center justify-center text-white text-sm font-bold">
              {{ (appStore.authState.user?.username || '?')[0].toUpperCase() }}
            </div>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-white">
                {{ appStore.authState.user?.username || '未知' }}
              </p>
              <p class="text-xs text-gray-500 dark:text-gray-400">
                管理员账号
              </p>
            </div>
          </div>
          <el-button
            size="small"
            @click="showChangeDialog = true"
          >
            修改账号
          </el-button>
        </div>
      </div>

      <el-dialog
        v-model="showChangeDialog"
        title="修改登录凭据"
        width="420px"
        :close-on-click-modal="false"
        align-center
      >
        <el-form
          ref="credFormRef"
          :model="credForm"
          :rules="credRules"
          label-position="top"
        >
          <el-form-item
            label="当前密码"
            prop="current_password"
          >
            <el-input
              v-model="credForm.current_password"
              type="password"
              show-password
              autocomplete="current-password"
              placeholder="请输入当前密码"
            />
          </el-form-item>
          <el-form-item
            label="新用户名（留空则不修改）"
            prop="new_username"
          >
            <el-input
              v-model="credForm.new_username"
              autocomplete="off"
              placeholder="输入新用户名或留空"
            />
          </el-form-item>
          <el-form-item
            label="新密码"
            prop="new_password"
          >
            <el-input
              v-model="credForm.new_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="请输入新密码"
            />
          </el-form-item>
          <el-form-item
            label="确认新密码"
            prop="confirm_password"
          >
            <el-input
              v-model="credForm.confirm_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="请再次输入新密码"
            />
          </el-form-item>
        </el-form>

        <template #footer>
          <el-button @click="showChangeDialog = false">
            取消
          </el-button>
          <el-button
            type="primary"
            :loading="credLoading"
            @click="handleChangeCredentials"
          >
            确认修改
          </el-button>
        </template>
      </el-dialog>
    </div>

    <!-- Notification Settings -->
    <div class="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
      <div class="flex items-center space-x-3 mb-6">
        <div class="w-10 h-10 bg-gradient-to-br from-amber-500 to-orange-500 rounded-lg flex items-center justify-center shadow">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
        </div>
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
            通知管理
          </h3>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            自定义浏览器通知行为
          </p>
        </div>
      </div>

      <!-- Permission -->
      <div
        v-if="notifications.permission.value !== 'granted'"
        class="mb-5 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-xl"
      >
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-yellow-800 dark:text-yellow-300">
              浏览器通知未授权
            </p>
            <p class="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
              需要授权后才能接收通知
            </p>
          </div>
          <el-button
            type="warning"
            size="small"
            @click="handleRequestPermission"
          >
            开启权限
          </el-button>
        </div>
      </div>
      <div
        v-else
        class="mb-5 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl"
      >
        <p class="text-sm text-green-700 dark:text-green-300">
          ✅ 浏览器通知已授权
        </p>
      </div>

      <!-- Toggles -->
      <div class="space-y-3">
        <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              新版本检测
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              订阅应用有更新时通知
            </p>
          </div>
          <el-switch
            :model-value="notifications.settings.value.versionUpdate"
            @change="(v) => toggleNotification('versionUpdate', v)"
          />
        </div>
        <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              下载完成
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              IPA 下载成功时通知
            </p>
          </div>
          <el-switch
            :model-value="notifications.settings.value.downloadComplete"
            @change="(v) => toggleNotification('downloadComplete', v)"
          />
        </div>
        <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
          <div>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              下载失败
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              IPA 下载出错时通知
            </p>
          </div>
          <el-switch
            :model-value="notifications.settings.value.downloadFailed"
            @change="(v) => toggleNotification('downloadFailed', v)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '../stores/app'
import { useNotifications } from '../composables/useNotifications'
import AccountManager from './AccountManager.vue'

const emit = defineEmits(['accounts-updated', 'logout'])
const appStore = useAppStore()
const notifications = useNotifications()

// ---- Notification helpers ----
async function handleRequestPermission() {
  const result = await notifications.requestPermission()
  if (result === 'granted') {
    ElMessage.success('通知权限已开启')
  } else {
    ElMessage.warning('通知权限被拒绝，可在浏览器设置中手动开启')
  }
}

function toggleNotification(type, value) {
  notifications.toggle(type, value)
  if (type === 'versionUpdate') {
    value ? notifications.startVersionPolling() : notifications.stopVersionPolling()
  }
}

// ---- Credential change ----
const showChangeDialog = ref(false)
const credFormRef = ref(null)
const credLoading = ref(false)

const credForm = reactive({
  current_password: '',
  new_username: '',
  new_password: '',
  confirm_password: ''
})

const credRules = {
  current_password: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  new_password: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
  confirm_password: [
    { required: true, message: '请确认新密码', trigger: 'blur' },
    {
      validator: (_, value, callback) => {
        if (value !== credForm.new_password) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

async function handleChangeCredentials() {
  if (!credFormRef.value) return
  try {
    await credFormRef.value.validate()
    credLoading.value = true

    const body = {
      current_password: credForm.current_password,
      new_password: credForm.new_password
    }
    const trimmed = credForm.new_username.trim()
    if (trimmed) body.new_username = trimmed

    const res = await fetch('/api/auth/change-password', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(body)
    })

    if (!res.ok) {
      let msg = '修改失败'
      try { const j = await res.json(); msg = j?.error || msg } catch {}
      throw new Error(msg)
    }

    const json = await res.json()
    appStore.setAuthUser(json?.data || null)

    // Reset form & close
    showChangeDialog.value = false
    credForm.current_password = ''
    credForm.new_username = ''
    credForm.new_password = ''
    credForm.confirm_password = ''

    ElMessage.success('登录凭据已修改，请重新登录')
    emit('logout')
  } catch (e) {
    ElMessage.error(e?.message || '修改失败')
  } finally {
    credLoading.value = false
  }
}
</script>
