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
          下载与签名
        </h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          搜索应用、查询版本并下载IPA文件
        </p>
      </div>
    </div>

    <!-- Search Section -->
    <div class="space-y-4 mb-6">
      <!-- 账号选择提示 -->
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
              需要先登录账号
            </h4>
            <p class="text-sm text-orange-700 dark:text-orange-400 mt-1">
              请先在"账号"标签页登录 Apple ID 账号，然后才能搜索应用。
            </p>
            <el-button 
              type="warning" 
              size="small" 
              class="mt-2" 
              plain
              @click="goToAccountTab"
            >
              前往登录
            </el-button>
          </div>
        </div>
      </div>

      <!-- 账号选择区域 -->
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
              搜索区域: <strong>{{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }}</strong>
            </span>
          </div>
          <el-select 
            v-model="selectedAccount"
            placeholder="选择账号"
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
              <div class="flex items-center justify-between w-full">
                <span class="flex-1 truncate">{{ account.email }}</span>
                <span
                  class="region-badge-mini ml-2"
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
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">搜索应用</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer">
          <input
            v-model="searchMode"
            type="radio"
            value="appid"
            class="w-4 h-4 text-primary-600 focus:ring-primary-500 border-gray-300"
          >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">直接输入 App ID</span>
        </label>
      </div>

      <el-input
        v-model="searchQuery"
        :placeholder="searchMode === 'search' ? '搜索应用名称、Bundle ID 或 App ID...' : '输入 App ID（纯数字）...'"
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
            即使未找到应用信息，也可以继续查询版本号
          </p>
        </div>
        <el-button
          type="primary"
          size="default"
          @click="confirmDirectAppId"
        >
          确认并继续
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
              版本: {{ selectedApp.version }} | ID: {{ selectedApp.trackId }}
              <span
                v-if="selectedApp.isDirectAppId"
                class="ml-2 px-2 py-0.5 bg-yellow-200 dark:bg-yellow-800 text-yellow-800 dark:text-yellow-200 rounded-full text-xs"
              >
                直接输入
              </span>
            </p>
          </div>
        </div>
      </div>

      <!-- Download Options -->
      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            选择账号
            <span
              v-if="selectedAccount !== null && selectedAccount !== undefined && selectedAccount !== ''"
              class="ml-2 text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded-full"
            >
              商店区域: {{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }}
            </span>
          </label>
          <el-select 
            v-model="selectedAccount"
            placeholder="请先登录账号"
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
            ⚠️ 请先登录账号
          </p>
          <p
            v-else
            class="text-xs text-gray-500 dark:text-gray-400 mt-1"
          >
            ✅ 搜索和下载将使用此账号的 {{ getRegionLabel(accounts[selectedAccount]?.region || 'US') }} 商店
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">APPID</label>
          <el-input
            v-model="appid"
            placeholder="例如：1160172628"
            class="form-input"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">版本（历史版本下拉）</label>
          <el-select 
            v-model="selectedVersion"
            placeholder="请先查询版本"
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
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">appVerId（自动填充）</label>
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
            查询版本
          </el-button>

          <el-button
            :disabled="!selectedAccount && selectedAccount !== 0"
            type="info"
            class="w-full action-button"
            @click="directLinkDownload"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            直链下载（仅下载文件）
          </el-button>

          <el-button
            :disabled="!selectedAccount && selectedAccount !== 0"
            :loading="downloading"
            type="primary"
            class="w-full action-button"
            @click="startDownloadWithProgress"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            {{ downloading ? '处理中...' : '下载并自动安装' }}
          </el-button>

          <el-button
            :disabled="!canAddToBatch"
            type="success"
            plain
            class="w-full action-button"
            @click="addCurrentSelectionToBatch"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            添加到批量下载
          </el-button>
        </el-space>
      </div>

      <!-- Upload IPA Section -->
      <div class="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          上传 IPA 文件
        </h3>
        <el-upload
          ref="uploadRef"
          class="upload-demo"
          :action="uploadUrl"
          :on-success="handleUploadSuccess"
          :on-error="handleUploadError"
          :on-progress="handleUploadProgress"
          :before-upload="beforeUpload"
          :show-file-list="false"
          accept=".ipa"
          :auto-upload="true"
          drag
        >
          <div class="el-upload__text">
            <el-icon class="el-icon--upload">
              <upload-filled />
            </el-icon>
            <div class="text-sm text-gray-600 dark:text-gray-400 mt-2">
              将 IPA 文件拖到此处，或<em>点击上传</em>
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-500 mt-1">
              支持 .ipa 格式，最大 2GB
            </div>
          </div>
        </el-upload>

        <!-- Upload Result -->
        <div
          v-if="uploadResult.jobId"
          class="mt-4 p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl"
        >
          <div class="flex items-start space-x-3">
            <svg
              class="w-5 h-5 text-green-600 dark:text-green-400 mt-0.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div class="flex-1">
              <h4 class="font-semibold text-green-900 dark:text-green-300">
                上传成功
              </h4>
              <p class="text-sm text-green-700 dark:text-green-400 mt-1">
                文件：{{ uploadResult.fileName }}
              </p>
              
              <!-- Environment Warning for Upload -->
              <div
                v-if="!isHttps && currentProtocol !== 'http:'"
                class="mt-2 p-2 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg"
              >
                <p class="text-xs text-yellow-800 dark:text-yellow-300">
                  ⚠️ 当前非 HTTPS 环境，iOS 设备可能无法安装
                </p>
              </div>
              
              <el-button 
                type="success" 
                size="small" 
                class="mt-2" 
                plain
                @click="installUploadedIpa"
              >
                点击安装
              </el-button>
            </div>
          </div>
        </div>

        <!-- Upload Progress -->
        <div
          v-if="uploading"
          class="mt-4"
        >
          <el-progress
            :percentage="uploadProgress"
            :stroke-width="10"
          />
          <p class="text-sm text-gray-600 dark:text-gray-400 mt-2">
            正在上传...
          </p>
        </div>
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
        
        <!-- Install Button -->
        <div
          v-if="showInstallButton && downloadInstallUrl"
          class="mt-4"
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
                  环境检测
                </p>
                <p class="text-xs text-yellow-700 dark:text-yellow-400 mt-1">
                  当前协议: {{ currentProtocol || '未知' }} | iOS 安装需要 HTTPS 环境
                </p>
              </div>
            </div>
          </div>
          
          <el-button 
            type="success" 
            size="large" 
            class="w-full"
            @click="installDownloadedIpa"
          >
            <template #icon>
              <el-icon><Download /></el-icon>
            </template>
            点击安装到设备
          </el-button>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-2 text-center">
            请在 iOS 设备的 Safari 中打开此页面并点击安装
          </p>
          <p
            v-if="!isHttps"
            class="text-xs text-orange-600 dark:text-orange-400 mt-1 text-center"
          >
            ⚠️ 非 HTTPS 环境可能无法安装，点击按钮查看选项
          </p>
        </div>
      </el-card>
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
          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
        />
      </svg>
      <p class="text-lg font-medium">
        未选择应用
      </p>
      <p class="text-sm mt-2">
        请先在上方搜索并选择一个应用
      </p>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, onMounted, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { useAppStore } from '../stores/app'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, ArrowRight, Download, UploadFilled } from '@element-plus/icons-vue'

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

const emit = defineEmits(['download-started', 'app-selected'])

// 获取区域标签
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

// 处理账号选择变化
const handleAccountChange = () => {
  const account = accounts.value[selectedAccount.value]
  if (account) {
    console.log(`[DownloadManager] Selected account: ${account.email}, Region: ${account.region || 'US'}`)
  }
  
  // 清空之前查询的版本信息
  versions.value = []
  selectedVersion.value = ''
  appVerId.value = ''
  versionsFetched.value = false
  
  // 同步状态到store
  syncStateToStore()
}

// 自动选择第一个账号
const autoSelectFirstAccount = () => {
  if (accounts.value.length > 0 && (selectedAccount.value === null || selectedAccount.value === undefined || selectedAccount.value === '')) {
    // 尝试从 localStorage 恢复上次选择的账号
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
const selectedAccount = ref(null)  // 改为 null 而不是空字符串

// 监听账号选择变化，保存到 localStorage
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

// Progress state - sync with store
const showProgress = ref(false)
const progressPercent = ref(0)
const progressStage = ref('等待任务…')
const logs = ref('')

// Search state
const searchMode = ref('search') // 'search' or 'appid'
const searchQuery = ref('')
const searchResults = ref([])
const searching = ref(false)

// Upload state
const uploadUrl = ref(`${API_BASE}/upload-ipa`)
const uploading = ref(false)
const uploadProgress = ref(0)
const uploadResult = ref({
  jobId: '',
  fileName: '',
  installUrl: ''
})

// Install state
const downloadInstallUrl = ref('')
const showInstallButton = ref(false)

// HTTPS detection
const isHttps = ref(false)
const currentProtocol = ref('')

// Sync state with store on mount and update
const syncStateToStore = () => {
  const appStore = useAppStore()
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
  const appStore = useAppStore()
  const state = appStore.downloadState
  // 只恢复非 undefined 的值，避免覆盖自动选择的账号
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

// 监听账号列表变化，自动选择账号
watch(accounts, () => {
  autoSelectFirstAccount()
}, { deep: true })

const API_BASE = '/api'

const canAddToBatch = computed(() => {
  return (selectedAccount.value === 0 || !!selectedAccount.value) && !!appid.value
})

const loadAccounts = async () => {
  const saved = localStorage.getItem('ipa_accounts')
  if (saved) {
    try {
      accounts.value = JSON.parse(saved)
    } catch (e) {
      accounts.value = []
    }
  }
  
  // 从服务器获取最新的账号列表
  try {
    const response = await fetch(`${API_BASE}/accounts`)
    const data = await response.json()
    
    if (data.ok && data.data) {
      accounts.value = data.data.map(acc => ({
        token: acc.token,
        email: acc.email,
        dsid: acc.dsid,
        region: acc.region || 'US'
      }))
      // 更新本地存储
      localStorage.setItem('ipa_accounts', JSON.stringify(accounts.value))
      
      // 自动选择第一个账号
      autoSelectFirstAccount()
    }
  } catch (error) {
    console.error('Failed to load accounts from server:', error)
  }
}

const addLog = (message) => {
  const timestamp = new Date().toLocaleTimeString()
  logs.value += `[${timestamp}] ${message}\n`
}

// 跳转到账号标签页
const goToAccountTab = () => {
  const appStore = useAppStore()
  appStore.activeTab = 'account'
}

// Search functionality - 使用所选账号的区域
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

  // 检查是否已选择账号
  if (accounts.value.length === 0 || selectedAccount.value === '' || selectedAccount.value === null) {
    searchResults.value = []
    return
  }

  searching.value = true
  try {
    // 获取当前选择账号的区域
    const account = accounts.value[selectedAccount.value]
    const region = account?.region || 'US'
    
    // Check if it's a numeric App ID
    if (/^\d+$/.test(query)) {
      // Direct App ID lookup
      const response = await fetch(`https://itunes.apple.com/lookup?id=${query}&country=${region}`)
      const data = await response.json()
      
      if (data.results && data.results.length > 0) {
        searchResults.value = data.results
      } else {
        searchResults.value = []
      }
    } else {
      // Search by name or bundle ID
      const response = await fetch(`https://itunes.apple.com/search?term=${encodeURIComponent(query)}&country=${region}&media=software&limit=10`)
      const data = await response.json()
      
      if (data.results) {
        searchResults.value = data.results
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
      artistName: '未知开发者',
      bundleId: 'unknown.bundle',
      artworkUrl60: null,
      artworkUrl100: null,
      version: '未知',
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
    // 自动查询版本
    fetchVersions()
  }
})

const fetchVersions = async () => {
  if (!appid.value) {
    ElMessage.warning('请填写 APPID')
    return
  }

  if (selectedAccount.value === '' || selectedAccount.value === null) {
    ElMessage.warning('请先选择账号')
    return
  }

  const account = accounts.value[selectedAccount.value]
  const region = account?.region || 'US'

  fetchingVersions.value = true
  addLog(`[查询] 正在查询 APPID=${appid.value} 的历史版本（区域：${getRegionLabel(region)}）...`)

  try {
    const response = await fetch(`${API_BASE}/versions?appid=${encodeURIComponent(appid.value)}&region=${encodeURIComponent(region)}`)
    const data = await response.json()

    if (!data.ok) {
      ElMessage.error(`查询失败：${data.error || '未知错误'}`)
      addLog(`[查询] 失败：${data.error || '未知错误'}`)
      return
    }

    versions.value = data.data || []
    versionsFetched.value = true
    addLog(`[查询] 获取到 ${versions.value.length} 条版本记录`)
  } catch (error) {
    ElMessage.error(`查询失败：${error.message}`)
    addLog(`[查询] 失败：${error.message}`)
  } finally {
    fetchingVersions.value = false
  }
}

const handleVersionChange = () => {
  appVerId.value = selectedVersion.value || ''
}

const addCurrentSelectionToBatch = () => {
  if (!canAddToBatch.value) {
    ElMessage.warning('请先选择账号并填写 APPID')
    return
  }

  const account = accounts.value[selectedAccount.value]
  const versionLabel = versions.value.find(v => String(v.external_identifier) === String(selectedVersion.value))?.bundle_version
  const appStore = useAppStore()
  const result = appStore.addBatchDraftItem({
    app_id: String(appid.value),
    app_name: props.selectedApp?.trackName || `App ID: ${appid.value}`,
    version: appVerId.value || undefined,
    version_label: versionLabel || undefined,
    account_email: account.email,
    account_region: account.region || 'US'
  })

  if (result.added) {
    ElMessage.success('已加入批量下载草稿')
  } else {
    ElMessage.success('批量下载草稿已更新')
  }

  const appStoreRef = useAppStore()
  appStoreRef.activeTab = 'batch'
}

const directLinkDownload = async (autoPurchase = false) => {
  if (!selectedAccount.value && selectedAccount.value !== 0) {
    ElMessage.warning('请选择登录账号')
    return
  }
  if (!appid.value) {
    ElMessage.warning('请填写 APPID')
    return
  }

  const account = accounts.value[selectedAccount.value]
  addLog('[直链] 获取直链中…')

  try {
    const url = `${API_BASE}/download-url?token=${encodeURIComponent(account.token)}&appid=${encodeURIComponent(appid.value)}${appVerId.value ? `&appVerId=${encodeURIComponent(appVerId.value)}` : ''}${autoPurchase ? '&autoPurchase=true' : ''}`
    const response = await fetch(url)
    const data = await response.json()

    if (!data.ok) {
      if (data.needsPurchase && !autoPurchase) {
        // 需要购买，显示确认对话框
        const confirmed = await ElMessageBox.confirm(
          '您尚未购买此应用，是否现在购买并下载？',
          '需要购买',
          {
            confirmButtonText: '购买并下载',
            cancelButtonText: '取消',
            type: 'warning'
          }
        ).then(() => true).catch(() => false)
        
        if (confirmed) {
          return directLinkDownload(true)
        } else {
          addLog(`[直链] 用户取消购买`)
          return
        }
      }
      ElMessage.error(`直链获取失败：${data.error || '未知错误'}`)
      addLog(`[直链] 失败：${data.error || '未知错误'}`)
      return
    }

    addLog(`[直链] 成功：文件名=${data.fileName}，即将从 Apple CDN 直连下载`)
    addLog(`[直链] URL（部分）=${String(data.url).slice(0, 80)}...`)

    // Trigger browser download
    const a = document.createElement('a')
    a.href = data.url
    a.download = data.fileName || ''
    a.rel = 'noopener'
    document.body.appendChild(a)
    a.click()
    a.remove()
  } catch (error) {
    ElMessage.error(`直链获取失败：${error.message}`)
    addLog(`[直链] 失败：${error.message}`)
  }
}

const startDownloadWithProgress = async (autoPurchase = false) => {
  if (!selectedAccount.value && selectedAccount.value !== 0) {
    ElMessage.warning('请选择登录账号')
    return
  }
  if (!appid.value) {
    ElMessage.warning('请填写 APPID')
    return
  }

  const account = accounts.value[selectedAccount.value]
  
  // Reset progress
  showProgress.value = true
  progressPercent.value = 0
  progressStage.value = '准备中…'
  logs.value = ''
  addLog('[进度] 创建下载任务…')

  try {
    const response = await fetch(`${API_BASE}/start-download-direct`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        token: account.token,
        appid: appid.value,
        appVerId: appVerId.value || undefined,
        autoPurchase
      })
    })
    const data = await response.json()

    if (!data.ok) {
      if (data.needsPurchase && !autoPurchase) {
        // 需要购买，显示确认对话框
        const confirmed = await ElMessageBox.confirm(
          '您尚未购买此应用，是否现在购买并下载？',
          '需要购买',
          {
            confirmButtonText: '购买并下载',
            cancelButtonText: '取消',
            type: 'warning'
          }
        ).then(() => true).catch(() => false)
        
        if (confirmed) {
          return startDownloadWithProgress(true)
        } else {
          showProgress.value = false
          addLog(`[进度] 用户取消购买`)
          return
        }
      }
      ElMessage.error(`创建任务失败：${data.error || '未知错误'}`)
      addLog(`[进度] 创建任务失败：${data.error || '未知错误'}`)
      return
    }

    const { jobId } = data
    addLog(`[进度] 任务已创建：${jobId}`)

    // 添加到队列
    const queueItem = {
      id: jobId,
      app: props.selectedApp || { trackName: appid.value, artworkUrl100: '', artworkUrl60: '' },
      account: account,
      status: 'downloading',
      progress: 0,
      logs: logs.value,
      timestamp: new Date().toISOString()
    }
    emit('download-started', queueItem)

    // Connect to SSE
    connectToSSE(jobId, queueItem)
  } catch (error) {
    ElMessage.error(`创建任务失败：${error.message}`)
    addLog(`[进度] 创建任务失败：${error.message}`)
  }
}

const connectToSSE = (jobId, queueItem) => {
  const es = new EventSource(`${API_BASE}/progress-sse?jobId=${encodeURIComponent(jobId)}`)

  es.addEventListener('progress', (ev) => {
    try {
      const data = JSON.parse(ev.data)
      
      if (data?.progress?.percent != null) {
        progressPercent.value = data.progress.percent
        // 更新队列项进度
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { progress: data.progress.percent })
      }
      
      if (data?.progress?.stage) {
        const stageMap = {
          'auth': '获取下载信息',
          'download-start': '开始下载',
          'download-progress': '下载中',
          'merge': '合并分块',
          'sign': '写入签名',
          'done': '完成'
        }
        progressStage.value = stageMap[data.progress.stage] || data.progress.stage
        // 更新队列项状态
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, { stage: progressStage.value })
      }
      
      if (data?.error) {
        addLog(`[错误] ${data.error}`)
        const appStore = useAppStore()
        appStore.updateQueueItem(jobId, {
          status: 'failed',
          error: data.error
        })
      }
      
      if (data.status === 'ready') {
        progressStage.value = '准备下载文件…'
        const a = document.createElement('a')
        a.href = `${API_BASE}/download-file?jobId=${encodeURIComponent(jobId)}`
        a.rel = 'noopener'
        document.body.appendChild(a)
        a.click()
        a.remove()
        
        progressPercent.value = 100
        progressStage.value = '已开始下载'
        addLog('[进度] 文件下载已开始')

        // 更新队列项状态
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
    } catch (_) {}
  })

  es.addEventListener('end', (ev) => {
    try {
      const data = JSON.parse(ev.data || '{}')
      if (data.status === 'ready') {
        addLog('[完成] 任务已就绪')
        // 获取任务信息，包括安装URL
        fetch(`${API_BASE}/job-info?jobId=${encodeURIComponent(jobId)}`)
          .then(res => res.json())
          .then(jobData => {
            if (jobData.ok && jobData.data?.installUrl) {
              addLog('[安装] 安装链接已生成')
              // 显示安装按钮
              downloadInstallUrl.value = jobData.data.installUrl
              showInstallButton.value = true
            }
          })
          .catch(() => {
            // 忽略错误
          })
      } else if (data.status === 'failed') {
        addLog('[失败] 任务失败')
        if (queueItem) {
          queueItem.status = 'error'
        }
      } else {
        addLog(`[结束] 任务结束：${data.status || 'unknown'}`)
      }
    } catch (_) {}
    es.close()
  })

  es.onerror = () => {
    addLog('[错误] SSE 连接断开')
    if (queueItem) {
      queueItem.status = 'error'
      queueItem.error = 'SSE 连接断开'
    }
    es.close()
  }
}

// 监听账号更新
watch(() => props.accountsUpdated, () => {
  loadAccounts()
})

// 上传相关函数
const beforeUpload = (file) => {
  const isIPA = file.name.endsWith('.ipa')
  const isLt2G = file.size / 1024 / 1024 / 1024 < 2

  if (!isIPA) {
    ElMessage.error('只能上传 .ipa 格式的文件')
    return false
  }
  if (!isLt2G) {
    ElMessage.error('上传文件大小不能超过 2GB')
    return false
  }

  uploading.value = true
  uploadProgress.value = 0
  return true
}

const handleUploadProgress = (event) => {
  uploadProgress.value = Math.floor(event.percent)
}

const handleUploadSuccess = (response) => {
  uploading.value = false
  uploadProgress.value = 100

  if (response.ok) {
    uploadResult.value = {
      jobId: response.jobId,
      fileName: response.fileName,
      installUrl: response.installUrl
    }
    ElMessage.success('文件上传成功')
  } else {
    ElMessage.error(response.error || '上传失败')
  }
}

const handleUploadError = (error) => {
  uploading.value = false
  uploadProgress.value = 0
  ElMessage.error('上传失败：' + error.message)
}

const installDownloadedIpa = async () => {
  if (!downloadInstallUrl.value) {
    ElMessage.warning('安装链接未生成')
    return
  }

  // 检测当前环境是否为 HTTPS
  const isHttpsEnvironment = window.location.protocol === 'https:'
  const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  
  // iOS 设备通常需要 HTTPS 才能使用 itms-services 安装
  if (!isHttpsEnvironment && !isLocalhost) {
    // 非 HTTPS 环境，给出提示
    const action = await ElMessageBox.confirm(
      '当前环境不是 HTTPS，iOS 设备无法直接安装。您希望：',
      '环境检测',
      {
        distinguishCancelAndClose: true,
        confirmButtonText: '直接下载文件',
        cancelButtonText: '取消操作',
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
      // 用户选择直接下载文件
      ElMessage.info('正在准备下载...')
      // 这里可以触发文件下载，需要从 jobId 获取文件
      // 由于当前没有保存 jobId，我们提示用户
      ElMessageBox.alert(
        '请使用"直链下载"功能重新下载文件，或部署到 HTTPS 环境后再试。',
        '提示',
        {
          type: 'info',
          confirmButtonText: '我知道了'
        }
      )
    }
    // 如果用户选择取消，什么都不做
    return
  }

  // HTTPS 环境或 localhost，直接安装
  if (isHttpsEnvironment) {
    ElMessage.success('正在打开安装链接...')
    window.location.href = downloadInstallUrl.value
  } else if (isLocalhost) {
    // localhost 环境，给出提示但仍允许尝试
    const confirmed = await ElMessageBox.confirm(
      '当前是 localhost 环境，iOS 设备可能无法安装。建议部署到 HTTPS 服务器。是否继续？',
      '环境提示',
      {
        confirmButtonText: '继续尝试',
        cancelButtonText: '取消',
        type: 'info'
      }
    ).then(() => true).catch(() => false)

    if (confirmed) {
      window.location.href = downloadInstallUrl.value
    }
  }
}

const installUploadedIpa = async () => {
  if (!uploadResult.value.installUrl) {
    ElMessage.warning('安装链接未生成')
    return
  }

  // 检测当前环境是否为 HTTPS
  const isHttpsEnvironment = window.location.protocol === 'https:'
  const isLocalhost = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'
  
  if (!isHttpsEnvironment && !isLocalhost) {
    // 非 HTTPS 环境，给出提示
    const action = await ElMessageBox.confirm(
      '当前环境不是 HTTPS，iOS 设备无法直接安装。您希望：',
      '环境检测',
      {
        distinguishCancelAndClose: true,
        confirmButtonText: '直接下载文件',
        cancelButtonText: '取消操作',
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
      // 用户选择直接下载文件
      ElMessage.info('上传的文件已保存在服务器，请联系管理员获取')
    }
    return
  }

  // HTTPS 环境或 localhost，直接安装
  if (isHttpsEnvironment) {
    ElMessage.success('正在打开安装链接...')
    window.location.href = uploadResult.value.installUrl
  } else if (isLocalhost) {
    const confirmed = await ElMessageBox.confirm(
      '当前是 localhost 环境，iOS 设备可能无法安装。建议部署到 HTTPS 服务器。是否继续？',
      '环境提示',
      {
        confirmButtonText: '继续尝试',
        cancelButtonText: '取消',
        type: 'info'
      }
    ).then(() => true).catch(() => false)

    if (confirmed) {
      window.location.href = uploadResult.value.installUrl
    }
  }
}

onMounted(() => {
  loadAccounts()
  restoreStateFromStore()
  
  // 检测当前环境
  isHttps.value = window.location.protocol === 'https:'
  currentProtocol.value = window.location.protocol
  
  console.log(`[Environment] Protocol: ${currentProtocol.value}, HTTPS: ${isHttps.value}`)
})
</script>

<style scoped>
.search-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  padding: 8px 16px;
}

.search-input :deep(.el-input__inner) {
  font-size: 15px;
}

/* 快速账号选择器样式 */
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

/* 迷你区域徽章 */
.region-badge-mini {
  display: inline-flex;
  height: 28px;
  align-items: center;
  padding: 2px 8px;
  border-radius: 6px;
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

/* 区域徽章样式 */
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

/* 移动端响应式样式 */
@media (max-width: 767px) {
  .card {
    padding: 12px;
  }
  
  .action-button {
    height: 48px;
    font-size: 15px;
  }
  
  /* 移动端账号选择器 */
  .account-quick-select {
    width: 100%;
    margin-top: 12px;
  }
  
  /* 移动端搜索区域提示 */
  .bg-blue-50.dark\:bg-blue-900\/20 {
    flex-direction: column;
    align-items: flex-start !important;
  }
  
  .bg-blue-50.dark\:bg-blue-900\/20 .flex {
    flex-direction: column;
    width: 100%;
  }
  
  /* 搜索结果卡片自适应 */
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
  
  /* 选中应用信息卡片 */
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
  
  /* 上传区域 */
  .upload-demo :deep(.el-upload-dragger) {
    padding: 20px !important;
  }
  
  /* 进度卡片 */
  .el-card {
    margin-top: 12px !important;
  }
}
</style>
