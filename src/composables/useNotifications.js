import { ref, watch } from 'vue'

// --- Singleton state shared across all consumers ---
const permission = ref(
  typeof Notification !== 'undefined' ? Notification.permission : 'denied'
)

const defaultSettings = {
  versionUpdate: true,
  downloadComplete: true,
  downloadFailed: true
}

const settingsKey = 'ipa_notification_settings'

function loadSettings() {
  try {
    const raw = localStorage.getItem(settingsKey)
    if (raw) {
      const parsed = JSON.parse(raw)
      return { ...defaultSettings, ...parsed }
    }
  } catch { /* ignore */ }
  return { ...defaultSettings }
}

const settings = ref(loadSettings())

watch(settings, (val) => {
  localStorage.setItem(settingsKey, JSON.stringify(val))
}, { deep: true })

// --- Interval-based version check ---
let versionCheckTimer = null

/**
 * Periodically check subscribed apps for new versions.
 * Uses the existing /api/check-updates endpoint.
 */
function startVersionPolling(intervalMs = 30 * 60 * 1000) {
  stopVersionPolling()
  // Run once immediately, then periodically
  _checkForUpdates()
  versionCheckTimer = setInterval(_checkForUpdates, intervalMs)
}

function stopVersionPolling() {
  if (versionCheckTimer) {
    clearInterval(versionCheckTimer)
    versionCheckTimer = null
  }
}

async function _checkForUpdates() {
  if (!settings.value.versionUpdate) return
  try {
    const res = await fetch('/api/check-updates')
    const data = await res.json()
    if (data.ok && data.data?.updates?.length > 0) {
      for (const update of data.data.updates) {
        send(
          `🔄 ${update.app_name} has a new version`,
          `${update.current_version} → ${update.latest_version}`,
          { tag: `version-${update.app_id}` }
        )
      }
    }
  } catch { /* silent */ }
}

// --- Core send ---

/**
 * Send a browser notification.
 * Respects user settings and permission state.
 */
function send(title, body, options = {}) {
  if (typeof Notification === 'undefined') return
  if (permission.value !== 'granted') return
  // Don't notify if tab is focused and user prefers not to be disturbed (optional future enhancement)
  try {
    // Deduplicate by tag
    const n = new Notification(title, {
      body,
      icon: '/favicon.ico',
      ...options
    })
    n.onclick = () => {
      window.focus()
      n.close()
    }
  } catch { /* silent */ }
}

/**
 * Request notification permission from the browser.
 * Returns the new permission state.
 */
async function requestPermission() {
  if (typeof Notification === 'undefined') return 'denied'
  if (permission.value === 'granted') return 'granted'
  const result = await Notification.requestPermission()
  permission.value = result
  return result
}

/**
 * Check if a specific notification type is enabled.
 */
function isEnabled(type) {
  return settings.value[type] === true
}

/**
 * Toggle a specific notification type.
 */
function toggle(type, value) {
  if (type in settings.value) {
    settings.value[type] = value
  }
}

/**
 * Enable all notifications (convenience).
 */
function enableAll() {
  settings.value = { ...defaultSettings }
}

/**
 * Disable all notifications (convenience).
 */
function disableAll() {
  settings.value.versionUpdate = false
  settings.value.downloadComplete = false
  settings.value.downloadFailed = false
}

/**
 * Initialize: start version polling if enabled.
 * Call once from App.vue on mount.
 */
function init() {
  if (settings.value.versionUpdate) {
    startVersionPolling()
  }
}

/**
 * Notification helpers for specific events.
 */
function notifyDownloadComplete(appName, fileName) {
  if (!settings.value.downloadComplete) return
  send(
    `✅ Download Complete`,
    `${appName}${fileName ? ` — ${fileName}` : ''}`,
    { tag: `dl-complete-${Date.now()}` }
  )
}

function notifyDownloadFailed(appName, error) {
  if (!settings.value.downloadFailed) return
  send(
    `❌ Download Failed`,
    `${appName}${error ? `：${error}` : ''}`,
    { tag: `dl-failed-${Date.now()}` }
  )
}

function notifyVersionUpdate(appName, fromVersion, toVersion) {
  if (!settings.value.versionUpdate) return
  send(
    `🔄 New Version Found`,
    `${appName}：${fromVersion} → ${toVersion}`,
    { tag: `version-${appName}` }
  )
}

export function useNotifications() {
  return {
    permission,
    settings,
    // Core
    send,
    requestPermission,
    isEnabled,
    toggle,
    enableAll,
    disableAll,
    init,
    // Version polling
    startVersionPolling,
    stopVersionPolling,
    // Event helpers
    notifyDownloadComplete,
    notifyDownloadFailed,
    notifyVersionUpdate
  }
}
