<template>
  <div class="account-manager">
    <!-- Header -->
    <div class="account-header">
      <div class="header-icon">
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
            d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
          />
        </svg>
      </div>
      <div class="header-text">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white">
          账号管理
        </h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          管理 Apple ID 账号
        </p>
      </div>
    </div>

    <!-- Account List -->
    <div
      v-if="accounts.length > 0"
      class="accounts-section"
    >
      <div class="section-header">
        <h3 class="section-title">
          已登录账号
        </h3>
        <span class="section-count">{{ accounts.length }}</span>
      </div>
      <div class="accounts-list">
        <div
          v-for="(account, index) in accounts"
          :key="index"
          class="account-item"
        >
          <div class="account-avatar">
            <el-icon><User /></el-icon>
          </div>
          <div class="account-info">
            <p class="account-email">
              {{ account.email }}
            </p>
            <p class="account-dsid">
              DSID: {{ account.dsid }}
            </p>
            <p class="account-region">
              <span
                class="region-badge"
                :class="`region-${(account.region || 'US').toLowerCase()}`"
              >
                {{ getRegionLabel(account.region || 'US') }}
              </span>
            </p>
          </div>
          <div class="account-actions">
            <el-button
              type="primary"
              :icon="Refresh"
              circle
              size="small"
              class="refresh-button"
              title="刷新会话"
              :loading="refreshingIndex === index"
              @click="refreshAccount(index)"
            />
            <el-button
              type="danger"
              :icon="Delete"
              circle
              size="small"
              class="remove-button"
              title="删除账号"
              @click="removeAccount(index)"
            />
          </div>
        </div>
      </div>
    </div>

    <div class="account-content">
      <!-- Add Account Form -->
      <div class="form-section">
        <div class="form-header">
          <h3 class="form-title">
            登录 Apple ID
          </h3>
          <p class="form-subtitle">
            支持多账号登录
          </p>
        </div>
        <div class="form-fields">
          <div class="form-field">
            <label class="field-label">邮箱</label>
            <el-input
              v-model="newAccount.email"
              type="email"
              placeholder="your@email.com"
              :disabled="logging"
              size="large"
              clearable
              class="form-input"
            >
              <template #prefix>
                <el-icon class="field-icon">
                  <User />
                </el-icon>
              </template>
            </el-input>
          </div>

          <div class="form-field">
            <label class="field-label">密码</label>
            <el-input
              v-model="newAccount.password"
              type="password"
              placeholder="••••••••"
              :disabled="logging"
              show-password
              size="large"
              class="form-input"
            >
              <template #prefix>
                <el-icon class="field-icon">
                  <Lock />
                </el-icon>
              </template>
            </el-input>
          </div>

          <div class="form-field">
            <label class="field-label">验证码</label>
            <el-input
              v-model="newAccount.code"
              type="text"
              placeholder="两步验证码（如需要）"
              :disabled="logging"
              size="large"
              clearable
              class="form-input"
            >
              <template #prefix>
                <el-icon class="field-icon">
                  <Key />
                </el-icon>
              </template>
            </el-input>
          </div>

          <!-- 保存密码选项 -->
          <div class="form-field">
            <el-checkbox
              v-model="savePassword"
              :disabled="logging"
              class="save-password-checkbox"
            >
              <span class="checkbox-label">保存密码以便下次自动登录</span>
            </el-checkbox>
          </div>

          <el-button
            :disabled="logging || autoLogging || !isFormValid"
            :loading="logging"
            type="success"
            size="large"
            class="submit-button"
            @click="loginAccount"
          >
            <template #icon>
              <el-icon><Right /></el-icon>
            </template>
            {{ logging ? '登录中...' : '登录' }}
          </el-button>

          <!-- 自动登录状态提示 -->
          <div
            v-if="autoLogging"
            class="auto-login-status"
          >
            <el-icon class="is-loading">
              <Loading />
            </el-icon>
            <span>正在自动登录保存的账号...</span>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div
        v-if="accounts.length === 0"
        class="empty-state"
      >
        <div class="empty-icon">
          <svg
            class="w-16 h-16 text-gray-300 dark:text-gray-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
            />
          </svg>
        </div>
        <h3 class="empty-title\">
          暂无登录账号
        </h3>
        <p class="empty-description">
          登录 Apple ID 账号以开始使用
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
	User,
	Lock,
	Key,
	Right,
	Delete,
	Loading,
	Refresh,
} from '@element-plus/icons-vue'

const emit = defineEmits(['accounts-updated'])

const accounts = ref([])
const savedCredentials = ref([]) // 保存的账号密码（仅邮箱）
const newAccount = ref({
	email: '',
	password: '',
	code: '',
})
const logging = ref(false)
const autoLogging = ref(false)
const savePassword = ref(true) // 默认保存密码
const refreshingIndex = ref(null) // 正在刷新的账号索引

// 表单验证
const isFormValid = computed(() => {
	return newAccount.value.email && newAccount.value.password
})

const API_BASE = '/api'

// 加载保存的凭证列表（仅邮箱）
const loadSavedCredentials = async () => {
	try {
		const response = await fetch(`${API_BASE}/credentials`)
		const data = await response.json()

		if (data.ok && data.data) {
			savedCredentials.value = data.data
		}
	} catch (error) {
		console.error('Failed to load saved credentials:', error)
	}
}

const loadAccounts = async () => {
	// 先从 localStorage 加载（用于显示）
	const saved = localStorage.getItem('ipa_accounts')
	if (saved) {
		try {
			accounts.value = JSON.parse(saved)
		} catch {
			accounts.value = []
		}
	}

	// 然后从服务器获取最新的已登录账号列表
	try {
		const response = await fetch(`${API_BASE}/accounts`)
		const data = await response.json()

		if (data.ok && data.data) {
			// 同步服务器账号列表到本地
			accounts.value = data.data.map((acc) => ({
				token: acc.token,
				email: acc.email,
				dsid: acc.dsid,
				region: acc.region || 'US',
			}))
			saveAccounts()
		}
	} catch (error) {
		console.error('Failed to load accounts from server:', error)
	}
}

const saveAccounts = () => {
	localStorage.setItem('ipa_accounts', JSON.stringify(accounts.value))
	emit('accounts-updated', accounts.value)
}

const loginAccount = async () => {
	if (!newAccount.value.email || !newAccount.value.password) {
		ElMessage.warning('请填写完整的账号信息')
		return
	}

	// 检查账号是否已存在
	const existingAccount = accounts.value.find(
		(acc) => acc.email === newAccount.value.email,
	)
	if (existingAccount) {
		ElMessage.warning('该账号已登录，无需重复登录')
		return
	}

	logging.value = true

	// 尝试登录，不带MFA代码
	try {
		const response = await fetch(`${API_BASE}/login`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				email: newAccount.value.email,
				password: newAccount.value.password,
				mfa: newAccount.value.code || undefined,
				saveCredentials: savePassword.value,
			}),
		})

		const data = await response.json()

		// MFA needed — backend returns ok=true with status=need_mfa
		if (data.ok && data.data?.status === 'need_mfa') {
			ElMessage.warning('此账号需要二次验证，请在验证码输入框输入 6 位数验证码后，再次点击登录')
			logging.value = false
			return
		}

		if (!data.ok) {
			ElMessage.error(`登录失败：${data.error || '未知错误'}`)
			logging.value = false
			return
		}

		// 登录成功，保存账号信息
		accounts.value.push({
			token: data.token,
			email: data.email,
			dsid: data.dsid,
			region: data.region || 'US',
		})

		// 更新保存的凭证列表
		await loadSavedCredentials()

		saveAccounts()

		// 重置表单
		newAccount.value = { email: '', password: '', code: '' }

		ElMessage.success(`登录成功：${data.email}`)
	} catch (error) {
		ElMessage.error(`登录失败：${error.message}`)
	} finally {
		logging.value = false
	}
}

const removeAccount = async (index) => {
	if (confirm('确定要删除这个账号吗？')) {
		const account = accounts.value[index]

		// 从服务器删除账号（会同时删除保存的凭证）
		try {
			const response = await fetch(`${API_BASE}/accounts/${account.token}`, {
				method: 'DELETE',
			})

			if (response.ok) {
				accounts.value.splice(index, 1)
				saveAccounts()
				// 更新保存的凭证列表
				await loadSavedCredentials()
			} else {
				ElMessage.warning('删除失败')
			}
		} catch (error) {
			console.error('Failed to remove account:', error)
			ElMessage.warning('删除失败')
		}
	}
}

// 刷新账号会话
const refreshAccount = async (index) => {
	const account = accounts.value[index]
	if (!account) return

	refreshingIndex.value = index

	try {
		const response = await fetch(`${API_BASE}/login/refresh`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ token: account.token })
		})

		const data = await response.json()

		if (data.ok) {
			ElMessage.warning('账号会话已刷新')
			// 刷新账号列表以获取最新信息
			await fetchAccounts()
		} else {
			ElMessage.error(`刷新失败: ${data.error}`)
		}
	} catch (error) {
		console.error('Failed to refresh account:', error)
		ElMessage.warning('刷新失败，请检查网络连接')
	} finally {
		refreshingIndex.value = null
	}
}

// 自动登录所有保存的账号
const autoLoginAll = async () => {
	if (savedCredentials.value.length === 0) return

	autoLogging.value = true

	try {
		const response = await fetch(`${API_BASE}/auto-login`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
		})

		const data = await response.json()

		if (data.ok && data.results) {
			const { success, needCode, failed } = data.results

			// 添加成功登录的账号
			success.forEach((result) => {
				if (!result.alreadyLoggedIn) {
					accounts.value.push({
						token: result.token,
						email: result.email,
						dsid: result.dsid,
					})
				}
			})

			saveAccounts()

			// 显示自动登录结果
			if (success.length > 0 || needCode.length > 0 || failed.length > 0) {
				let message = ''
				if (success.length > 0) {
					message += `成功登录 ${success.length} 个账号`
				}
				if (needCode.length > 0) {
					if (message) message += '，'
					message += `${needCode.length} 个账号需要验证码`
				}
				if (failed.length > 0) {
					if (message) message += '，'
					message += `${failed.length} 个账号登录失败`
				}

				// 延迟显示，避免打扰用户
				setTimeout(() => {
					if (
						success.length > 0 &&
						needCode.length === 0 &&
						failed.length === 0
					) {
						// 全部成功，不显示提示
					} else {
						ElMessage.info(message)
					}
				}, 500)
			}
		}
	} catch (error) {
		console.error('Auto login failed:', error)
	} finally {
		autoLogging.value = false
	}
}

onMounted(async () => {
	// 先加载保存的凭证列表
	await loadSavedCredentials()

	// 加载已登录账号
	await loadAccounts()

	// 尝试自动登录保存的账号
	await autoLoginAll()

	emit('accounts-updated', accounts.value)
})

// 获取区域标签
const getRegionLabel = (region) => {
	const regionMap = {
		US: '🇺🇸 US',
		CN: '🇨🇳 CN',
		JP: '🇯🇵 JP',
		GB: '🇬🇧 GB',
		DE: '🇩🇪 DE',
		FR: '🇫🇷 FR',
		CA: '🇨🇦 CA',
		AU: '🇦🇺 AU',
	}
	return regionMap[region] || region
}

// 暴露账号列表供其他组件使用
defineExpose({
	accounts,
})
</script>

<style scoped>
.account-manager {
	padding: 0;
}

.account-header {
	display: flex;
	align-items: center;
	gap: 12px;
	margin-bottom: 24px;
	padding: 20px;
	background: linear-gradient(
		135deg,
		rgba(16, 185, 129, 0.1) 0%,
		rgba(5, 150, 105, 0.1) 100%
	);
	border-radius: 16px;
	border: 1px solid rgba(16, 185, 129, 0.2);
}

.dark .account-header {
	background: linear-gradient(
		135deg,
		rgba(16, 185, 129, 0.15) 0%,
		rgba(5, 150, 105, 0.15) 100%
	);
	border-color: rgba(16, 185, 129, 0.3);
}

.header-icon {
	width: 48px;
	height: 48px;
	background: linear-gradient(135deg, #10b981 0%, #059669 100%);
	border-radius: 12px;
	display: flex;
	align-items: center;
	justify-content: center;
	box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.header-text h2 {
	font-size: 20px;
	font-weight: 700;
	color: #111827;
	margin: 0;
}

.dark .header-text h2 {
	color: #f9fafb;
}

.header-text p {
	font-size: 14px;
	color: #6b7280;
	margin: 4px 0 0 0;
}

.dark .header-text p {
	color: #9ca3af;
}

.account-content {
	display: flex;
	flex-direction: column;
	gap: 24px;
}

/* 表单区域 */
.form-section {
	background: #ffffff;
	border-radius: 16px;
	padding: 24px;
	box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	border: 1px solid #e5e7eb;
}

.dark .form-section {
	background: rgba(31, 41, 55, 0.8);
	border-color: rgba(55, 65, 81, 0.5);
}

.form-header {
	margin-bottom: 20px;
}

.form-title {
	font-size: 16px;
	font-weight: 600;
	color: #111827;
	margin: 0 0 4px 0;
}

.dark .form-title {
	color: #f9fafb;
}

.form-subtitle {
	font-size: 13px;
	color: #6b7280;
	margin: 0;
}

.dark .form-subtitle {
	color: #9ca3af;
}

.form-fields {
	display: flex;
	flex-direction: column;
	gap: 16px;
}

.form-field {
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.field-label {
	font-size: 13px;
	font-weight: 500;
	color: #374151;
}

.dark .field-label {
	color: #d1d5db;
}

.form-input :deep(.el-input__wrapper) {
	border-radius: 10px;
	padding: 8px 12px;
	transition: all 0.2s ease;
}

.form-input :deep(.el-input__wrapper:hover) {
	box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.form-input :deep(.el-input__wrapper.is-focus) {
	box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.field-icon {
	color: #9ca3af;
	font-size: 16px;
}

.submit-button {
	width: 100%;
	border-radius: 10px;
	font-weight: 600;
	height: 44px;
	margin-top: 8px;
	transition: all 0.2s ease;
}

.submit-button:hover:not(:disabled) {
	transform: translateY(-1px);
	box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.submit-button:active:not(:disabled) {
	transform: translateY(0);
}

/* 账号列表区域 */
.accounts-section {
	margin-bottom: 20px;
	background: #ffffff;
	border-radius: 16px;
	padding: 20px;
	box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	border: 1px solid #e5e7eb;
}

.dark .accounts-section {
	background: rgba(31, 41, 55, 0.8);
	border-color: rgba(55, 65, 81, 0.5);
}

.section-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	margin-bottom: 16px;
}

.section-title {
	font-size: 16px;
	font-weight: 600;
	color: #111827;
	margin: 0;
}

.dark .section-title {
	color: #f9fafb;
}

.section-count {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	min-width: 24px;
	height: 24px;
	padding: 0 8px;
	background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
	color: #ffffff;
	font-size: 12px;
	font-weight: 600;
	border-radius: 12px;
}

.accounts-list {
	display: flex;
	flex-direction: column;
	gap: 12px;
}

.account-item {
	display: flex;
	align-items: center;
	gap: 12px;
	padding: 16px;
	background: #f9fafb;
	border-radius: 12px;
	border: 1px solid #e5e7eb;
	transition: all 0.2s ease;
}

.dark .account-item {
	background: rgba(17, 24, 39, 0.5);
	border-color: rgba(55, 65, 81, 0.5);
}

.account-item:hover {
	background: #f3f4f6;
	border-color: #d1d5db;
	transform: translateX(4px);
}

.dark .account-item:hover {
	background: rgba(17, 24, 39, 0.8);
	border-color: rgba(75, 85, 99, 0.8);
}

.account-avatar {
	width: 40px;
	height: 40px;
	background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
	border-radius: 10px;
	display: flex;
	align-items: center;
	justify-content: center;
	color: #ffffff;
	font-size: 18px;
	flex-shrink: 0;
}

.account-info {
	flex: 1;
	min-width: 0;
}

.account-email {
	font-size: 14px;
	font-weight: 500;
	color: #111827;
	margin: 0 0 2px 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.dark .account-email {
	color: #f9fafb;
}

.account-dsid {
	font-size: 12px;
	color: #6b7280;
	margin: 0;
}

.dark .account-dsid {
	color: #9ca3af;
}

.account-region {
	font-size: 12px;
	color: #6b7280;
	margin: 2px 0 0 0;
}

.dark .account-region {
	color: #9ca3af;
}

.region-badge {
	display: inline-flex;
	align-items: center;
	padding: 2px 8px;
	border-radius: 6px;
	font-size: 11px;
	font-weight: 600;
	letter-spacing: 0.5px;
}

.region-us {
	background: linear-gradient(
		135deg,
		rgba(59, 130, 246, 0.15) 0%,
		rgba(37, 99, 235, 0.15) 100%
	);
	color: #3b82f6;
	border: 1px solid rgba(59, 130, 246, 0.3);
}

.dark .region-us {
	background: linear-gradient(
		135deg,
		rgba(59, 130, 246, 0.25) 0%,
		rgba(37, 99, 235, 0.25) 100%
	);
	color: #60a5fa;
	border-color: rgba(59, 130, 246, 0.4);
}

.region-cn {
	background: linear-gradient(
		135deg,
		rgba(239, 68, 68, 0.15) 0%,
		rgba(220, 38, 38, 0.15) 100%
	);
	color: #ef4444;
	border: 1px solid rgba(239, 68, 68, 0.3);
}

.dark .region-cn {
	background: linear-gradient(
		135deg,
		rgba(239, 68, 68, 0.25) 0%,
		rgba(220, 38, 38, 0.25) 100%
	);
	color: #f87171;
	border-color: rgba(239, 68, 68, 0.4);
}

.region-jp {
	background: linear-gradient(
		135deg,
		rgba(16, 185, 129, 0.15) 0%,
		rgba(5, 150, 105, 0.15) 100%
	);
	color: #10b981;
	border: 1px solid rgba(16, 185, 129, 0.3);
}

.dark .region-jp {
	background: linear-gradient(
		135deg,
		rgba(16, 185, 129, 0.25) 0%,
		rgba(5, 150, 105, 0.25) 100%
	);
	color: #34d399;
	border-color: rgba(16, 185, 129, 0.4);
}

.region-gb,
.region-de,
.region-fr,
.region-ca,
.region-au {
	background: linear-gradient(
		135deg,
		rgba(139, 92, 246, 0.15) 0%,
		rgba(124, 58, 237, 0.15) 100%
	);
	color: #8b5cf6;
	border: 1px solid rgba(139, 92, 246, 0.3);
}

.dark .region-gb,
.dark .region-de,
.dark .region-fr,
.dark .region-ca,
.dark .region-au {
	background: linear-gradient(
		135deg,
		rgba(139, 92, 246, 0.25) 0%,
		rgba(124, 58, 237, 0.25) 100%
	);
	color: #a78bfa;
	border-color: rgba(139, 92, 246, 0.4);
}

.remove-button {
	flex-shrink: 0;
	transition: all 0.2s ease;
}

.remove-button:hover {
	transform: scale(1.1);
}

.refresh-button {
	margin-right: 8px;
	flex-shrink: 0;
	transition: all 0.2s ease;
}

.refresh-button:hover {
	transform: scale(1.1);
}

.account-actions {
	display: flex;
	align-items: center;
	gap: 8px;
}

/* 空状态 */
.empty-state {
	text-align: center;
	padding: 48px 24px;
	background: #ffffff;
	border-radius: 16px;
	border: 2px dashed #e5e7eb;
}

.dark .empty-state {
	background: rgba(31, 41, 55, 0.5);
	border-color: rgba(55, 65, 81, 0.5);
}

.empty-icon {
	display: flex;
	justify-content: center;
	margin-bottom: 16px;
}

.empty-title {
	font-size: 16px;
	font-weight: 600;
	color: #111827;
	margin: 0 0 8px 0;
}

.dark .empty-title {
	color: #f9fafb;
}

.empty-description {
	font-size: 14px;
	color: #6b7280;
	margin: 0;
}

.dark .empty-description {
	color: #9ca3af;
}

/* 自动登录状态 */
.auto-login-status {
	display: flex;
	align-items: center;
	justify-content: center;
	gap: 8px;
	padding: 12px;
	background: linear-gradient(
		135deg,
		rgba(59, 130, 246, 0.1) 0%,
		rgba(37, 99, 235, 0.1) 100%
	);
	border: 1px solid rgba(59, 130, 246, 0.2);
	border-radius: 10px;
	color: #3b82f6;
	font-size: 14px;
	font-weight: 500;
}

.dark .auto-login-status {
	background: linear-gradient(
		135deg,
		rgba(59, 130, 246, 0.15) 0%,
		rgba(37, 99, 235, 0.15) 100%
	);
	border-color: rgba(59, 130, 246, 0.3);
	color: #60a5fa;
}

.auto-login-status .el-icon {
	font-size: 16px;
	animation: spin 1s linear infinite;
}

@keyframes spin {
	from {
		transform: rotate(0deg);
	}
	to {
		transform: rotate(360deg);
	}
}

/* 保存密码复选框 */
.save-password-checkbox {
	margin-top: 4px;
}

.save-password-checkbox :deep(.el-checkbox__label) {
	font-size: 13px;
	color: #6b7280;
}

.dark .save-password-checkbox :deep(.el-checkbox__label) {
	color: #9ca3af;
}

.checkbox-label {
	font-size: 13px;
	color: #6b7280;
}

.dark .checkbox-label {
	color: #9ca3af;
}

/* 响应式设计 */
@media (max-width: 640px) {
	.account-header {
		padding: 16px;
	}

	.form-section,
	.accounts-section {
		padding: 16px;
	}

	.account-item {
		padding: 12px;
	}
}
</style>
