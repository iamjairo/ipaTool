<template>
  <div class="card">
    <div class="flex items-center space-x-3 mb-6">
      <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-indigo-500 rounded-xl flex items-center justify-center shadow-lg">
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
        <h2 class="text-xl font-bold text-gray-900 dark:text-white">
          Download & Sign
        </h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Search apps, look up versions, and download IPA files
        </p>
      </div>
    </div>

    <!-- Search Section -->
    <div class="space-y-4 mb-6">
      <!-- Account selection notice -->
      <div
        v-if="accounts.length === 0"
        class="bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-xl p-4"
      >
        <div class="flex items-start space-x-3">
          <svg
            class="w-5 h-5 text-orange-600 dark:text-orange-400 mt-0.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
          <div class="flex-1">
            <h4 class="font-semibold text-orange-900 dark:text-orange-300">
              Account Login Required
            </h4>
            <p class="text-sm text-orange-700 dark:text-orange-400 mt-1">
              Please log in with an Apple ID in the "Accounts" tab before searching for apps.
            </p>
            <el-button 
              type="warning" 
              size="small" 
              class="mt-2" 
              plain
              @click="goToAccountTab"
            >
              Go to Login
            </el-button>
          </div>
        </div>
      </div>

      <!-- Account selection area -->
      <div
        v-else
        class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2 flex-1">
            <svg
              class="w-4 h-4 text-blue-600 dark:text-blue-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span class="text-sm text-blue-700 dark:text-blue-400">
              Search Region: <strong>{{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }}</strong>
            </span>
          </div>
          <el-select 
            v-model="selectedAccount"
            placeholder="Select Account"
            class="account-quick-select"
            size="small"
            @change="handleAccountChange"
          >
            <el-option
              v-for="(account, index) in accounts"
              :key="index"
              :label="account.email"
              :value="index"
            >
              <div class="account-option-row">
                <span class="account-option-email">{{ account.email }}</span>
                <span
                  class="region-badge-mini"
                  :class="`region-${(account.region || 'US').toLowerCase()}`"
                >
                  {{ getRegionLabel(account.region || 'US') }}
                </span>
              </div>
            </el-option>
          </el-select>
        </div>
      </div>

      <!-- Search Mode Toggle -->
      <div class="flex items-center space-x-4 p-3 bg-gray-50 dark:bg-gray-800 rounded-xl mb-3">
        <label class="flex items-center space-x-2 cursor-pointer">
          <input
            v-model="searchMode"
            type="radio"
            value="search"
            class="w-4 h-4 text-primary-600 focus:ring-primary-500 border-gray-300"
          >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Search Apps</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer">
          <input
            v-model="searchMode"
            type="radio"
            value="appid"
            class="w-4 h-4 text-primary-600 focus:ring-primary-500 border-gray-300"
          >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Enter App ID Directly</span>
        </label>
      </div>

      <el-input
        v-model="searchQuery"
        :placeholder="searchMode === 'search' ? 'Search app name, Bundle ID or App ID...' : 'Enter App ID (numbers only)...'"
        :prefix-icon="Search"
        :loading="searching"
        :disabled="accounts.length === 0"
        clearable
        size="large"
        class="search-input"
        @input="handleSearch"
        @keyup.enter="handleSearch"
      />

      <!-- Direct App ID Confirm Button -->
      <div
        v-if="searchMode === 'appid' && searchQuery && /^\d+$/.test(searchQuery.trim()) && !searching"
        class="flex items-center justify-between p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-xl mt-3"
      >
        <div class="flex-1">
          <p class="text-sm font-medium text-yellow-800 dark:text-yellow-300">
            App ID: <span class="font-bold">{{ searchQuery.trim() }}</span>
          </p>
          <p class="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
            You can continue to look up version info even if the app is not found
          </p>
        </div>
        <el-button
          type="primary"
          size="default"
          @click="confirmDirectAppId"
        >
          Confirm & Continue
        </el-button>
      </div>

      <!-- Search Results -->
      <el-scrollbar
        v-if="searchResults.length > 0"
        max-height="256px"
      >
        <div class="space-y-2">
          <div
            v-for="app in searchResults"
            :key="app.trackId"
            class="search-result-item flex items-center space-x-4 p-3 rounded-xl hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-all duration-200 border border-transparent hover:border-primary-300 dark:hover:border-primary-700"
            @click="selectApp(app)"
          >
            <img 
              :src="app.artworkUrl100 || app.artworkUrl60" 
              :alt="app.trackName"
              class="w-12 h-12 rounded-lg shadow-md object-cover"
            >
            <div class="flex-1 min-w-0">
              <h3 class="font-semibold text-gray-900 dark:text-white truncate text-sm">
                {{ app.trackName }}
              </h3>
              <p class="text-xs text-gray-500 dark:text-gray-400">
                {{ app.artistName }}
              </p>
            </div>
            <el-icon class="w-5 h-5 text-gray-400 flex-shrink-0">
              <ArrowRight />
            </el-icon>
          </div>
        </div>
      </el-scrollbar>
    </div>

    <div
      v-if="selectedApp"
      class="space-y-4"
    >
      <!-- Selected App Info -->
      <div class="selected-app-card bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-xl p-4 border border-blue-200 dark:border-blue-800">
        <div class="flex items-center space-x-4">
          <img 
            v-if="!selectedApp.isDirectAppId"
            :src="selectedApp.artworkUrl100 || selectedApp.artworkUrl60" 
            :alt="selectedApp.trackName"
            class="w-16 h-16 rounded-xl shadow-md object-cover"
          >
          <div 
            v-else
            class="w-16 h-16 rounded-xl shadow-md object-cover bg-gradient-to-br from-gray-400 to-gray-600 flex items-center justify-center"
          >
            <svg
              class="w-8 h-8 text-white"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <div class="flex-1">
            <h3 class="font-semibold text-gray-900 dark:text-white">
              {{ selectedApp.trackName }}
            </h3>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ selectedApp.artistName }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
              Version: {{ selectedApp.version }} | ID: {{ selectedApp.trackId }}
              <span
                v-if="selectedApp.isDirectAppId"
                class="ml-2 px-2 py-0.5 bg-yellow-200 dark:bg-yellow-800 text-yellow-800 dark:text-yellow-200 rounded-full text-xs"
              >
                Direct Input
              </span>
            </p>
            <div class="selected-app-badges mt-2">
              <span class="selected-app-badge">Price: {{ getSelectedAppPriceLabel() }}</span>
              <span class="selected-app-badge">Size: {{ getSelectedAppSizeLabel() }}</span>
              <span class="selected-app-badge">Purchase Status: {{ getPurchaseBehaviorLabel() }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Download Options -->
      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Select Account
            <span
              v-if="selectedAccount !== null && selectedAccount !== undefined && selectedAccount !== ''"
              class="ml-2 text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded-full"
            >
              Store Region: {{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }}
            </span>
          </label>
          <el-select 
            v-model="selectedAccount"
            placeholder="Please sign in first"
            class="w-full form-select"
            :disabled="accounts.length === 0"
            @change="handleAccountChange"
          >
            <el-option
              v-for="(account, index) in accounts"
              :key="index"
              :label="account.email"
              :value="index"
            >
              <div class="flex items-center justify-between w-full">
                <span class="flex-1 truncate">{{ account.email }}</span>
                <span
                  class="region-badge ml-2"
                  :class="`region-${(account.region || 'US').toLowerCase()}`"
                >
                  {{ getRegionLabel(account.region || 'US') }}
                </span>
              </div>
            </el-option>
          </el-select>
          <p
            v-if="accounts.length === 0"
            class="text-xs text-orange-600 dark:text-orange-400 mt-1"
          >
            ⚠️ Please sign in first
          </p>
          <p
            v-else
            class="text-xs text-gray-500 dark:text-gray-400 mt-1"
          >
            ✅ Search and download will use the {{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }} store for this account
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">APPID</label>
          <el-input
            v-model="appid"
            placeholder="e.g., 1160172628"
            class="form-input"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Version (history dropdown)</label>
          <el-select 
            v-model="selectedVersion"
            placeholder="Look up versions first"
            class="w-full form-select"
            :disabled="!versionsFetched"
            :loading="fetchingVersions"
            @change="handleVersionChange"
          >
            <el-option
              v-for="version in versions"
              :key="version.external_identifier"
              :label="`${version.bundle_version} | ${version.created_at}`"
              :value="version.external_identifier"
            />
          </el-select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">appVerId (auto-filled)</label>
          <el-input
            v-model="appVerId"
            placeholder="external_identifier"
            readonly
            class="form-input"
          />
        </div>

        <el-space
          direction="vertical"
          :size="12"
          fill
          style="width: 100%"
        >
          <el-button
            :disabled="!appid || fetchingVersions"
            :loading="fetchingVersions"
            type="info"
            class="w-full action-button"
            @click="fetchVersions"
          >
            <template #icon>
              <el-icon><Search /></el-icon>
            </template>
            Look Up Versions
          </el-button>

          <el-button
            v-if="!claimRequired"
            :disabled="(!selectedAccount && selectedAccount !== 0) || downloadBlocked"
            :class="{ 'purchase-blocked-btn': paidPurchaseRequired }"
            :title="downloadBlockedReason"
            type="info"
            class="w-full action-button"
            @click="directLinkDownload"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            Direct Link Download (file only)
          </el-button>

          <el-button
            v-if="!claimRequired"
            :disabled="(!selectedAccount && selectedAccount !== 0) || downloadBlocked"
            :loading="downloading"
            :class="{ 'purchase-blocked-btn': paidPurchaseRequired }"
            :title="downloadBlockedReason"
            type="primary"
            class="w-full action-button"
            @click="startDownloadWithProgress"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            {{ downloading ? 'Processing...' : 'Download to Server' }}
          </el-button>

          <div v-if="purchaseRequired" class="download-disabled-hint">
            ⚠️ {{ downloadBlockedReason }}
          </div>

          <el-button
            v-if="purchaseRequired"
            :disabled="!selectedAccount && selectedAccount !== 0"
            type="warning"
            class="w-full action-button"
            @click="buyOrClaimSelectedApp"
          >
            <template #icon>
              <el-icon><ArrowRight /></el-icon>
            </template>
            {{ purchaseActionLabel }}
          </el-button>

        </el-space>
      </div>

      <!-- Progress Box -->
      <el-card
        v-if="showProgress"
        class="mt-4"
        shadow="never"
      >
        <div class="flex justify-between items-center mb-2">
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{{ progressStage }}</span>
          <span class="text-sm font-bold text-blue-600 dark:text-blue-400">{{ progressPercent }}%</span>
        </div>
        <el-progress 
          :percentage="progressPercent" 
          :stroke-width="10"
          class="mb-3"
        />
        <el-scrollbar max-height="160px">
          <pre class="bg-black rounded-lg p-3 text-green-400 text-xs whitespace-pre-wrap font-mono">{{ logs }}</pre>
        </el-scrollbar>
        
        <div
          v-if="showActionButtons && (downloadReadyUrl || downloadInstallUrl)"
          class="mt-4 space-y-3"
        >
          <!-- Environment Warning -->
          <div
            v-if="!isHttps && currentProtocol !== 'http:'"
            class="mb-3 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg"
          >
            <div class="flex items-start space-x-2">
              <svg
                class="w-5 h-5 text-yellow-600 dark:text-yellow-400 mt-0.5 flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
              <div class="flex-1">
                <p class="text-sm text-yellow-800 dark:text-yellow-300 font-medium">
                  Environment Check
                </p>
                <p class="text-xs text-yellow-700 dark:text-yellow-400 mt-1">
                  Current protocol: {{ currentProtocol || 'Unknown' }} | iOS installation requires HTTPS
                </p>
              </div>
            </div>
          </div>
          
          <div class="grid gap-3 sm:grid-cols-2">
            <el-button
              v-if="downloadReadyUrl"
              type="primary"
              size="large"
              class="w-full"
              @click="downloadCompletedIpa"
            >
              <template #icon>
                <el-icon><Download /></el-icon>
              </template>
              Download IPA{{ downloadReadyFileSize ? ` (${formatFileSize(downloadReadyFileSize)})` : '' }}
            </el-button>
            <a
              v-if="downloadOtaInstallable && downloadInstallUrl && isHttps"
              :href="downloadInstallUrl"
              class="block w-full"
            >
              <el-button
                type="success"
                size="large"
                class="w-full"
              >
                <template #icon>
                  <el-icon><Download /></el-icon>
                </template>
                Install to Device
              </el-button>
            </a>
            <el-button
              v-else-if="downloadOtaInstallable && downloadInstallUrl"
              type="success"
              size="large"
              class="w-full"
              @click="installDownloadedIpa"
            >
              <template #icon>
                <el-icon><Download /></el-icon>
              </template>
              Install to Device
            </el-button>
            <el-tooltip v-else-if="downloadInstallMethod === 'download_only' && downloadInspection && downloadInspection.summary" :content="downloadInspection.summary" placement="top">
              <span class="block w-full">
                <el-tag size="large" type="info" class="w-full text-center">Download Only</el-tag>
              </span>
            </el-tooltip>
            <el-tag v-else-if="downloadInstallMethod === 'download_only'" size="large" type="info" class="w-full text-center">Download Only</el-tag>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 text-center">
            Download and installation are separate. Please proceed manually as needed.
          </p>
          <p
            v-if="downloadInstallUrl && !isHttps"
            class="text-xs text-orange-600 dark:text-orange-400 mt-1 text-center"
          >
            ⚠️ Per the OpenList/Oplist approach, OTA installation requires HTTPS + valid certificate + signed IPA. If opened in Telegram's built-in browser, please use Safari instead.
          </p>
        </div>
      </el-card>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="!searching && searchResults.length === 0 && !searchQuery.trim()"
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
          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
        />
      </svg>
      <p class="text-lg font-medium">
        No App Selected
      </p>
      <p class="text-sm mt-2">
        Search for and select an app above to get started
      </p>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, onMounted, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { useAppStore } from '../stores/app'
import { useNotifications } from '../composables/useNotifications'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, ArrowRight, Download } from '@element-plus/icons-vue'

const notifications = useNotifications()
const appStore = useAppStore()

const props = defineProps({
  selectedApp: {
    type: Object,
    default: null
  },
  accountsUpdated: {
    type: Number,
    default: 0
  }
})

// Make selectedApp reactive for template
const selectedApp = computed(() => props.selectedApp)

const emit = defineEmits(['download-started', 'app-selected'])

// Get region label
const getRegionLabel = (region) => {
  const regionMap = {
    'US': '🇺🇸 US',
    'CN': '🇨🇳 CN',
    'JP': '🇯🇵 JP',
    'GB': '🇬🇧 GB',
    'DE': '🇩🇪 DE',
    'FR': '🇫🇷 FR',
    'CA': '🇨🇦 CA',
    'AU': '🇦🇺 AU'
  }
  return regionMap[region] || region
}

// Handle account selection change
const handleAccountChange = () => {
  const account = accounts.value[selectedAccount.value]
  if (account) {
    console.log(`[DownloadManager] Selected account: ${account.email}, Region: ${account.region || 'US'}`)
  }
  
  // Clear previously fetched version info
  versions.value = []
  selectedVersion.value = ''
  appVerId.value = ''
  versionsFetched.value = false
  
  // Sync state to store
  syncStateToStore()
}

// Auto-select first account
const autoSelectFirstAccount = () => {
  if (accounts.value.length > 0 && (selectedAccount.value === null || selectedAccount.value === undefined || selectedAccount.value === '')) {
    // Try to restore previously selected account from localStorage
    const savedAccountIndex = localStorage.getItem('ipa_selected_account_index')
    if (savedAccountIndex !== null && savedAccountIndex !== '' && !isNaN(parseInt(savedAccountIndex)) && parseInt(savedAccountIndex) < accounts.value.length) {
      selectedAccount.value = parseInt(savedAccountIndex)
      console.log(`[DownloadManager] Restored selected account: ${accounts.value[selectedAccount.value].email}`)
    } else {
      selectedAccount.value = 0
      console.log(`[DownloadManager] Auto-selected first account: ${accounts.value[0].email}`)
    }
  }
}

const accounts = ref([])
const selectedAccount = ref(null)  // Use null instead of empty string

// Watch account selection and save to localStorage
watch(selectedAccount, (newValue) => {
  if (newValue !== null && newValue !== undefined && newValue !== '') {
    localStorage.setItem('ipa_selected_account_index', String(newValue))
    console.log(`[DownloadManager] Saved selected account index: ${newValue}`)
  }
})
const appid = ref('')
const appVerId = ref('')
const versions = ref([])
const selectedVersion = ref('')
const versionsFetched = ref(false)
const fetchingVersions = ref(false)
const downloading = ref(false)
const checkingPurchaseStatus = ref(false)
const purchaseStatusText = ref('Pending')
const purchaseStatus = ref({ purchased: null, needsPurchase: false, status: 'unknown', error: null })

// Progress state - sync with store
const showProgress = ref(false)
const progressPercent = ref(0)
const progressStage = ref('Waiting for task…')
const logs = ref('')

// Search state
const searchMode = ref('search') // 'search' or 'appid'
const searchQuery = ref('')
const searchResults = ref([])
const searching = ref(false)

// Download/install state
const downloadReadyUrl = ref('')
const downloadReadyFileSize = ref(0)
const downloadInstallUrl = ref('')
const downloadPackageKind = ref('')
const downloadOtaInstallable = ref(false)
const downloadInstallMethod = ref('')
const downloadInspection = ref(null)
const showActionButtons = ref(false)

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

const purchaseRequired = computed(() => !!purchaseStatus.value.needsPurchase)
const claimRequired = computed(() => {
  if (!purchaseRequired.value) return false
  const price = getSelectedAppPrice()
  return price !== null && price <= 0
})
const paidPurchaseRequired = computed(() => purchaseRequired.value && !claimRequired.value)
const downloadBlocked = computed(() => checkingPurchaseStatus.value || purchaseRequired.value)
const downloadBlockedReason = computed(() => {
  if (checkingPurchaseStatus.value) return 'Checking purchase status…'
  if (!purchaseRequired.value) return ''
  const price = getSelectedAppPrice()
  if (price !== null && price > 0) return 'Not purchased: please purchase in the App Store first'
  return 'Not claimed: please tap "Get" in the official App Store first'
})
const purchaseActionLabel = computed(() => {
  const price = getSelectedAppPrice()
  if (price !== null && price > 0) return 'Go to Purchase'
  return 'Get on App Store'
})

// HTTPS detection
const isHttps = ref(false)
const currentProtocol = ref('')

// Sync state with store on mount and update
const syncStateToStore = () => {
  appStore.updateDownloadState('selectedAccountIndex', selectedAccount.value)
  appStore.updateDownloadState('appId', appid.value)
  appStore.updateDownloadState('appVersionId', appVerId.value)
  appStore.updateDownloadState('availableVersions', versions.value)
  appStore.updateDownloadState('selectedVersionId', selectedVersion.value)
  appStore.updateDownloadState('versionsLoaded', versionsFetched.value)
  appStore.updateDownloadState('showProgressPanel', showProgress.value)
  appStore.updateDownloadState('progressPercentage', progressPercent.value)
  appStore.updateDownloadState('progressMessage', progressStage.value)
  appStore.updateDownloadState('progressLogs', logs.value)
}

const restoreStateFromStore = () => {
  const state = appStore.downloadState
  // Only restore non-undefined values to avoid overwriting auto-selected account
  if (state.selectedAccountIndex !== undefined && state.selectedAccountIndex !== null && state.selectedAccountIndex !== '') {
    selectedAccount.value = state.selectedAccountIndex
  }
  if (state.appId !== undefined) appid.value = state.appId
  if (state.appVersionId !== undefined) appVerId.value = state.appVersionId
  if (state.availableVersions !== undefined) versions.value = state.availableVersions
  if (state.selectedVersionId !== undefined) selectedVersion.value = state.selectedVersionId
  if (state.versionsLoaded !== undefined) versionsFetched.value = state.versionsLoaded
  if (state.showProgressPanel !== undefined) showProgress.value = state.showProgressPanel
  if (state.progressPercentage !== undefined) progressPercent.value = state.progressPercentage
  if (state.progressMessage !== undefined) progressStage.value = state.progressMessage
  if (state.progressLogs !== undefined) logs.value = state.progressLogs
}

// Watch state changes and sync to store
watch([selectedAccount, appid, appVerId, versions, selectedVersion, versionsFetched, showProgress, progressPercent, progressStage, logs], () => {
  syncStateToStore()
}, { deep: true })

// Watch account list and auto-select account
watch(accounts, () => {
  autoSelectFirstAccount()
}, { deep: true })

const API_BASE = '/api'

const loadAccounts = async () => {
  const saved = localStorage.getItem('ipa_accounts')
  if (saved) {
    try {
      accounts.value = dedupeAccounts(JSON.parse(saved))
    } catch {
      accounts.value = []
    }
  }
  
  // Fetch latest account list from server
  try {
    const response = await fetch(`${API_BASE}/accounts`, { credentials: 'include' })
    const data = await response.json()
    
    if (data.ok && data.data) {
      accounts.value = dedupeAccounts(data.data.map(acc => ({
        token: acc.token,
        email: acc.email,
        dsid: acc.dsid,
        region: acc.region || 'US',
        hasSavedCredentials: !!acc.hasSavedCredentials,
      })))
      // Update local storage
      localStorage.setItem('ipa_accounts', JSON.stringify(accounts.value))
      
      // Auto-select first account
      autoSelectFirstAccount()
    } else if (data.ok && (!data.data || data.data.length === 0)) {
      // No accounts on server, try auto-restore with saved credentials
      try {
        const autoRes = await fetch(`${API_BASE}/auto-login`, { method: 'POST', credentials: 'include' })
        const autoData = await autoRes.json()
        if (autoData.ok && autoData.data?.succeeded?.length > 0) {
          // Auto-login succeeded, reload account list
          const retryRes = await fetch(`${API_BASE}/accounts`, { credentials: 'include' })
          const retryData = await retryRes.json()
          if (retryData.ok && retryData.data) {
            accounts.value = dedupeAccounts(retryData.data.map(acc => ({
              token: acc.token,
              email: acc.email,
              dsid: acc.dsid,
              region: acc.region || 'US',
              hasSavedCredentials: !!acc.hasSavedCredentials,
            })))
            localStorage.setItem('ipa_accounts', JSON.stringify(accounts.value))
            autoSelectFirstAccount()
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

const resolveActiveAccount = async () => {
  if (!selectedAccount.value && selectedAccount.value !== 0) {
    throw new Error('Please select an account')
  }

  const currentAccount = accounts.value[selectedAccount.value]
  if (!currentAccount) {
    throw new Error('Account not found, please re-select an account')
  }

  const targetEmail = currentAccount.email
  await loadAccounts()

  const freshIndex = accounts.value.findIndex(
    acc => accountIdentityKey(acc) === accountIdentityKey(currentAccount) || acc.token === currentAccount.token || acc.email === targetEmail
  )

  if (freshIndex < 0) {
    throw new Error('Account session has expired, please sign in again in the Account settings')
  }

  selectedAccount.value = freshIndex
  return accounts.value[freshIndex]
}

const addLog = (message) => {
  const timestamp = new Date().toLocaleTimeString()
  logs.value += `[${timestamp}] ${message}\n`
}

// Navigate to account tab
const goToAccountTab = () => {
  const appStore = useAppStore()
  appStore.activeTab = 'settings'
}

// Search functionality - uses the region of the selected account
const handleSearch = useDebounceFn(async () => {
  const query = searchQuery.value.trim()
  if (!query) {
    searchResults.value = []
    return
  }

  // In direct App ID mode, don't search automatically
  if (searchMode.value === 'appid') {
    return
  }

  // Check if an account is selected
  if (accounts.value.length === 0 || selectedAccount.value === '' || selectedAccount.value === null) {
    searchResults.value = []
    return
  }

  searching.value = true
  try {
    // Get region of the currently selected account
    const account = accounts.value[selectedAccount.value]
    const region = account?.region || 'US'
    
    // Check if it's a numeric App ID
    if (/^\d+$/.test(query)) {
      // Direct App ID lookup
      const response = await fetch(`${API_BASE}/app-meta?appid=${encodeURIComponent(query)}&region=${encodeURIComponent(region)}`, { credentials: 'include' })
      const data = await response.json()

      if (data.ok && data.data) {
        searchResults.value = [data.data]
      } else {
        searchResults.value = []
      }
    } else {
      // Search by name or bundle ID
      const response = await fetch(`${API_BASE}/search?term=${encodeURIComponent(query)}&region=${encodeURIComponent(region)}&media=software&limit=10`, { credentials: 'include' })
      const data = await response.json()

      if (data.ok) {
        searchResults.value = data.data || []
      } else {
        searchResults.value = []
      }
    }
  } catch (error) {
    console.error('Search failed:', error)
    searchResults.value = []
  } finally {
    searching.value = false
  }
}, 300)

const selectApp = (app) => {
  emit('app-selected', app)
  searchQuery.value = ''
  searchResults.value = []
}

const confirmDirectAppId = () => {
  const appId = searchQuery.value.trim()
  if (/^\d+$/.test(appId)) {
    // Create a minimal app object with just the App ID
    const minimalApp = {
      trackId: appId,
      trackName: `App ID: ${appId}`,
      artistName: 'Unknown Developer',
      bundleId: 'unknown.bundle',
      artworkUrl60: null,
      artworkUrl100: null,
      version: 'Unknown',
      isDirectAppId: true // Flag to indicate this is a direct App ID input
    }
    emit('app-selected', minimalApp)
    searchQuery.value = ''
    searchResults.value = []
  }
}

// Watch for search mode changes
watch(searchMode, () => {
  searchQuery.value = ''
  searchResults.value = []
})

// Watch for selectedApp changes to auto-fill appid
watch(() => props.selectedApp, (newApp) => {
  if (newApp && newApp.trackId) {
    appid.value = String(newApp.trackId)
  }
}, { immediate: true })

// Watch for accounts changes to auto-select first account
watch(accounts, () => {
  autoSelectFirstAccount()
}, { deep: true, immediate: true })

// Watch for account and appid changes to auto-fetch versions
watch([selectedAccount, appid], ([newAccount, newAppid]) => {
  if (newAccount !== '' && newAccount !== null && newAppid) {
    // Auto-fetch versions
    fetchVersions()
  }
})

watch(
  [() => props.selectedApp?.trackId, selectedAccount],
  async ([trackId, accountIndex]) => {
    if (!trackId) {
      purchaseStatusText.value = 'Pending'
      return
    }
    if (accountIndex === '' || accountIndex === null || accountIndex === undefined) {
      purchaseStatusText.value = 'Select an account to check'
      return
    }
    await refreshSelectedAppMetadata()
    await refreshPurchaseStatus()
  },
  { immediate: true }
)

const fetchVersions = async () => {
  if (!appid.value) {
    ElMessage.warning('Please enter an App ID')
    return
  }

  if (selectedAccount.value === '' || selectedAccount.value === null) {
    ElMessage.warning('Please select an account first')
    return
  }

  const account = accounts.value[selectedAccount.value]
  const region = account?.region || 'US'

  fetchingVersions.value = true
  addLog(`[Lookup] Fetching version history for App ID=${appid.value} (region: ${getRegionLabel(region)})...`)

  try {
    const response = await fetch(`${API_BASE}/versions?appid=${encodeURIComponent(appid.value)}&region=${encodeURIComponent(region)}`, { credentials: 'include' })
    const data = await response.json()

    if (!data.ok) {
      ElMessage.error(`Lookup failed: ${data.error || 'Unknown error'}`)
      addLog(`[Lookup] Failed: ${data.error || 'Unknown error'}`)
      return
    }

    versions.value = [...(data.data || [])].sort(compareVersionDesc)
    versionsFetched.value = true
    addLog(`[Lookup] Retrieved ${versions.value.length} version record(s)`)
  } catch (error) {
    ElMessage.error(`Lookup failed: ${error.message}`)
    addLog(`[Lookup] Failed: ${error.message}`)
  } finally {
    fetchingVersions.value = false
  }
}

const handleVersionChange = () => {
  appVerId.value = selectedVersion.value || ''
}

const getSelectedAppPrice = () => {
  const price = Number(props.selectedApp?.price)
  return Number.isFinite(price) ? price : null
}

const getSelectedAppPriceLabel = () => {
  const formatted = props.selectedApp?.formattedPrice
  if (formatted && formatted !== '0' && formatted !== '0.00') return formatted
  const price = getSelectedAppPrice()
  if (price === null) return 'Unknown'
  if (price <= 0) return 'Free'
  return `${price}`
}

const getSelectedAppSizeLabel = () => {
  const size = Number(props.selectedApp?.fileSizeBytes)
  if (!Number.isFinite(size) || size <= 0) return 'Unknown'
  return `${(size / 1024 / 1024).toFixed(size / 1024 / 1024 >= 100 ? 0 : 1)} M`
}

const getPurchaseBehaviorLabel = () => {
  if (checkingPurchaseStatus.value) return 'Checking...'
  return purchaseStatusText.value
}

const compareVersionDesc = (a, b) => {
  const normalize = (value) => String(value || '')
    .split(/[^0-9A-Za-z]+/)
    .filter(Boolean)
    .map(part => (/^\d+$/.test(part) ? Number(part) : part.toLowerCase()))

  const av = normalize(a?.bundle_version)
  const bv = normalize(b?.bundle_version)
  const len = Math.max(av.length, bv.length)

  for (let i = 0; i < len; i += 1) {
    const left = av[i]
    const right = bv[i]
    if (left === undefined) return 1
    if (right === undefined) return -1
    if (left === right) continue
    if (typeof left === 'number' && typeof right === 'number') {
      return right - left
    }
    return String(right).localeCompare(String(left), undefined, { numeric: true, sensitivity: 'base' })
  }

  return String(b?.created_at || '').localeCompare(String(a?.created_at || ''))
}

const refreshSelectedAppMetadata = async () => {
  if (!props.selectedApp?.trackId) return

  const region = accounts.value[selectedAccount.value]?.region || 'US'
  const needsFill = !props.selectedApp?.formattedPrice || !props.selectedApp?.fileSizeBytes
  if (!needsFill) return

  try {
    const response = await fetch(`${API_BASE}/app-meta?appid=${encodeURIComponent(props.selectedApp.trackId)}&region=${encodeURIComponent(region)}`, {
      credentials: 'include'
    })
    const data = await response.json()
    const app = data?.data
    if (!data.ok || !app) return

    appStore.setSelectedApp({
      ...props.selectedApp,
      ...app,
    })
  } catch (error) {
    console.warn('Failed to enrich selected app metadata:', error)
  }
}

const refreshPurchaseStatus = async () => {
  if (!props.selectedApp?.trackId) {
    purchaseStatusText.value = 'Pending'
    return
  }

  if (selectedAccount.value === null || selectedAccount.value === undefined || selectedAccount.value === '') {
    purchaseStatusText.value = 'Select an account to check'
    return
  }

  const account = accounts.value[selectedAccount.value]
  if (!account?.token) {
    purchaseStatusText.value = 'Invalid account'
    return
  }

  checkingPurchaseStatus.value = true
  try {
    const response = await fetch(`${API_BASE}/purchase-status?token=${encodeURIComponent(account.token)}&appid=${encodeURIComponent(props.selectedApp.trackId)}${appVerId.value ? `&appVerId=${encodeURIComponent(appVerId.value)}` : ''}`, {
      credentials: 'include'
    })
    const data = await response.json()
    const payload = data?.data || {}

    if (!data.ok) throw new Error(data.error || 'Check failed')

    purchaseStatus.value = {
      purchased: !!payload.purchased,
      needsPurchase: !!payload.needsPurchase,
      status: payload.status || 'unknown',
      error: payload.error || null
    }

    const price = getSelectedAppPrice()
    if (payload.purchased) {
      purchaseStatusText.value = price !== null && price > 0 ? 'Purchased' : 'Claimed'
    } else if (payload.needsPurchase) {
      purchaseStatusText.value = price !== null && price > 0 ? 'Not purchased' : 'Not claimed'
    } else {
      purchaseStatusText.value = payload.error ? `Check failed: ${payload.error}` : 'Unknown status'
    }
  } catch (error) {
    purchaseStatusText.value = 'Check failed'
    console.warn('Failed to refresh purchase status:', error)
  } finally {
    checkingPurchaseStatus.value = false
  }
}

const preflightPurchaseGate = async (account, modeLabel, retryFn) => {
  if (!account?.token || !props.selectedApp?.trackId) return true

  checkingPurchaseStatus.value = true
  try {
    const response = await fetch(`${API_BASE}/purchase-status?token=${encodeURIComponent(account.token)}&appid=${encodeURIComponent(props.selectedApp.trackId)}${appVerId.value ? `&appVerId=${encodeURIComponent(appVerId.value)}` : ''}`, {
      credentials: 'include'
    })
    const data = await response.json()
    const payload = data?.data || {}

    if (!data.ok) {
      throw new Error(data.error || 'Failed to check purchase status')
    }

    const price = getSelectedAppPrice()
    purchaseStatus.value = {
      purchased: !!payload.purchased,
      needsPurchase: !!payload.needsPurchase,
      status: payload.status || 'unknown',
      error: payload.error || null
    }

    if (payload.purchased) {
      purchaseStatusText.value = price !== null && price > 0 ? 'Purchased' : 'Claimed'
      return true
    }

    if (payload.needsPurchase) {
      purchaseStatusText.value = price !== null && price > 0 ? 'Not purchased' : 'Not claimed'
      ElMessage.warning(downloadBlockedReason.value || 'Not purchased/claimed by current account')
      return false
    }

    purchaseStatusText.value = payload.error ? `Check failed: ${payload.error}` : 'Unknown status'
    await ElMessageBox.alert(
      `Pre-download purchase check failed: ${payload.error || 'Unknown status'}. Download aborted to prevent errors.`,
      'Cannot Start Download',
      {
        confirmButtonText: 'OK',
        type: 'warning'
      }
    )
    return false
  } catch (error) {
    purchaseStatusText.value = 'Check failed'
    await ElMessageBox.alert(
      `Pre-download purchase check failed: ${error.message || error}`,
      'Cannot Start Download',
      {
        confirmButtonText: 'OK',
        type: 'warning'
      }
    )
    return false
  } finally {
    checkingPurchaseStatus.value = false
  }
}

const buyOrClaimSelectedApp = async () => {
  try {
    const account = await resolveActiveAccount()
    const price = getSelectedAppPrice()

    if (price === null) {
      await ElMessageBox.alert('Price is unknown, cannot safely claim/purchase. Please confirm the price in search results first.', 'Cannot Claim', {
        confirmButtonText: 'OK',
        type: 'warning'
      })
      return
    }

    if (price > 0) {
      await ElMessageBox.alert('This is a paid app. Please purchase it in the App Store first. The download button will be available after purchase.', 'Purchase Required', {
        confirmButtonText: 'OK',
        type: 'warning'
      })
      return
    }

    const appName = props.selectedApp?.trackName || appid.value || 'Current App'
    const appStoreUrl = props.selectedApp?.trackViewUrl || `https://apps.apple.com/app/id${props.selectedApp.trackId}`
    await ElMessageBox.alert(
      `Free app "${appName}": Please tap "Get" in the App Store first. Come back here to refresh status, then choose "Direct Link Download" or "Download to Server".`,
      'Get App from App Store First',
      {
        confirmButtonText: 'Open App Store',
        type: 'info'
      }
    ).catch(() => {})

    window.open(appStoreUrl, '_blank', 'noopener')
    await refreshPurchaseStatus()
  } catch (error) {
    ElMessage.warning(error.message || 'Claim failed')
  }
}

const handleNeedsPurchase = async (retryFn, modeLabel, account = null) => {
  const price = getSelectedAppPrice()
  const appName = props.selectedApp?.trackName || appid.value || 'Current App'
  const accountEmail = account?.email || accounts.value[selectedAccount.value]?.email || 'Unknown Account'
  const accountRegion = getRegionLabel(account?.region || accounts.value[selectedAccount.value]?.region || 'US')

  if (price === null) {
    await ElMessageBox.alert(
      `App: ${appName}\nPrice: Unknown\nAccount: ${accountEmail}\nRegion: ${accountRegion}\n\nUnable to confirm if this app is free. To avoid accidental paid purchases, please check the price in search results first; if it is a paid app, complete the purchase in the App Store.`,
      'Cannot Auto-Purchase',
      {
        confirmButtonText: 'OK',
        type: 'warning'
      }
    )
    addLog(`[${modeLabel}] Not purchased, price unknown, blocked auto-purchase`)
    return
  }

  if (price > 0) {
    await ElMessageBox.alert(
      `App: ${appName}\nPrice: ${getSelectedAppPriceLabel()}\nAccount: ${accountEmail}\nRegion: ${accountRegion}\n\nThis is a paid app. Auto-purchase is not supported. Please complete the purchase in the App Store and come back to download.`,
      'Cannot Auto-Purchase Paid App',
      {
        confirmButtonText: 'OK',
        type: 'warning'
      }
    )
    addLog(`[${modeLabel}] Paid app not purchased, user prompted to buy in App Store`)
    return
  }

  const confirmed = await ElMessageBox.confirm(
    `App: ${appName}\nPrice: Free\nAccount: ${accountEmail}\nRegion: ${accountRegion}\n\nThis is a free app, but the current account has not claimed it yet. Would you like to trigger a purchase (claim) and continue downloading?`,
    'Free App Needs to Be Claimed First',
    {
      confirmButtonText: 'Claim & Download',
      cancelButtonText: 'Cancel',
      type: 'warning'
    }
  ).then(() => true).catch(() => false)

  if (confirmed) {
    addLog(`[${modeLabel}] Free app not claimed, user confirmed to trigger purchase`)
    return retryFn(true)
  }

  addLog(`[${modeLabel}] User cancelled free app claim`)
}

const directLinkDownload = async (autoPurchase = false) => {
  if (!selectedAccount.value && selectedAccount.value !== 0) {
    ElMessage.warning('Please select an account')
    return
  }
  if (!appid.value) {
    ElMessage.warning('Please enter an App ID')
    return
  }

  try {
    const account = await resolveActiveAccount()
    if (!autoPurchase) {
      const allowed = await preflightPurchaseGate(account, 'Direct', directLinkDownload)
      if (!allowed) return
    }
    addLog('[Direct] Fetching direct link…')
    const url = `${API_BASE}/download-url?token=${encodeURIComponent(account.token)}&appid=${encodeURIComponent(appid.value)}${appVerId.value ? `&appVerId=${encodeURIComponent(appVerId.value)}` : ''}${autoPurchase ? '&autoPurchase=true' : ''}`
    const response = await fetch(url, { credentials: 'include' })
    const data = await response.json()
    const payload = data?.data || data

    if (!data.ok) {
      if (data.needsPurchase && !autoPurchase) {
        ElMessage.warning(downloadBlockedReason.value || 'Not purchased/claimed by current account')
        return
      }
      ElMessage.error(`Direct link failed: ${data.error || 'Unknown error'}`)
      addLog(`[Direct] Failed: ${data.error || 'Unknown error'}`)
      return
    }

    addLog(`[Direct] Success: filename=${payload.fileName}, downloading directly from Apple CDN`)
    addLog(`[Direct] URL (partial)=${String(payload.url).slice(0, 80)}...`)

    // Trigger browser download
    const a = document.createElement('a')
    a.href = payload.url
    a.download = payload.fileName || ''
    a.rel = 'noopener'
    document.body.appendChild(a)
    a.click()
    a.remove()
  } catch (error) {
    ElMessage.error(`Direct link failed: ${error.message}`)
    addLog(`[Direct] Failed: ${error.message}`)
  }
}

const startDownloadWithProgress = async (autoPurchase = false) => {
  if (!selectedAccount.value && selectedAccount.value !== 0) {
    ElMessage.warning('Please select an account')
    return
  }
  if (!appid.value) {
    ElMessage.warning('Please enter an App ID')
    return
  }

  try {
    const account = await resolveActiveAccount()
    if (!autoPurchase) {
      const allowed = await preflightPurchaseGate(account, 'Progress', startDownloadWithProgress)
      if (!allowed) return
    }

    // Reset progress
    showProgress.value = true
    progressPercent.value = 0
    progressStage.value = 'Preparing…'
    logs.value = ''
    downloadReadyUrl.value = ''
    downloadReadyFileSize.value = 0
    downloadInstallUrl.value = ''
    downloadPackageKind.value = ''
    downloadOtaInstallable.value = false
    downloadInstallMethod.value = ''
    downloadInspection.value = null
    showActionButtons.value = false
    addLog('[Progress] Creating download task…')

    addLog(`[Progress] Using account ${account.email}, token=${String(account.token).slice(0, 8)}…`)
    const response = await fetch(`${API_BASE}/start-download-direct`, {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        token: account.token,
        appid: appid.value,
        appVerId: appVerId.value ? String(appVerId.value) : undefined,
        autoPurchase: !!autoPurchase,
        appName: props.selectedApp?.trackName || undefined,
        bundleId: props.selectedApp?.bundleId || undefined,
        appVersion: props.selectedApp?.version || undefined,
        artworkUrl: props.selectedApp?.artworkUrl100 || props.selectedApp?.artworkUrl60 || undefined,
        artistName: props.selectedApp?.artistName || undefined
      })
    })
    const data = await response.json()

    if (!data.ok) {
      if (data.needsPurchase && !autoPurchase) {
        showProgress.value = false
        ElMessage.warning(downloadBlockedReason.value || 'Not purchased/claimed by current account')
        return
      }
      ElMessage.error(`Failed to create task: ${data.error || 'Unknown error'}`)
      addLog(`[Progress] Failed to create task: ${data.error || 'Unknown error'}`)
      return
    }

    const { jobId } = data
    addLog(`[Progress] Task created: ${jobId}`)

    // Add to queue — flatten selectedApp's icon/name/developer to top level for DownloadQueue rendering
    const app = props.selectedApp || {}
    const queueItem = {
      id: jobId,
      appName: app.trackName || appid.value,
      artworkUrl: app.artworkUrl100 || app.artworkUrl60 || '',
      artistName: app.artistName || '',
      version: app.version || '',
      app: app,
      account: account,
      accountEmail: account.email || '',
      status: 'downloading',
      progress: 0,
      logs: logs.value,
      timestamp: new Date().toISOString()
    }
    emit('download-started', queueItem)

    // Connect to SSE / fallback polling
    connectToSSE(jobId, queueItem)
  } catch (error) {
    ElMessage.error(`Failed to create task: ${error.message}`)
    addLog(`[Progress] Failed to create task: ${error.message}`)
  }
}

const pollJobStatus = (jobId, queueItem) => {
  addLog('[Progress] SSE unavailable, switching to polling mode')

  const markInterrupted = (message = 'Task is no longer valid, possibly due to server restart or page navigation. Please re-initiate the download.') => {
    clearInterval(timer)
    addLog(`[Failed] ${message}`)
    const appStore = useAppStore()
    appStore.updateQueueItem(jobId, {
      status: 'failed',
      stage: 'interrupted',
      error: message
    })
    if (queueItem) {
      queueItem.status = 'failed'
      queueItem.error = message
    }
  }

  const timer = setInterval(async () => {
    try {
      const response = await fetch(`${API_BASE}/job-info?jobId=${encodeURIComponent(jobId)}`, { credentials: 'include' })
      const data = await response.json()
      if (response.status === 404) {
        markInterrupted(data?.error || 'Task no longer exists')
        return
      }
      if (!response.ok || !data.ok || !data.data) return

      const snapshot = data.data
      if (snapshot.progress != null) {
        progressPercent.value = snapshot.progress
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { progress: snapshot.progress })
      }
      if (snapshot.stage) {
        progressStage.value = snapshot.stage
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { stage: snapshot.stage })
      }
      if (snapshot.error) {
        addLog(`[Error] ${snapshot.error}`)
      }

      if (snapshot.status === 'ready') {
        clearInterval(timer)
        progressPercent.value = 100
        progressStage.value = 'Download complete'
        if (snapshot.installMethod === 'download_only') {
          addLog('[Progress] File saved to server, download export only')
        } else {
          addLog('[Progress] File saved to server, can be downloaded or installed manually')
        }

        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, {
          status: 'completed',
          progress: 100,
          downloadUrl: snapshot.downloadUrl,
          installUrl: snapshot.installUrl,
          fileSize: snapshot.fileSize || 0,
          packageKind: snapshot.packageKind,
          otaInstallable: snapshot.otaInstallable,
          installMethod: snapshot.installMethod,
          inspection: snapshot.inspection
        })

        downloadReadyUrl.value = snapshot.downloadUrl || ''
        downloadReadyFileSize.value = snapshot.fileSize || 0
        downloadInstallUrl.value = snapshot.installUrl || ''
        downloadPackageKind.value = snapshot.packageKind || ''
        downloadOtaInstallable.value = !!snapshot.otaInstallable
        downloadInstallMethod.value = snapshot.installMethod || ''
        downloadInspection.value = snapshot.inspection || null
        showActionButtons.value = !!(snapshot.downloadUrl || snapshot.installUrl)
      } else if (snapshot.status === 'failed') {
        clearInterval(timer)
        addLog(`[Failed] ${snapshot.error || 'Task failed'}`)
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, {
          status: 'failed',
          error: snapshot.error || 'Task failed'
        })
        if (queueItem) {
          queueItem.status = 'error'
          queueItem.error = snapshot.error || 'Task failed'
        }
      }
    } catch (error) {
      clearInterval(timer)
      addLog(`[Error] Failed to poll task status: ${error.message}`)
      const appStore = useAppStore()
      appStore.updateQueueItem(jobId, {
        status: 'failed',
        error: error.message
      })
      if (queueItem) {
        queueItem.status = 'error'
        queueItem.error = error.message
      }
    }
  }, 1500)
}

const connectToSSE = (jobId, queueItem) => {
  let es
  try {
    const origin = window.location.origin || `${window.location.protocol}//${window.location.host}`
    const sseUrl = new URL(`${API_BASE}/progress-sse?jobId=${encodeURIComponent(jobId)}`, origin).toString()
    es = new EventSource(sseUrl)
  } catch (error) {
    addLog(`[Progress] SSE initialization failed: ${error.message}`)
    pollJobStatus(jobId, queueItem)
    return
  }

  es.addEventListener('progress', (ev) => {
    try {
      const data = JSON.parse(ev.data)
      
      if (data?.progress?.percent != null) {
        progressPercent.value = data.progress.percent
        // Update queue item progress
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { progress: data.progress.percent })
      }
      
      if (data?.progress?.stage) {
        const stageMap = {
          'auth': 'Fetching download info',
          'download-start': 'Starting download',
          'download-progress': 'Downloading',
          'merge': 'Merging chunks',
          'sign': 'Writing signature',
          'done': 'Done'
        }
        progressStage.value = stageMap[data.progress.stage] || data.progress.stage
        // Update queue item status
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { stage: progressStage.value })
      }
      
      if (data?.error) {
        addLog(`[Error] ${data.error}`)
        const appName = props.selectedApp?.trackName || appid.value
        notifications.notifyDownloadFailed(appName, data.error)
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, {
          status: 'failed',
          error: data.error
        })
      }
      
      if (data.status === 'ready') {
        progressPercent.value = 100
        progressStage.value = 'Download complete'
        addLog('[Progress] File saved to server, refresh after task completes to get delivery info')

        // Update queue item status
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, {
          status: 'completed',
          progress: 100
        })
      }
    } catch (e) {
      console.error(e)
    }
  })

  es.addEventListener('log', (ev) => {
    try {
      const { line } = JSON.parse(ev.data)
      if (line) {
        addLog(line)
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { logs: logs.value })
      }
    } catch {}
  })

  es.addEventListener('end', (ev) => {
    try {
      const data = JSON.parse(ev.data || '{}')
      if (data.status === 'ready') {
        addLog('[Done] Task is ready')
        // Send download complete notification
        const appName = props.selectedApp?.trackName || appid.value
        notifications.notifyDownloadComplete(appName)
        // Fetch task info including install URL
        fetch(`${API_BASE}/job-info?jobId=${encodeURIComponent(jobId)}`, { credentials: 'include' })
          .then(res => res.json())
          .then(jobData => {
            if (jobData.ok && jobData.data) {
              downloadReadyUrl.value = jobData.data.downloadUrl || ''
              downloadReadyFileSize.value = jobData.data.fileSize || 0
              downloadInstallUrl.value = jobData.data.installUrl || ''
              downloadPackageKind.value = jobData.data.packageKind || ''
              downloadOtaInstallable.value = !!jobData.data.otaInstallable
              downloadInstallMethod.value = jobData.data.installMethod || ''
              downloadInspection.value = jobData.data.inspection || null
              showActionButtons.value = !!(jobData.data.downloadUrl || jobData.data.installUrl)

              if (jobData.data.otaInstallable && jobData.data.installUrl) {
                addLog('[Install] OTA install link generated')
              } else if (jobData.data.installMethod === 'download_only') {
                addLog('[Delivery] This package does not support OTA installation, download only')
              }

              const appStore = useAppStore()
              appStore.updateQueueItem(jobId, {
                status: 'completed',
                progress: 100,
                downloadUrl: jobData.data.downloadUrl,
                installUrl: jobData.data.installUrl,
                fileSize: jobData.data.fileSize || 0,
                packageKind: jobData.data.packageKind,
                otaInstallable: jobData.data.otaInstallable,
                installMethod: jobData.data.installMethod,
                inspection: jobData.data.inspection
              })
            }
          })
          .catch(() => {
            // Ignore errors
          })
      } else if (data.status === 'failed') {
        addLog('[Failed] Task failed')
        const appName = props.selectedApp?.trackName || appid.value
        notifications.notifyDownloadFailed(appName)
        if (queueItem) {
          queueItem.status = 'error'
        }
      } else {
        addLog(`[End] Task ended: ${data.status || 'unknown'}`)
      }
    } catch {}
    es.close()
  })

  es.onerror = () => {
    addLog('[Error] SSE connection lost, switching to polling mode')
    es.close()
    pollJobStatus(jobId, queueItem)
  }
}

// Watch account updates
watch(() => props.accountsUpdated, async () => {
  ElMessage.info('Account status change detected, refreshing accounts and purchase status…')
  await loadAccounts()
  await refreshSelectedAppMetadata()
  await refreshPurchaseStatus()
  if (appid.value && selectedAccount.value !== null && selectedAccount.value !== undefined && selectedAccount.value !== '') {
    await fetchVersions()
  }
  ElMessage.success('Account refreshed and page state synced')
})

const openInstallUrl = (url) => {
  if (!url) {
    ElMessage.warning('Install link not yet available')
    return
  }

  window.location.assign(url)
}

const installDownloadedIpa = async () => {
  if (!downloadInstallUrl.value) {
    ElMessage.warning('Install link not yet available')
    return
  }

  const isHttpsEnvironment = window.location.protocol === 'https:'
  const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'

  if (!isHttpsEnvironment && !isLocalhost) {
    const action = await ElMessageBox.confirm(
      'Per the OpenList/Oplist approach, OTA installation requires HTTPS + valid certificate. The current environment is not HTTPS, so iOS will not respond. You can download the IPA file directly or switch to an HTTPS domain and try again.',
      'Cannot Start OTA Installation',
      {
        distinguishCancelAndClose: true,
        confirmButtonText: 'Download File',
        cancelButtonText: 'Cancel',
        type: 'warning',
        center: true
      }
    ).then(
      () => 'download',
      () => 'cancel'
    ).catch(
      (action) => action === 'cancel' ? 'cancel' : 'close'
    )

    if (action === 'download') {
      downloadCompletedIpa()
    }
    return
  }

  if (isHttpsEnvironment) {
    ElMessage.success('Opening install link...')
    openInstallUrl(downloadInstallUrl.value)
  } else if (isLocalhost) {
    const confirmed = await ElMessageBox.confirm(
      'Current environment is localhost. Per OpenList/Oplist documentation, OTA installation requires HTTPS + valid certificate; localhost will likely not work. You can continue trying or switch to an HTTPS domain.',
      'Pre-Installation Check',
      {
        confirmButtonText: 'Try Anyway',
        cancelButtonText: 'Cancel',
        type: 'info'
      }
    ).then(() => true).catch(() => false)

    if (confirmed) {
      openInstallUrl(downloadInstallUrl.value)
    }
  }
}

const downloadCompletedIpa = () => {
  if (!downloadReadyUrl.value) {
    ElMessage.warning('Download link not yet available')
    return
  }

  window.open(downloadReadyUrl.value, '_blank', 'noopener')
}

const formatFileSize = (bytes) => {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let value = bytes
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return `${value.toFixed(value >= 100 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}

onMounted(() => {
  loadAccounts()
  restoreStateFromStore()
  
  // Detect current environment
  isHttps.value = window.location.protocol === 'https:'
  currentProtocol.value = window.location.protocol
  
  console.log(`[Environment] Protocol: ${currentProtocol.value}, HTTPS: ${isHttps.value}`)
})
</script>

<style scoped>
.download-disabled-hint {
  padding: 10px 12px;
  border-radius: 12px;
  font-size: 12px;
  line-height: 1.4;
  color: #92400e;
  background: rgba(254, 243, 199, 0.85);
  border: 1px solid rgba(245, 158, 11, 0.35);
}

.dark .download-disabled-hint {
  color: #fcd34d;
  background: rgba(120, 53, 15, 0.35);
  border-color: rgba(245, 158, 11, 0.35);
}

.purchase-blocked-btn.is-disabled {
  opacity: 0.62 !important;
  filter: grayscale(0.15);
  box-shadow: none !important;
}

.purchase-blocked-btn.is-disabled :deep(span),
.purchase-blocked-btn.is-disabled :deep(i),
.purchase-blocked-btn.is-disabled :deep(svg) {
  opacity: 1 !important;
}

.selected-app-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.selected-app-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: 9999px;
  font-size: 12px;
  line-height: 1;
  color: #1d4ed8;
  background: rgba(219, 234, 254, 0.9);
}

.dark .selected-app-badge {
  color: #bfdbfe;
  background: rgba(30, 58, 138, 0.5);
}

.search-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  padding: 8px 16px;
}

.search-input :deep(.el-input__inner) {
  font-size: 15px;
}

/* Quick account selector styles */
.account-quick-select {
  width: 320px;
  max-width: 100%;
}

.account-quick-select :deep(.el-select__wrapper) {
  border-radius: 10px;
  font-size: 13px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.account-quick-select :deep(.el-select__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.dark .account-quick-select :deep(.el-select__wrapper) {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.dark .account-quick-select :deep(.el-select__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
}

.account-quick-select :deep(.el-select__placeholder) {
  font-size: 12px;
  color: #9ca3af;
}

.account-quick-select :deep(.el-select__input) {
  font-size: 13px;
}

.account-quick-select :deep(.el-select-dropdown__item) {
  overflow: hidden;
}

.account-option-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-width: 0;
}

.account-option-email {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Mini region badge */
.region-badge-mini {
  display: inline-flex;
  flex-shrink: 0;
  max-width: 110px;
  height: 20px;
  line-height: 1;
  align-items: center;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.3px;
  white-space: nowrap;
}

.form-select :deep(.el-select__wrapper) {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.form-select :deep(.el-select__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.dark .form-select :deep(.el-select__wrapper) {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.dark .form-select :deep(.el-select__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
}

.form-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.form-input :deep(.el-input__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.dark .form-input :deep(.el-input__wrapper) {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.dark .form-input :deep(.el-input__wrapper:hover) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
}

.action-button {
  border-radius: 12px;
  font-weight: 500;
  height: 44px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.action-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.action-button:active {
  transform: translateY(0);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.action-button :deep(.el-icon) {
  font-size: 18px;
}

.log-container {
  background-color: rgba(0, 0, 0, 0.03);
  border-radius: 12px;
  padding: 16px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  max-height: 300px;
  overflow-y: auto;
}

.dark .log-container {
  background-color: rgba(0, 0, 0, 0.3);
}

.log-entry {
  padding: 4px 0;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.dark .log-entry {
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.log-entry:last-child {
  border-bottom: none;
}

.log-time {
  color: #9ca3af;
  font-size: 12px;
}

.dark .log-time {
  color: #6b7280;
}

.log-content {
  color: #374151;
}

.dark .log-content {
  color: #d1d5db;
}

.log-success {
  color: #059669;
}

.dark .log-success {
  color: #34d399;
}

.log-error {
  color: #dc2626;
}

.dark .log-error {
  color: #f87171;
}

/* Region badge styles */
.region-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.region-us {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15) 0%, rgba(37, 99, 235, 0.15) 100%);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.dark .region-us {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.25) 0%, rgba(37, 99, 235, 0.25) 100%);
  color: #60a5fa;
  border-color: rgba(59, 130, 246, 0.4);
}

.region-cn {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(220, 38, 38, 0.15) 100%);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.dark .region-cn {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.25) 0%, rgba(220, 38, 38, 0.25) 100%);
  color: #f87171;
  border-color: rgba(239, 68, 68, 0.4);
}

.region-jp {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15) 0%, rgba(5, 150, 105, 0.15) 100%);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.dark .region-jp {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.25) 0%, rgba(5, 150, 105, 0.25) 100%);
  color: #34d399;
  border-color: rgba(16, 185, 129, 0.4);
}

.region-gb,
.region-de,
.region-fr,
.region-ca,
.region-au {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.15) 0%, rgba(124, 58, 237, 0.15) 100%);
  color: #8b5cf6;
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.dark .region-gb,
.dark .region-de,
.dark .region-fr,
.dark .region-ca,
.dark .region-au {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.25) 0%, rgba(124, 58, 237, 0.25) 100%);
  color: #a78bfa;
  border-color: rgba(139, 92, 246, 0.4);
}

.log-info {
  color: #2563eb;
}

.dark .log-info {
  color: #60a5fa;
}

/* Mobile responsive styles */
@media (max-width: 767px) {
  .card {
    padding: 12px;
  }
  
  .action-button {
    height: 48px;
    font-size: 15px;
  }
  
  /* Mobile account selector */
  .account-quick-select {
    width: 100%;
    margin-top: 12px;
  }
  
  /* Mobile search area notice */
  .bg-blue-50.dark\:bg-blue-900\/20 {
    flex-direction: column;
    align-items: flex-start !important;
  }
  
  .bg-blue-50.dark\:bg-blue-900\/20 .flex {
    flex-direction: column;
    width: 100%;
  }
  
  /* Search result card adaptive */
  .search-result-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
  }
  
  .search-result-item img {
    width: 48px !important;
    height: 48px !important;
  }
  
  .search-result-item h3 {
    font-size: 13px !important;
    max-width: calc(100vw - 140px);
  }
  
  /* Selected app info card */
  .selected-app-card {
    padding: 12px !important;
  }
  
  .selected-app-card img {
    width: 48px !important;
    height: 48px !important;
  }
  
  .selected-app-card h3 {
    font-size: 14px !important;
    word-break: break-word;
    overflow-wrap: break-word;
  }
  
  /* Upload area */
  .upload-demo :deep(.el-upload-dragger) {
    padding: 20px !important;
  }
  
  /* Progress card */
  .el-card {
    margin-top: 12px !important;
  }
}
</style>
