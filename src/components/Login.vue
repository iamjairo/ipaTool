<template>
  <div class="w-full flex justify-center">
    <div class="w-full max-w-md mt-10">
      <div class="glass-card p-6 border border-gray-200/50 dark:border-gray-700/50">
        <div class="mb-6">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            管理员登录
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            默认账号：admin / admin
          </p>
        </div>

        <el-form
          ref="loginFormRef"
          :model="loginForm"
          :rules="loginRules"
          label-position="top"
        >
          <el-form-item
            label="用户名"
            prop="username"
          >
            <el-input
              v-model="loginForm.username"
              autocomplete="username"
              placeholder="请输入用户名"
              size="large"
              @keyup.enter="handleLogin"
            />
          </el-form-item>

          <el-form-item
            label="密码"
            prop="password"
          >
            <el-input
              v-model="loginForm.password"
              type="password"
              show-password
              autocomplete="current-password"
              placeholder="请输入密码"
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
            登录
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
            title="检测到仍在使用默认密码，必须先修改密码后才能进入系统"
          />
        </div>
      </div>

      <el-dialog
        v-model="showChangePassword"
        title="首次登录：请修改密码"
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
            label="当前密码"
            prop="current_password"
          >
            <el-input
              v-model="pwdForm.current_password"
              type="password"
              show-password
              autocomplete="current-password"
              placeholder="请输入当前密码"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>

          <el-form-item
            label="新密码"
            prop="new_password"
          >
            <el-input
              v-model="pwdForm.new_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="请输入新密码"
              @keyup.enter="handleChangePassword"
            />
          </el-form-item>

          <el-form-item
            label="确认新密码"
            prop="confirm_password"
          >
            <el-input
              v-model="pwdForm.confirm_password"
              type="password"
              show-password
              autocomplete="new-password"
              placeholder="请再次输入新密码"
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
            修改密码
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

const appStore = useAppStore()

const loginFormRef = ref(null)
const loginLoading = ref(false)

const loginForm = reactive({
  username: 'admin',
  password: ''
})

const loginRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }]
}

const showChangePassword = ref(false)
const pwdFormRef = ref(null)
const pwdLoading = ref(false)

const pwdForm = reactive({
  current_password: '',
  new_password: '',
  confirm_password: ''
})

const pwdRules = {
  current_password: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  new_password: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
  confirm_password: [
    { required: true, message: '请确认新密码', trigger: 'blur' },
    {
      validator: (_, value, callback) => {
        if (value !== pwdForm.new_password) {
          callback(new Error('两次输入的新密码不一致'))
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

    ElMessage.success('登录成功')

    if (user?.is_default) {
      showChangePassword.value = true
      pwdForm.current_password = loginForm.password
    }
  } catch (e) {
    ElMessage.error(e?.message || '登录失败')
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
        new_password: pwdForm.new_password
      })
    })

    if (!res.ok) {
      let msg = '修改密码失败'
      try {
        const json = await res.json()
        msg = json?.error || msg
      } catch (_) {}
      throw new Error(msg)
    }

    const json = await res.json()
    appStore.setAuthUser(json?.data || null)

    ElMessage.success('密码修改成功')
    showChangePassword.value = false

    pwdForm.current_password = ''
    pwdForm.new_password = ''
    pwdForm.confirm_password = ''
  } catch (e) {
    ElMessage.error(e?.message || '修改密码失败')
  } finally {
    pwdLoading.value = false
  }
}
</script>
