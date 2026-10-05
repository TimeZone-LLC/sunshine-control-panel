<template>
  <SidebarMenu
    ref="sidebarMenuRef"
    :webui-path="currentPath"
    @route-change="handleRouteChange"
    @webui-navigate="handleWebuiNavigate"
  >
    <div class="iframe-container">
      <div v-if="loading" class="loading-overlay">
        <div class="loading-container">
          <svg class="loading-spinner" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <circle class="loading-spinner-track" cx="8" cy="8" r="7.5" />
            <path class="loading-spinner-head" d="M8 .5A7.5 7.5 0 0 1 15.5 8" />
          </svg>
          <div class="loading-text" role="status" aria-live="polite">
            <template v-if="proxyHealthFailed">
              <p class="proxy-health-error">{{ t.sunshineFrame.proxyUnavailable }}</p>
              <button class="proxy-health-retry" type="button" @click="retryProxyHealthCheck">
                {{ t.sunshineFrame.retry }}
              </button>
            </template>
            <p v-else>{{ t.sunshineFrame.preparing.replace('{path}', currentPath) }}</p>
          </div>
        </div>
      </div>

      <iframe
        ref="sunshineIframe"
        v-show="!loading"
        :src="sunshineUrl"
        class="sunshine-iframe"
        @load="onLoad"
        frameborder="0"
        allow="autoplay; clipboard-read; clipboard-write; fullscreen"
      ></iframe>
    </div>
  </SidebarMenu>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { sunshine } from '@/tauri-adapter.js'
import { checkLocalProxyHealth } from '../utils/proxyHealth.js'
import { isNativeControlPanelMessage, isTrustedNativeControlPanelMessage } from '../composables/nativeBridge.js'
import { useI18n } from '../desktop/i18n/index.js'
import SidebarMenu from './SidebarMenu.vue'
import { isWebuiNavigationPath } from '../composables/webuiNavigation.js'

invoke('main_panel_loading').catch(() => {})
const { t } = useI18n()

// Refs
const loading = ref(true)
const sunshineUrl = ref('')
const currentPath = ref('/')
const sunshineIframe = ref(null)
const sidebarMenuRef = ref(null)
const proxyHealthFailed = ref(false)

// State
let pollTimer = null
let unlistenVddSettings = null
let messageHandler = null
let visibilityHandlerRef = null
let proxyBase = '' // Proxy base URL, used to restore the iframe to the right page.
let loadedFrameRevealTimer = null
let awaitingAppReady = true
let localProxyVerified = false
let requestedWebuiPath = ''

// Constants
const POLL_INTERVAL = 3000
const LOADED_FRAME_REVEAL_DELAY = 300

const clearLoadedFrameReveal = () => {
  if (!loadedFrameRevealTimer) return
  clearTimeout(loadedFrameRevealTimer)
  loadedFrameRevealTimer = null
}

const beginLoading = () => {
  clearLoadedFrameReveal()
  awaitingAppReady = true
  loading.value = true
}

const finishLoading = () => {
  clearLoadedFrameReveal()
  awaitingAppReady = false
  loading.value = false
}

const scheduleLoadedFrameReveal = () => {
  clearLoadedFrameReveal()
  if (!awaitingAppReady) return

  loadedFrameRevealTimer = setTimeout(() => {
    console.debug('[SunshineFrame] revealing loaded iframe without WebUI ready signal')
    finishLoading()
  }, LOADED_FRAME_REVEAL_DELAY)
}

// Utility functions
const extractPathFromUrl = (url) => {
  try {
    const { pathname, search, hash } = new URL(url)
    return pathname + search + hash
  } catch {
    return '/'
  }
}

const setAnimationsPaused = (paused) => {
  document.body?.classList.toggle('paused-animations', paused)
}

const isWelcomePath = (url) => {
  if (!url) return false
  try {
    const path = new URL(url).pathname.toLowerCase()
    return path === '/welcome' || path.startsWith('/welcome/') || path === '/welcome.html'
  } catch {
    return url.includes('/welcome') || url.includes('welcome.html')
  }
}

const openWelcome = () => sidebarMenuRef.value?.openWelcome?.()

const refreshProxyTarget = async () => {
  const refreshedTarget = await sunshine.refreshTarget()
  if (!refreshedTarget) {
    console.warn('[SunshineFrame] refresh proxy target failed')
  }
}

const verifyLocalProxy = async () => {
  try {
    const healthCheck = await invoke('get_proxy_health_check')
    return checkLocalProxyHealth(healthCheck)
  } catch (error) {
    console.warn('[SunshineFrame] local proxy health check failed:', error)
    return false
  }
}

const retryProxyHealthCheck = () => {
  window.location.reload()
}

// Navigation handler
const handleNavigateFrame = (event) => {
  const url = event.detail?.url
  if (!url) return

  if (isWelcomePath(url)) {
    console.log('Intercepted the welcome page load; opening the Vue welcome component')
    openWelcome()
    return
  }

  beginLoading()
  sunshineUrl.value = url
}

// Route changes: suspend the iframe when leaving HOME, resume on return.
let savedIframeUrl = ''
// iframe URL saved when the window is hidden or minimised, tracked separately
// from tab switches.
let windowSuspendedUrl = ''

const handleWebuiNavigate = (path) => {
  if (!isWebuiNavigationPath(path)) return
  requestedWebuiPath = path
  savedIframeUrl = ''
  windowSuspendedUrl = ''
  if (path === currentPath.value && sunshineUrl.value && sunshineUrl.value !== 'about:blank') return
  currentPath.value = path
  beginLoading()
  if (proxyBase && localProxyVerified) sunshineUrl.value = proxyBase + path
}

const handleRouteChange = ({ from, to }) => {
  if (from === 'home' && to !== 'home') {
    // Leaving the streaming pages suspends the iframe and its timers.
    savedIframeUrl = proxyBase ? proxyBase + currentPath.value : sunshineUrl.value
    sunshineUrl.value = 'about:blank'
    beginLoading()
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
    console.log('[SunshineFrame] iframe suspended: navigated to about:blank')
  } else if (from !== 'home' && to === 'home') {
    // Resume the last page unless an explicit sidebar destination was requested.
    if (savedIframeUrl && savedIframeUrl !== 'about:blank') {
      sunshineUrl.value = savedIframeUrl
      savedIframeUrl = ''
      console.log('[SunshineFrame] iframe resumed: URL restored')
    }
    // Restart the window state polling.
    if (!pollTimer && checkWindowStateFn) {
      pollTimer = setInterval(checkWindowStateFn, POLL_INTERVAL)
    }
  }
}

// Message handling
const createMessageHandler = () => {
  const getSunshineOrigin = () => {
    try {
      const url = new URL(sunshineUrl.value || sunshineIframe.value?.src || '', window.location.href)
      return url.origin === 'null' ? `${url.protocol}//${url.host}` : url.origin
    } catch {
      return ''
    }
  }

  const isTrustedSunshineEvent = (event) => (
    event.source === sunshineIframe.value?.contentWindow
    && event.origin === getSunshineOrigin()
  )

  const replyToSunshine = (event, message) => {
    const origin = getSunshineOrigin()
    if (!origin || !isTrustedSunshineEvent(event)) return
    event.source.postMessage(message, origin)
  }

  const handlers = {
    'native-navigation-context-request': (_data, event) => {
      replyToSunshine(event, {
        type: 'native-navigation-context',
        source: 'sunshine-control-panel',
        unified: true,
      })
    },
    'path-update': (data) => {
      currentPath.value = data.path
    },
    'navigation-start': (data) => {
      if (data.path) {
        if (isWelcomePath(data.path) || data.path.toLowerCase().includes('welcome')) {
          console.log('Intercepted navigation to the welcome page; opening the Vue welcome component')
          openWelcome()
          return
        }
        currentPath.value = data.path
      }
      beginLoading()
    },
    'webui-ready': finishLoading,
    'show-message': (data) => {
      // Handle message-display requests coming from the Web UI.
      if (data.source === 'sunshine-webui' && data.message) {
        const messageType = data.messageType || 'info'
        switch (messageType) {
          case 'success':
            ElMessage.success(data.message)
            break
          case 'error':
            ElMessage.error(data.message)
            break
          case 'warning':
            ElMessage.warning(data.message)
            break
          default:
            ElMessage.info(data.message)
        }
      }
    },
    'native-rtx-hdr-context-request': (_data, event) => {
      replyToSunshine(event, {
        type: 'native-rtx-hdr-context',
        source: 'sunshine-control-panel',
        available: true,
      })
    },
    'native-rtx-hdr-open-request': (data, event) => {
      const requestId = typeof data.requestId === 'string' ? data.requestId : ''
      sidebarMenuRef.value?.openRtxHdr?.()
      replyToSunshine(event, {
        type: 'native-rtx-hdr-open-result',
        source: 'sunshine-control-panel',
        requestId,
        ok: true,
      })
    },
  }

  return async (event) => {
    const { data } = event
    if (!isTrustedSunshineEvent(event)) return
    if (
      isNativeControlPanelMessage(data)
      && !isTrustedNativeControlPanelMessage(data, true)
    ) {
      return
    }
    if (data?.type && handlers[data.type]) {
      await handlers[data.type](data, event)
    }
  }
}


// Window state monitoring
let checkWindowStateFn = null
let visibilityGeneration = 0

const setupWindowStateMonitor = async (currentWindow) => {
  let lastMinimized = false
  let lastHidden = false

  checkWindowStateFn = async () => {
    try {
      const [isMinimized, isVisible] = await Promise.all([currentWindow.isMinimized(), currentWindow.isVisible()])

      if (isMinimized !== lastMinimized || !isVisible !== lastHidden) {
        lastMinimized = isMinimized
        lastHidden = !isVisible
        setAnimationsPaused(isMinimized || !isVisible)
      }
    } catch (e) {
      console.warn('Failed to detect the window state:', e)
    }
  }

  pollTimer = setInterval(checkWindowStateFn, POLL_INTERVAL)
  await checkWindowStateFn()

  const visibilityHandler = async () => {
    const generation = ++visibilityGeneration
    const hidden = document.hidden
    setAnimationsPaused(hidden)

    if (hidden) {
      // Window minimised or hidden: suspend the iframe, but only when it has content.
      if (sunshineUrl.value && sunshineUrl.value !== 'about:blank') {
        // Save the page the user is actually on, not the initial URL, so the
        // restore lands in the right place.
        windowSuspendedUrl = proxyBase ? proxyBase + currentPath.value : sunshineUrl.value
        // Prepare the shell while the native window is still hidden so the
        // about:blank iframe is never exposed during the next activation.
        beginLoading()
        sunshineUrl.value = 'about:blank'
        console.log('[SunshineFrame] window hidden, iframe suspended, saved:', windowSuspendedUrl)
      }
      if (pollTimer) {
        clearInterval(pollTimer)
        pollTimer = null
      }
    } else {
      // Window restored: resume the iframe, but only if the window hiding is
      // what suspended it.
      if (windowSuspendedUrl) {
        beginLoading()
        try {
          await refreshProxyTarget()
        } catch (error) {
          console.warn('[SunshineFrame] refresh proxy target on resume failed:', error)
        }

        // A second visibility transition may have happened while the proxy
        // refresh was in flight. Keep the iframe asleep if we are hidden again.
        if (generation !== visibilityGeneration || document.hidden) return

        const resumeUrl = windowSuspendedUrl
        windowSuspendedUrl = ''
        sunshineUrl.value = resumeUrl
        console.log('[SunshineFrame] window restored, iframe resumed')
      }
      if (!pollTimer && checkWindowStateFn) {
        pollTimer = setInterval(checkWindowStateFn, POLL_INTERVAL)
      }
    }
  }
  document.addEventListener('visibilitychange', visibilityHandler)
  visibilityHandlerRef = visibilityHandler

  return visibilityHandler
}

// Reload the iframe when the locale changes so the new language applies.
const handleLocaleChanged = () => {
  if (sunshineUrl.value && sunshineUrl.value !== 'about:blank') {
    beginLoading()
    // Remember the current path and reload through about:blank, which avoids
    // the cross-origin restriction.
    const targetUrl = proxyBase ? proxyBase + currentPath.value : sunshineUrl.value
    sunshineUrl.value = 'about:blank'
    setTimeout(() => { sunshineUrl.value = targetUrl }, 50)
  }
}

// Lifecycle
onUnmounted(() => {
  window.removeEventListener('navigate-frame', handleNavigateFrame)
  window.removeEventListener('locale-changed', handleLocaleChanged)
  if (messageHandler) window.removeEventListener('message', messageHandler)
  if (visibilityHandlerRef) document.removeEventListener('visibilitychange', visibilityHandlerRef)
  if (pollTimer) clearInterval(pollTimer)
  clearLoadedFrameReveal()
  unlistenVddSettings?.()
})

onMounted(async () => {
  window.addEventListener('navigate-frame', handleNavigateFrame)
  window.addEventListener('locale-changed', handleLocaleChanged)

  try {
    await invoke('wait_for_proxy_ready')
    const proxyBaseUrl = await sunshine.getProxyUrl()
    proxyBase = proxyBaseUrl

    if (!await verifyLocalProxy()) {
      proxyHealthFailed.value = true
      sunshineUrl.value = 'about:blank'
      await invoke('main_panel_ready').catch(() => {})
      return
    }
    localProxyVerified = true
    await refreshProxyTarget()

    const cmdLineUrl = await sunshine.getCommandLineUrl()

    messageHandler = createMessageHandler()
    window.addEventListener('message', messageHandler)

    if (cmdLineUrl || requestedWebuiPath) {
      const targetPath = requestedWebuiPath || extractPathFromUrl(cmdLineUrl)
      const fullUrl = proxyBaseUrl + targetPath

      if (isWelcomePath(fullUrl)) {
        console.log('Welcome page detected at startup; opening the Vue welcome component')
        openWelcome()
        sunshineUrl.value = 'about:blank'
        currentPath.value = '/'
        finishLoading()
        return
      }

      sunshineUrl.value = fullUrl
      currentPath.value = targetPath
    } else {
      sunshineUrl.value = proxyBaseUrl + '/'
      currentPath.value = '/'
    }

    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    const currentWindow = getCurrentWebviewWindow()

    await setupWindowStateMonitor(currentWindow)

    unlistenVddSettings = await currentWindow.listen('open-vdd-settings', () => {
      sidebarMenuRef.value?.openVddSettings?.()
    })

    await currentWindow.listen('open-web-stream', () => {
      sidebarMenuRef.value?.openWebStream?.()
    })

    await currentWindow.listen('open-welcome', openWelcome)
    await invoke('main_panel_ready')
  } catch (error) {
    console.error('Initialisation failed:', error)
    if (!localProxyVerified) {
      proxyHealthFailed.value = true
      sunshineUrl.value = 'about:blank'
      await invoke('main_panel_ready').catch(() => {})
      return
    }
    try {
      sunshineUrl.value = (await sunshine.getProxyUrl()) + '/'
    } catch {
      sunshineUrl.value = 'http://localhost:48081/'
    }
  }
})

const onLoad = () => {
  try {
    const iframe = sunshineIframe.value
    const newUrl = iframe?.contentWindow?.location?.href

    // Ignore the about:blank navigation used for suspending.
    if (!newUrl || newUrl === 'about:blank') return

    const path = extractPathFromUrl(newUrl)

    if (isWelcomePath(newUrl) || path.toLowerCase().includes('welcome')) {
      console.log('Welcome page load detected; intercepting and opening the Vue welcome component')
      openWelcome()
      sunshineUrl.value = 'about:blank'
      currentPath.value = '/'
      finishLoading()
      return
    }

    currentPath.value = path

    // Proxy and error documents do not mount the WebUI application.
    if (!iframe.contentDocument?.getElementById('app')) {
      finishLoading()
      return
    }
  } catch {
    // Unreadable across origins; keep the current path.
  }

  // Prefer the WebUI ready signal, but never hold a loaded iframe behind the
  // overlay for long when the signal is unavailable.
  scheduleLoadedFrameReveal()
}
</script>

<style scoped lang="less">
// The embedded Web UI is already flat and square, so the frame around it adds
// nothing: no padding, no border, no surface of its own. It is a bare viewport.
.iframe-container {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: var(--bg);
}

.sunshine-iframe {
  display: block;
  width: 100%;
  height: 100%;
  border: 0;
  background: var(--bg);
}

// A plain fill of the page ground, not a floating panel: no surface of its
// own, no border, no elevation. It exists only to hide a half-painted iframe.
.loading-overlay {
  position: absolute;
  inset: 0;
  background: var(--bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.loading-container {
  max-width: 420px;
  text-align: center;
}

// The one rotation the design system keeps: a genuine loading state. A 16px
// hairline ring - 1px stroke, muted, with a quarter-arc head. Nothing pulses
// and nothing is square.
.loading-spinner {
  display: block;
  width: 16px;
  height: 16px;
  margin: 0 auto 12px;
  color: var(--text-faint);
  animation: brutalist-spin 0.9s linear infinite;
}

.loading-spinner-track,
.loading-spinner-head {
  fill: none;
  stroke: currentColor;
  stroke-width: 1;
  vector-effect: non-scaling-stroke;
}

.loading-spinner-track {
  opacity: 0.35;
}

.loading-spinner-head {
  stroke-linecap: round;
}

.loading-text {
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  line-height: var(--line-height);

  p {
    margin: 6px 0;
    overflow-wrap: anywhere;
  }

  .proxy-health-error {
    color: var(--error);
  }
}

.proxy-health-retry {
  min-height: var(--row-height);
  margin-top: 10px;
  padding: 0 var(--pad-x);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
  font-family: var(--font-sans);
  font-size: var(--font-size);
  font-weight: 400;
  cursor: pointer;

  &:hover {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }
}
</style>

<style>
.paused-animations *,
.paused-animations *::before,
.paused-animations *::after {
  animation: none !important;
  transition: none !important;
}
</style>
