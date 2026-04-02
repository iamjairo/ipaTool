<template>
  <div class="w-full flex justify-center">
    <div class="w-full max-w-md mt-10">
      <div class="glass-card p-6 border border-gray-200/50 dark:border-gray-700/50">
        <div class="mb-6">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            Admin Login
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Default credentials: admin / admin
          </p>
        </div>

        <el-form
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginRules"
          label-position="top"
        >
          <el-form-item
            label="Username"
            prop="username"
          >
            <el-input
              v-model="loginForm.username"
              autocomplete="username"
              placeholder="Please enter username"
              size="large"
              @keyup.enter="handleLogin"
            />
          </el-form-item>

          <el-form-item
            label="Password"
            prop="password"
          >
            <el-input
              v-model="loginForm.password"
              type="password"
              show-password
              autocomplete="current-password"
              placeholder="Please enter password"
              size="large"
              @keyup.enter="handleLogin"
            />
          </el-form-item>

          <el-button
            type="primary"
            size="large"
            class="w-full"
            :loading="loginLoading"
            @click="handleLogin"
          >
            Sign In
          </el-button>
        </el-form>

        <div
          v-if="appStore.authState.user?.is_default"
          class="mt-4"
        >
          <el-alert
            type="warning"
            show-icon
            :closable="false"
            title="Default password detected, you must change your password before continuing"
          />
        </div>
      </div>

      <el-dialog
        v-model="showChangePassword"
        title="First Login: Please update your username and password"
        width="420px"
        :close-on-click-modal="false"
        :close-on-press-escape="false"
        :show-close="false"
        align-center
      >
        <el-form
          ref="pwdFormRef"
          :model="pwdForm"
          :rules="pwdRules"
          label-position="top"
        >
          <el-form-item
            label="New Username"
            prop="new_username"
          >
            <el-input
              v-model="pwdForm.new_username"
              autocomplete="off"
              placeholder="Please enter new username"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>
          <el-form-item
            label="Current Password"
            prop="current_password"
          >
            <el-input
              v-model="pwdForm.current_password"
              type="password"
              show-password
              autocomplete="current-password"
              placeholder="Please enter current password"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>

          <el-form-item
            label="New Password"
            prop="new_password"
          >
            <el-input
              v-model="pwdForm.new_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="Please enter new password"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>

          <el-form-item
            label="Confirm New Password"
            prop="confirm_password"
          >
            <el-input
              v-model="pwdForm.confirm_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="Please confirm new password"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>
        </el-form>

        <template #footer>
          <el-button
            type="primary"
            :loading="pwdLoading"
            @click="handleChangePassword"
          >
            Change Password
          </el-button>
        </template>
      </el-dialog>
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'

const emit = defineEmits(['login-success'])
const appStore = useAppStore()

const loginFormRef = ref(null)
const loginLoading = ref(false)

const loginForm = reactive({
  username: 'admin',
  password: ''
})

const loginRules = {
  username: [{ required: true, message: 'Please enter username', trigger: 'blur' }],
  password: [{ required: true, message: 'Please enter password', trigger: 'blur' }]
}

const showChangePassword = ref(false)
const pwdFormRef = ref(null)
const pwdLoading = ref(false)

const pwdForm = reactive({
  new_username: '',
  current_password: '',
  new_password: '',
  confirm_password: ''
})

const pwdRules = {
  current_password: [{ required: true, message: 'Please enter current password', trigger: 'blur' }],
  new_password: [{ required: true, message: 'Please enter new password', trigger: 'blur' }],
  confirm_password: [
    { required: true, message: 'Please confirm new password', trigger: 'blur' },
    {
      validator: (_, value, callback) => {
        if (value !== pwdForm.new_password) {
          callback(new Error('New passwords do not match'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

watch(
  () => appStore.authState.user?.is_default,
  (isDefault) => {
    if (isDefault) {
      showChangePassword.value = true
    }
  },
  { immediate: true }
)

const handleLogin = async () => {
  if (!loginFormRef.value) return

  try {
    await loginFormRef.value.validate()

    loginLoading.value = true
    const user = await appStore.loginAdmin(loginForm.username, loginForm.password)

    if (user?.is_default) {
      showChangePassword.value = true
      pwdForm.current_password = loginForm.password
      pwdForm.new_username = ''
    } else {
      ElMessage.success('Login successful')
      emit('login-success')
    }
  } catch (e) {
    ElMessage.error(e?.message || 'Login failed')
  } finally {
    loginLoading.value = false
  }
}

const handleChangePassword = async () => {
  if (!pwdFormRef.value) return

  try {
    await pwdFormRef.value.validate()

    pwdLoading.value = true

    const res = await fetch('/api/auth/change-password', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify({
        current_password: pwdForm.current_password,
        new_password: pwdForm.new_password,
        new_username: pwdForm.new_username.trim() || undefined
      })
    })

    if (!res.ok) {
      let msg = 'Failed to change password'
      try {
        const json = await res.json()
        msg = json?.error || msg
      } catch {}
      throw new Error(msg)
    }

    const json = await res.json()

    ElMessage.success('Password changed successfully, please log in with your new password')
    showChangePassword.value = false

    // Clear form
    pwdForm.new_username = ''
    pwdForm.current_password = ''
    pwdForm.new_password = ''
    pwdForm.confirm_password = ''

    // Properly logout: clear server session + cookie + local state
    await appStore.logoutAdmin()

    // Reload the page to force App.vue to re-check auth state.
    // Without this, the local authState in App.vue stays stale and
    // the user sees a blank/broken state instead of the login form.
    window.location.reload()
  } catch (e) {
    ElMessage.error(e?.message || 'Failed to change password')
  } finally {
    pwdLoading.value = false
  }
}
</script>
