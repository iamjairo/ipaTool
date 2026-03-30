<template>
  <div
    class="tab-layout"
    :class="{ 'mobile-layout': isMobile }"
  >
    <!-- Desktop: Tab Bar -->
    <div
      v-if="!isMobile"
      class="desktop-tab-bar"
    >
      <div class="tab-bar-inner">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="['tab-btn', { 'tab-btn-active': appStore.activeTab === tab.id }]"
          :title="tab.label"
          @click="appStore.activeTab = tab.id"
        >
          <div class="tab-btn-content">
            <!-- badge wrapper -->
            <el-badge
              v-if="tab.badge"
              :value="tab.badge"
              :max="99"
              class="tab-badge"
            >
              <svg
                class="tab-svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                v-html="tab.svgPath"
              />
            </el-badge>
            <svg
              v-else
              class="tab-svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              v-html="tab.svgPath"
            />
          </div>
        </button>
      </div>
    </div>

    <!-- Content -->
    <div
      class="tab-content"
      :class="{ 'with-mobile-tabs': isMobile }"
    >
      <component
        :is="currentTabComponent"
        v-bind="currentTabProps"
        @app-selected="handleAppSelected"
        @download-started="handleDownloadStarted"
        @accounts-updated="handleAccountsUpdated"
        @remove-item="emit('remove-item', $event)"
        @clear-all="emit('clear-queue')"
        @logout="emit('logout')"
      />
    </div>

    <!-- Mobile: Bottom Tab Bar -->
    <div
      v-if="isMobile"
      class="mobile-tab-bar"
    >
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['mobile-tab-btn', { 'mobile-tab-btn-active': appStore.activeTab === tab.id }]"
        @click="appStore.activeTab = tab.id"
      >
        <div class="mobile-tab-icon">
          <el-badge
            v-if="tab.badge"
            :value="tab.badge"
            :max="99"
            class="tab-badge"
          >
            <svg
              class="tab-svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              v-html="tab.svgPath"
            />
          </el-badge>
          <svg
            v-else
            class="tab-svg"
            viewBox="0 0 24 24"
              fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            v-html="tab.svgPath"
          />
        </div>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useAppStore } from '../stores/app'
import DownloadManager from './DownloadManager.vue'
import DownloadQueue from './DownloadQueue.vue'
import IpaManager from './IpaManager.vue'
import AppSubscription from './AppSubscription.vue'
import Settings from './Settings.vue'

const appStore = useAppStore()
const emit = defineEmits(['app-selected', 'download-started', 'accounts-updated', 'remove-item', 'clear-queue', 'logout'])
const isMobile = ref(false)

const checkMobile = () => {
  isMobile.value = window.innerWidth < 768
}

const handleAccountsUpdated = (v) => emit('accounts-updated', v)
const handleAppSelected = (app) => emit('app-selected', app)
const handleDownloadStarted = (task) => emit('download-started', task)

// SVG paths for tab icons — semantic, clear, consistent
const tabs = computed(() => [
  {
    id: 'download',
    label: '下载',
    svgPath: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>',
    badge: appStore.downloadState.selectedApp ? '1' : null
  },
  {
    id: 'queue',
    label: '队列',
    svgPath: '<line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>',
    badge: appStore.taskQueue.length > 0 ? String(appStore.taskQueue.length) : null
  },
  {
    id: 'ipa',
    label: 'IPA',
    svgPath: '<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>'
  },
  {
    id: 'subscription',
    label: '订阅',
    svgPath: '<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/>'
  },
  {
    id: 'settings',
    label: '设置',
    svgPath: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>'
  }
])

const currentTabComponent = computed(() => {
  const map = {
    download: DownloadManager,
    queue: DownloadQueue,
    ipa: IpaManager,
    subscription: AppSubscription,
    settings: Settings
  }
  return map[appStore.activeTab] || DownloadManager
})

const currentTabProps = computed(() => {
  if (appStore.activeTab === 'download') {
    return {
      selectedApp: appStore.downloadState.selectedApp,
      accountsUpdated: appStore.accountsUpdateCounter
    }
  }
  if (appStore.activeTab === 'queue') {
    return { queue: appStore.taskQueue }
  }
  return {}
})

onMounted(() => {
  checkMobile()
  window.addEventListener('resize', checkMobile)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkMobile)
})
</script>

<style scoped>
.tab-layout {
  display: flex;
  flex-direction: column;
  min-height: calc(100vh - 180px);
}

.tab-content {
  flex: 1;
  overflow-y: auto;
}

.tab-content.with-mobile-tabs {
  padding-bottom: 100px;
}

/* ===== Desktop Tab Bar ===== */
.desktop-tab-bar {
  position: sticky;
  top: 0;
  z-index: 100;
  margin: 0 -48px;
  padding: 12px 0;
  background: rgba(255,255,255,0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(0,0,0,0.06);
}

:root.dark .desktop-tab-bar,
.dark .desktop-tab-bar {
  background: rgba(17,24,39,0.85);
  border-bottom-color: rgba(55,65,81,0.5);
}

.tab-bar-inner {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  max-width: 320px;
  margin: 0 auto;
  padding: 4px;
  background: rgba(0,0,0,0.04);
  border-radius: 14px;
}

.dark .tab-bar-inner {
  background: rgba(255,255,255,0.06);
}

.tab-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 44px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.tab-btn:hover {
  color: #3b82f6;
  background: rgba(59,130,246,0.08);
}

.dark .tab-btn {
  color: #9ca3af;
}

.dark .tab-btn:hover {
  color: #60a5fa;
  background: rgba(96,165,250,0.12);
}

.tab-btn-active {
  color: #3b82f6 !important;
  background: rgba(255,255,255,0.95) !important;
  box-shadow: 0 1px 4px rgba(0,0,0,0.1);
}

.dark .tab-btn-active {
  color: #60a5fa !important;
  background: rgba(0,0,0,0.25) !important;
  box-shadow: 0 1px 4px rgba(0,0,0,0.3);
}

.tab-btn-content {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.tab-svg {
  width: 22px;
  height: 22px;
}

.tab-badge :deep(.el-badge__content) {
  position: absolute;
  top: -6px;
  right: -8px;
  font-size: 10px;
  min-width: 16px;
  height: 16px;
  line-height: 16px;
  padding: 0 4px;
  z-index: 10;
}

/* ===== Mobile Tab Bar ===== */
.mobile-tab-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
  display: flex;
  justify-content: space-around;
  align-items: center;
  height: 60px;
  padding-bottom: env(safe-area-inset-bottom);
  background: rgba(255,255,255,0.97);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-top: 1px solid rgba(0,0,0,0.08);
  box-shadow: 0 -2px 10px rgba(0,0,0,0.04);
}

.dark .mobile-tab-bar {
  background: rgba(31,41,55,0.97);
  border-top-color: rgba(55,65,81,0.5);
  box-shadow: 0 -2px 10px rgba(0,0,0,0.2);
}

.mobile-tab-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  border: none;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-tap-highlight-color: transparent;
}

.dark .mobile-tab-btn {
  color: #9ca3af;
}

.mobile-tab-btn-active {
  color: #3b82f6 !important;
}

.dark .mobile-tab-btn-active {
  color: #60a5fa !important;
}

.mobile-tab-icon {
  position: relative;
}

.mobile-tab-icon .tab-svg {
  width: 24px;
  height: 24px;
}

.mobile-tab-btn .tab-badge :deep(.el-badge__content) {
  position: absolute;
  top: -6px;
  right: -10px;
  font-size: 10px;
  min-width: 16px;
  height: 16px;
  line-height: 16px;
  padding: 0 4px;
  z-index: 10;
}

@media (max-width: 767px) {
  .tab-content.with-mobile-tabs {
    padding-bottom: 80px;
  }
}
</style>
