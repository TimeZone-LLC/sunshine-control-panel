<template>
  <div class="app-shell">
    <header class="app-bar">
      <div class="app-identity" aria-label="Sunshine">
        <span class="brand-mark" aria-hidden="true"><span></span></span>
        <span class="brand-name">Sunshine</span>
      </div>

      <div class="app-actions">
        <div class="host-summary" :data-state="hostState">
          <span class="host-state-mark" aria-hidden="true"></span>
          <span>{{ hostStatusLabel }}</span>
          <span v-if="activeSessions.length" class="host-session-count">
            {{ activeSessions.length }} {{ activeSessions.length === 1 ? 'session' : 'sessions' }}
          </span>
        </div>
      </div>
    </header>

    <nav class="primary-navigation" aria-label="Sunshine tools">
      <section
        v-for="section in sections"
        :key="section.id"
        class="primary-navigation-group"
        :aria-labelledby="`navigation-group-${section.id}`"
      >
        <h2 :id="`navigation-group-${section.id}`" class="primary-navigation-label">
          {{ section.label }}
        </h2>
        <button
          v-for="item in section.items"
          :key="item.id"
          type="button"
          class="primary-navigation-item"
          :data-active="activeView === item.id"
          :aria-current="activeView === item.id ? 'page' : undefined"
          :title="item.description"
          @click="selectView(item)"
        >
          {{ item.label }}
        </button>
      </section>
    </nav>

    <main class="workspace">
      <section v-if="activeDefinition.kind === 'web'" class="workspace-frame">
        <div
          v-if="frameLoading || hostState !== 'ready'"
          class="workspace-loading"
          role="status"
          aria-live="polite"
        >
          <div>{{ hostState === 'error' ? hostError : 'Connecting to the local host...' }}</div>
        </div>
        <iframe
          ref="hostFrame"
          :src="frameUrl"
          class="host-frame"
          title="Sunshine host configuration"
          allow="autoplay; clipboard-read; clipboard-write; fullscreen"
          @load="frameLoading = false"
        ></iframe>
      </section>

      <section v-else class="workspace-tool-surface">
        <div class="workspace-view">
          <header class="workspace-view-header">
            <div>
              <h1>{{ activeDefinition.label }}</h1>
              <p>{{ activeDefinition.description }}</p>
            </div>
          </header>
          <div class="workspace-tool-content">
            <component :is="activeDefinition.component" v-bind="activeDefinition.componentProps || {}" />
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup>
import { computed, defineAsyncComponent, markRaw, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { controllerMeta, sunshine, tools } from '@/tauri-adapter.js'
import { checkLocalProxyHealth } from '@/utils/proxyHealth.js'

const view = (loader) => markRaw(defineAsyncComponent(loader))

const WEB_VIEWS = {
  overview: { id: 'overview', label: 'Status', description: 'Host state and active streams', kind: 'web', path: '/' },
  applications: { id: 'applications', label: 'Apps', description: 'Published applications and commands', kind: 'web', path: '/apps' },
  pairing: { id: 'pairing', label: 'Pairing', description: 'Pair Moonlight clients with this host', kind: 'web', path: '/pin' },
  configuration: { id: 'configuration', label: 'Settings', description: 'Streaming, audio, input, and network settings', kind: 'web', path: '/config' },
}

const TOOL_VIEWS = {
  display: {
    id: 'display',
    label: 'Displays',
    description: 'Virtual display and HDR calibration',
    kind: 'component',
    component: view(() => import('@/components/VddSettings.vue')),
  },
  devices: {
    id: 'devices',
    label: 'Controllers',
    description: 'Controllers, USB forwarding, and input drivers',
    kind: 'component',
    component: view(() => import('@/components/controllersHub/ControllersHub.vue')),
    componentProps: {
      onOpenControllerMeta: () => controllerMeta.launch(),
      onOpenStylusInputProbe: () => tools.openNativeTool('alkaidlab.stylus'),
      onOpenHdrEnhanced: () => selectView(TOOL_VIEWS.enhancements),
    },
  },
  enhancements: {
    id: 'enhancements',
    label: 'Image',
    description: 'Image enhancement runtimes and live controls',
    kind: 'component',
    component: view(() => import('@/components/HdrEnhancedManager.vue')),
  },
  webstream: {
    id: 'webstream',
    label: 'Web access',
    description: 'Moonlight Web service and transport settings',
    kind: 'component',
    component: view(() => import('@/components/WebStreamSettings.vue')),
  },
  sharing: {
    id: 'sharing',
    label: 'File access',
    description: 'Read-only filesystem access for paired clients',
    kind: 'component',
    component: view(() => import('./views/FileSharingView.vue')),
  },
  utilities: {
    id: 'utilities',
    label: 'Maintenance',
    description: 'Diagnostics, recovery, and administrative actions',
    kind: 'component',
    component: view(() => import('./views/UtilityView.vue')),
  },
  logs: {
    id: 'logs',
    label: 'Logs',
    description: 'Local control-panel diagnostics',
    kind: 'component',
    component: view(() => import('@/console/LogConsoleApp.vue')),
  },
}

const SESSION_VIEWS = {
  performance: {
    id: 'performance',
    label: 'Metrics',
    description: 'Live host latency and pipeline metrics',
    kind: 'component',
    component: view(() => import('@/tool-window/tools/HostPerformanceTool.vue')),
    componentProps: { embedded: true },
  },
  bitrate: {
    id: 'bitrate',
    label: 'Bitrate',
    description: 'Inspect and change active-session bitrate',
    kind: 'component',
    component: view(() => import('@/tool-window/tools/BitrateTool.vue')),
    componentProps: { embedded: true },
  },
  rtss: {
    id: 'rtss',
    label: 'RTSS',
    description: 'Framerate, OSD, and HWiNFO controls',
    kind: 'component',
    component: view(() => import('@/tool-window/tools/RtssOsdTool.vue')),
    componentProps: { embedded: true },
  },
  dpi: {
    id: 'dpi',
    label: 'Desktop scale',
    description: 'Adjust the Windows desktop scale',
    kind: 'component',
    component: view(() => import('@/tool-window/tools/DpiAdjusterTool.vue')),
    componentProps: { embedded: true },
  },
  shortcuts: {
    id: 'shortcuts',
    label: 'Shortcuts',
    description: 'Reference for remote desktop shortcuts',
    kind: 'component',
    component: view(() => import('@/tool-window/tools/ShortcutsTool.vue')),
    componentProps: { embedded: true },
  },
}

const sections = [
  {
    id: 'host',
    label: 'Host',
    description: 'Host state, published applications, client pairing, and streaming configuration.',
    items: Object.values(WEB_VIEWS),
  },
  {
    id: 'hardware',
    label: 'Hardware',
    description: 'Displays, controllers, image processing, and browser-based client access.',
    items: [TOOL_VIEWS.display, TOOL_VIEWS.devices, TOOL_VIEWS.enhancements, TOOL_VIEWS.webstream],
  },
  {
    id: 'session',
    label: 'Session',
    description: 'Live stream inspection and controls that apply while a client is connected.',
    items: [SESSION_VIEWS.performance, SESSION_VIEWS.bitrate, SESSION_VIEWS.rtss, SESSION_VIEWS.dpi],
  },
  {
    id: 'system',
    label: 'System',
    description: 'Local diagnostics, filesystem access, logs, and remote-desktop references.',
    items: [TOOL_VIEWS.utilities, TOOL_VIEWS.sharing, TOOL_VIEWS.logs, SESSION_VIEWS.shortcuts],
  },
]

const definitions = Object.fromEntries(sections.flatMap((section) => section.items.map((item) => [item.id, item])))
const activeView = ref('overview')
const activeDefinition = computed(() => definitions[activeView.value] || WEB_VIEWS.overview)
const activeSessions = ref([])
const proxyBase = ref('')
const frameUrl = ref('about:blank')
const frameLoading = ref(true)
const hostState = ref('connecting')
const hostError = ref('The local Sunshine host is unavailable.')
const hostFrame = ref(null)
const nativeRuntime = '__TAURI_INTERNALS__' in window || '__TAURI__' in window
let pollTimer = null
let unlisteners = []

const hostStatusLabel = computed(() => {
  if (hostState.value === 'ready') return activeSessions.value.length ? 'Streaming' : 'Ready'
  if (hostState.value === 'error') return 'Unavailable'
  return 'Connecting'
})

function routeToFrame(path) {
  if (!proxyBase.value) return
  frameLoading.value = true
  frameUrl.value = proxyBase.value + path
}

function selectView(item) {
  activeView.value = item.id
  if (item.kind === 'web') routeToFrame(item.path)
}

function handleHostFrameMessage(event) {
  if (
    event.source !== hostFrame.value?.contentWindow ||
    event.data?.type !== 'native-navigation-context-request' ||
    event.data?.source !== 'sunshine-webui'
  ) return

  try {
    const trustedOrigin = new URL(proxyBase.value).origin
    if (event.origin !== trustedOrigin) return
    event.source.postMessage(
      { type: 'native-navigation-context', source: 'sunshine-control-panel', unified: true },
      trustedOrigin,
    )
  } catch {
    // Ignore requests until the local proxy has a valid URL.
  }
}

async function refreshStatus() {
  try {
    activeSessions.value = await sunshine.getActiveSessions()
  } catch {
    activeSessions.value = []
  }
}

async function initialiseHost() {
  hostState.value = 'connecting'
  try {
    await invoke('wait_for_proxy_ready')
    proxyBase.value = await sunshine.getProxyUrl()
    const healthCheck = await invoke('get_proxy_health_check')
    if (!checkLocalProxyHealth(healthCheck)) throw new Error('The local proxy health check failed.')
    await sunshine.refreshTarget()
    hostState.value = 'ready'
    routeToFrame(activeDefinition.value.path || '/')
    await refreshStatus()
  } catch (error) {
    hostState.value = 'error'
    hostError.value = error instanceof Error ? error.message : String(error)
    frameUrl.value = 'about:blank'
  } finally {
    await invoke('main_panel_ready').catch(() => {})
  }
}

async function listenForNativeNavigation() {
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    const currentWindow = getCurrentWebviewWindow()
    unlisteners = await Promise.all([
      currentWindow.listen('open-vdd-settings', () => selectView(TOOL_VIEWS.display)),
      currentWindow.listen('open-web-stream', () => selectView(WEB_VIEWS.configuration)),
      currentWindow.listen('open-welcome', () => selectView(WEB_VIEWS.overview)),
      currentWindow.listen('open-host-performance', () => selectView(SESSION_VIEWS.performance)),
      currentWindow.listen('open-rtss-controls', () => selectView(SESSION_VIEWS.rtss)),
    ])
  } catch {
    unlisteners = []
  }
}

onMounted(async () => {
  window.addEventListener('message', handleHostFrameMessage)
  if (!nativeRuntime) {
    hostState.value = 'error'
    hostError.value = 'Native host APIs are unavailable in browser preview.'
    frameLoading.value = false
    return
  }
  invoke('main_panel_loading').catch(() => {})
  await Promise.all([initialiseHost(), listenForNativeNavigation()])
  pollTimer = window.setInterval(refreshStatus, 5000)
})

onBeforeUnmount(() => {
  window.removeEventListener('message', handleHostFrameMessage)
  if (pollTimer) window.clearInterval(pollTimer)
  for (const unlisten of unlisteners) unlisten?.()
})
</script>
