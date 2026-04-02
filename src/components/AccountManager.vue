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
          Account Management
        </h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Manage Apple ID accounts
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
          Logged-in Accounts
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
              :title="account.hasSavedCredentials ? 'Refresh Session' : 'No saved password, cannot auto-refresh'"
              :disabled="!account.hasSavedCredentials"
              :loading="refreshingIndex === index"
              @click="refreshAccount(index)"
            />
            <el-button
              type="danger"
              :icon="Delete"
              circle
              size="small"
              class="remove-button"
              title="Delete Account"
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
            Sign in with Apple ID
          </h3>
          <p class="form-subtitle">
            Supports multiple accounts
          </p>
        </div>
        <div class="form-fields">
          <div class="form-field">
            <label class="field-label">Email</label>
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
            <label class="field-label">Password</label>
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
            <label class="field-label">App Store Region</label>
            <el-select
              v-model="newAccount.region"
              :disabled="logging"
              size="large"
              filterable
              placeholder="Select your account's App Store region"
              class="form-input w-full"
            >
              <el-option
                v-for="r in REGION_OPTIONS"
                :key="r.value"
                :label="r.label"
                :value="r.value"
              />
            </el-select>
            <p class="field-hint">
              Select the App Store region this Apple ID belongs to. This ensures searches and downloads use the correct storefront.
            </p>
          </div>

          <div class="form-field">
            <label class="field-label">Verification Code</label>
            <el-input
              v-model="newAccount.code"
              type="text"
              placeholder="Two-factor code (if required)"
              :disabled="logging"
              size="large"
              clearable
              class="form-input"
              :class="{ 'mfa-highlight': mfaRequired }"
            >
              <template #prefix>
                <el-icon class="field-icon">
                  <Key />
                </el-icon>
              </template>
            </el-input>
            <p
              v-if="mfaRequired"
              class="mfa-hint"
            >
              ⚠️ Please enter the 6-digit code received on your trusted device
            </p>
          </div>

          <!-- Save password option -->
          <div class="form-field">
            <el-checkbox
              v-model="savePassword"
              :disabled="logging"
              class="save-password-checkbox"
            >
              <span class="checkbox-label">Save password for automatic login next time</span>
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
            {{ logging ? 'Signing in...' : 'Sign In' }}
          </el-button>

          <!-- Auto-login status message -->
          <div
            v-if="autoLogging"
            class="auto-login-status"
          >
            <el-icon class="is-loading">
              <Loading />
            </el-icon>
            <span>Automatically signing in to saved accounts...</span>
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
          No accounts signed in
        </h3>
        <p class="empty-description">
          Sign in with an Apple ID to get started
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
const savedCredentials = ref([]) // Saved credentials (email only)
const accountIdentityKey = (acc = {}) => String(acc.email || acc.dsid || acc.token || '').trim().toLowerCase()
const dedupeAccounts = (list = []) => {
	const map = new Map()
	for (const acc of list) {
		const key = accountIdentityKey(acc)
		if (!key) continue
		map.set(key, acc)
	}
	return [...map.values()]
}
const newAccount = ref({
	email: '',
	password: '',
	code: '',
	region: 'US',
})
const logging = ref(false)
const autoLogging = ref(false)
const savePassword = ref(true) // Save password by default
const refreshingIndex = ref(null) // Index of account being refreshed
const mfaRequired = ref(false) // Whether waiting for MFA input

// Form validation
const isFormValid = computed(() => {
	return newAccount.value.email && newAccount.value.password
})

const API_BASE = '/api'

// Load saved credentials list (email only)
const loadSavedCredentials = async () => {
	try {
		const response = await fetch(`${API_BASE}/credentials`, { credentials: 'include' })
		const data = await response.json()

		if (data.ok && data.data) {
			savedCredentials.value = data.data
		}
	} catch (error) {
		console.error('Failed to load saved credentials:', error)
	}
}

const loadAccounts = async () => {
	// First load from localStorage (for display)
	const saved = localStorage.getItem('ipa_accounts')
	if (saved) {
		try {
			accounts.value = dedupeAccounts(JSON.parse(saved))
		} catch {
			accounts.value = []
		}
	}

		// Then fetch the latest logged-in account list from server
	try {
		const response = await fetch(`${API_BASE}/accounts`, { credentials: 'include' })
		const data = await response.json()

		if (data.ok && data.data && data.data.length > 0) {
			// Sync server account list to local
			accounts.value = dedupeAccounts(data.data.map((acc) => ({
				token: acc.token,
				email: acc.email,
				dsid: acc.dsid,
				region: acc.region || 'US',
				hasSavedCredentials: !!acc.hasSavedCredentials,
			})))
			saveAccounts()
		} else if (data.ok && (!data.data || data.data.length === 0)) {
			// No accounts on server, attempt auto-restore with saved credentials
			try {
				const autoRes = await fetch(`${API_BASE}/auto-login`, { credentials: 'include', method: 'POST' })
				const autoData = await autoRes.json()
				if (autoData.ok && autoData.data?.succeeded?.length > 0) {
					const retryRes = await fetch(`${API_BASE}/accounts`, { credentials: 'include' })
					const retryData = await retryRes.json()
					if (retryData.ok && retryData.data) {
						accounts.value = dedupeAccounts(retryData.data.map((acc) => ({
							token: acc.token,
							email: acc.email,
							dsid: acc.dsid,
							region: acc.region || 'US',
							hasSavedCredentials: !!acc.hasSavedCredentials,
						})))
						saveAccounts()
					}
				}
			} catch (e) {
				console.warn('Auto-login restore failed:', e)
			}
		}
	} catch (error) {
		console.error('Failed to load accounts from server:', error)
	}
}

const saveAccounts = () => {
	accounts.value = dedupeAccounts(accounts.value)
	localStorage.setItem('ipa_accounts', JSON.stringify(accounts.value))
	emit('accounts-updated', accounts.value)
}

const loginAccount = async () => {
	if (!newAccount.value.email || !newAccount.value.password) {
		ElMessage.warning('Please fill in all account information')
		return
	}

	// Check if account already exists
	const existingAccount = accounts.value.find(
		(acc) => acc.email === newAccount.value.email,
	)
	if (existingAccount) {
		ElMessage.warning('This account is already signed in')
		return
	}

	logging.value = true

	try {
		const response = await fetch(`${API_BASE}/login`, {
			credentials: 'include',
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				email: newAccount.value.email,
				password: newAccount.value.password,
				mfa: newAccount.value.code || undefined,
				saveCredentials: savePassword.value,
				region: newAccount.value.region || 'US',
			}),
		})

		const data = await response.json()

		// Network/server error
		if (!response.ok && !data.ok) {
			ElMessage.error(`Login failed: ${data.error || 'Server error'}`)
			logging.value = false
			return
		}

		// MFA needed — first round, no code provided yet
		if (data.ok && data.data?.status === 'need_mfa') {
			mfaRequired.value = true
			ElMessage({
				type: 'warning',
				message: 'This account requires two-factor authentication. Please check the verification code on your trusted device and click Sign In again',
				duration: 8000,
			})
			logging.value = false
			return
		}

		// MFA code was wrong/expired — keep the session, let user retry
		if (data.ok && data.data?.status === 'mfa_failed') {
			ElMessage.error('Verification code is invalid or expired, please try again')
			newAccount.value.code = ''
			logging.value = false
			return
		}

		// Business logic error (bad password, account locked, etc.)
		if (!data.ok) {
			const errMsg = data.error || 'Unknown error'
			ElMessage.error(`Login failed: ${errMsg}`)
			// If it looks like a credential error, hint about MFA
			if (errMsg.includes('password') || errMsg.includes('BadLogin')) {
				mfaRequired.value = true
			}
			logging.value = false
			return
		}

		// Login success
		mfaRequired.value = false
		accounts.value = dedupeAccounts([
			...accounts.value,
			{
				token: data.data.token,
				email: data.data.email,
				dsid: data.data.dsid,
				region: data.data.region || 'US',
				hasSavedCredentials: !!savePassword.value,
			}
		])

		// Update saved credentials list
		await loadSavedCredentials()

		saveAccounts()

		// Reset form
		newAccount.value = { email: '', password: '', code: '', region: 'US' }

		ElMessage.success(`Signed in successfully: ${data.data.email}`)
	} catch (error) {
		ElMessage.error(`Network error: ${error.message}`)
	} finally {
		logging.value = false
	}
}

const removeAccount = async (index) => {
	if (confirm('Are you sure you want to delete this account?')) {
		const account = accounts.value[index]

		// Delete account from server (also removes saved credentials)
		try {
			const response = await fetch(`${API_BASE}/accounts/${account.token}`, {
				credentials: 'include',
				method: 'DELETE',
			})

			if (response.ok) {
				accounts.value.splice(index, 1)
				saveAccounts()
				// Update saved credentials list
				await loadSavedCredentials()
			} else {
				ElMessage.warning('Delete failed')
			}
		} catch (error) {
			console.error('Failed to remove account:', error)
			ElMessage.warning('Delete failed')
		}
	}
}

// Refresh account session
const refreshAccount = async (index) => {
	const account = accounts.value[index]
	if (!account) return

	if (!account.hasSavedCredentials) {
		ElMessage.warning('This account has no saved password and cannot be auto-refreshed. Please sign in again and check "Save password".')
		return
	}

	refreshingIndex.value = index
	ElMessage.info(`Account detected in database, refreshing session for ${account.email}…`)

	try {
		const response = await fetch(`${API_BASE}/login/refresh`, {
			credentials: 'include',
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ token: account.token })
		})

		const data = await response.json()

		if (data.ok) {
			// Refresh account list to get latest info
			await loadSavedCredentials()
			await loadAccounts()
			ElMessage.success('Account session refreshed and page state synced')
		} else {
			const errMsg = data.error || 'Refresh failed'
			if (errMsg.includes('No saved password found')) {
				ElMessage.error('Refresh failed: this account has no saved password. Please sign in again and check "Save password".')
			} else {
				ElMessage.error(`Refresh failed: ${errMsg}`)
			}
		}
	} catch (error) {
		console.error('Failed to refresh account:', error)
		ElMessage.warning('Refresh failed, please check your network connection')
	} finally {
		refreshingIndex.value = null
	}
}

// Auto-login all saved accounts
const autoLoginAll = async () => {
	if (savedCredentials.value.length === 0) return

	autoLogging.value = true

	try {
		const response = await fetch(`${API_BASE}/auto-login`, {
			credentials: 'include',
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
		})

		const data = await response.json()

		if (data.ok && data.results) {
			const { success, needCode, failed } = data.results

			// Add successfully logged-in accounts
			success.forEach((result) => {
				if (!result.alreadyLoggedIn) {
					accounts.value.push({
						token: result.token,
						email: result.email,
						dsid: result.dsid,
						region: result.region || 'US',
						hasSavedCredentials: true,
					})
				}
			})

			saveAccounts()
			await loadAccounts()

			// Show auto-login results
			if (success.length > 0 || needCode.length > 0 || failed.length > 0) {
				let message = ''
				if (success.length > 0) {
					message += `Successfully signed in ${success.length} account(s)`
				}
				if (needCode.length > 0) {
					if (message) message += '，'
					message += `${needCode.length} account(s) require verification code`
				}
				if (failed.length > 0) {
					if (message) message += '，'
					message += `${failed.length} account(s) failed to sign in`
				}

				// Delay display to avoid interrupting user
				setTimeout(() => {
					if (
						success.length > 0 &&
						needCode.length === 0 &&
						failed.length === 0
					) {
						// All succeeded, no notification needed
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
	// First load saved credentials
	await loadSavedCredentials()

		// Load signed-in accounts
	await loadAccounts()

	// Attempt to auto-login saved accounts
	await autoLoginAll()

	emit('accounts-updated', accounts.value)
})

// Comprehensive App Store region map
const REGION_MAP = {
	US: '🇺🇸 United States',
	GB: '🇬🇧 United Kingdom',
	CA: '🇨🇦 Canada',
	AU: '🇦🇺 Australia',
	NZ: '🇳🇿 New Zealand',
	DE: '🇩🇪 Germany',
	FR: '🇫🇷 France',
	NL: '🇳🇱 Netherlands',
	BE: '🇧🇪 Belgium',
	AT: '🇦🇹 Austria',
	CH: '🇨🇭 Switzerland',
	ES: '🇪🇸 Spain',
	IT: '🇮🇹 Italy',
	PT: '🇵🇹 Portugal',
	SE: '🇸🇪 Sweden',
	NO: '🇳🇴 Norway',
	DK: '🇩🇰 Denmark',
	FI: '🇫🇮 Finland',
	PL: '🇵🇱 Poland',
	CZ: '🇨🇿 Czech Republic',
	HU: '🇭🇺 Hungary',
	RO: '🇷🇴 Romania',
	GR: '🇬🇷 Greece',
	TR: '🇹🇷 Turkey',
	RU: '🇷🇺 Russia',
	UA: '🇺🇦 Ukraine',
	IE: '🇮🇪 Ireland',
	SK: '🇸🇰 Slovakia',
	HR: '🇭🇷 Croatia',
	BG: '🇧🇬 Bulgaria',
	LT: '🇱🇹 Lithuania',
	LV: '🇱🇻 Latvia',
	EE: '🇪🇪 Estonia',
	SI: '🇸🇮 Slovenia',
	LU: '🇱🇺 Luxembourg',
	MT: '🇲🇹 Malta',
	CY: '🇨🇾 Cyprus',
	IS: '🇮🇸 Iceland',
	CN: '🇨🇳 China',
	JP: '🇯🇵 Japan',
	KR: '🇰🇷 South Korea',
	HK: '🇭🇰 Hong Kong',
	TW: '🇹🇼 Taiwan',
	SG: '🇸🇬 Singapore',
	MY: '🇲🇾 Malaysia',
	TH: '��🇭 Thailand',
	ID: '🇮🇩 Indonesia',
	PH: '🇵🇭 Philippines',
	VN: '🇻🇳 Vietnam',
	IN: '🇮🇳 India',
	PK: '🇵🇰 Pakistan',
	BD: '🇧🇩 Bangladesh',
	LK: '🇱🇰 Sri Lanka',
	BR: '🇧🇷 Brazil',
	MX: '🇲🇽 Mexico',
	AR: '🇦🇷 Argentina',
	CL: '🇨🇱 Chile',
	CO: '🇨🇴 Colombia',
	PE: '🇵🇪 Peru',
	VE: '🇻🇪 Venezuela',
	EC: '🇪🇨 Ecuador',
	BO: '🇧🇴 Bolivia',
	UY: '🇺🇾 Uruguay',
	PY: '🇵🇾 Paraguay',
	SA: '🇸🇦 Saudi Arabia',
	AE: '🇦🇪 United Arab Emirates',
	IL: '🇮🇱 Israel',
	EG: '🇪🇬 Egypt',
	ZA: '🇿🇦 South Africa',
	NG: '🇳🇬 Nigeria',
	KE: '🇰🇪 Kenya',
	GH: '🇬🇭 Ghana',
	ET: '🇪🇹 Ethiopia',
	JO: '🇯🇴 Jordan',
	KW: '🇰🇼 Kuwait',
	QA: '🇶🇦 Qatar',
	BH: '🇧🇭 Bahrain',
	OM: '🇴🇲 Oman',
	LB: '🇱🇧 Lebanon',
}

// Sorted region options for the dropdown
const REGION_OPTIONS = Object.entries(REGION_MAP)
	.map(([value, label]) => ({ value, label: `${label} (${value})` }))
	.sort((a, b) => {
		// Pin US to top, then sort alphabetically
		if (a.value === 'US') return -1
		if (b.value === 'US') return 1
		return a.label.localeCompare(b.label)
	})

// Get display label for a region code
const getRegionLabel = (region) => {
	if (!region) return '🇺🇸 US'
	const code = region.toUpperCase()
	return REGION_MAP[code] ? `${REGION_MAP[code].split(' ').slice(1).join(' ')} (${code})` : code
}

// Expose account list for other components
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

/* Form area */
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

.field-hint {
	font-size: 11px;
	color: #6b7280;
	margin: 0;
	line-height: 1.4;
}

.dark .field-hint {
	color: #9ca3af;
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

/* Account list area */
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

/* Named European/other regions */
.region-gb,
.region-de,
.region-fr,
.region-ca,
.region-au,
.region-nl,
.region-be,
.region-at,
.region-ch,
.region-se,
.region-no,
.region-dk,
.region-fi,
.region-ie,
.region-nz,
.region-es,
.region-it,
.region-pt,
.region-pl,
.region-cz,
.region-hu,
.region-ro,
.region-gr,
.region-tr,
.region-ru,
.region-ua,
.region-sk,
.region-hr,
.region-bg,
.region-lt,
.region-lv,
.region-ee,
.region-si,
.region-lu,
.region-mt,
.region-cy,
.region-is {
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
.dark .region-au,
.dark .region-nl,
.dark .region-be,
.dark .region-at,
.dark .region-ch,
.dark .region-se,
.dark .region-no,
.dark .region-dk,
.dark .region-fi,
.dark .region-ie,
.dark .region-nz,
.dark .region-es,
.dark .region-it,
.dark .region-pt,
.dark .region-pl,
.dark .region-cz,
.dark .region-hu,
.dark .region-ro,
.dark .region-gr,
.dark .region-tr,
.dark .region-ru,
.dark .region-ua,
.dark .region-sk,
.dark .region-hr,
.dark .region-bg,
.dark .region-lt,
.dark .region-lv,
.dark .region-ee,
.dark .region-si,
.dark .region-lu,
.dark .region-mt,
.dark .region-cy,
.dark .region-is {
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

/* Empty state */
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

/* Auto-login status */
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

/* Save password checkbox */
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

/* MFA highlight hint */
.mfa-highlight :deep(.el-input__wrapper) {
	box-shadow: 0 0 0 3px rgba(245, 158, 11, 0.3) !important;
	border-color: #f59e0b !important;
}

.mfa-hint {
	font-size: 12px;
	color: #f59e0b;
	margin: 4px 0 0 0;
	font-weight: 500;
}

.dark .mfa-hint {
	color: #fbbf24;
}

/* Responsive design */
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
