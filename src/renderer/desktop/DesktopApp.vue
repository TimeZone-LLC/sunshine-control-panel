<template>
  <DesktopWindow
    :title="appTitle"
    :icon="sunshineIcon"
    :has-sidebar="true"
    :show-title-bar="false"
    :class="{ 'gamepad-active': gamepadActive, 'cursor-mode': cursorEnabled }"
  >
    <template #sidebar>
      <DesktopSidebar
        :items="navItems"
        :bottom-items="bottomNavItems"
        :active-item="activeNav"
        @item-click="handleNavClick"
        @update:active-item="activeNav = $event"
      />
    </template>

    <template #default>
      <component :is="currentView" @openThemeEditor="themeEditorOpen = true" />
    </template>

    <template #footer>
      <!-- 两者都是底部居中的浮层，必须放在同一个 flex 列里堆叠。
           各自绝对定位时会完全重叠，而运行条的 z-index 更高，会把按键提示整条盖掉
           —— 而「游戏在跑 + 手柄模式」正是沙发场景下最常见的状态。 -->
      <div class="desktop-footer-dock">
        <RunningGameBar
          :game="runningGame"
          :elapsed="elapsedSeconds"
          :stat="runningStat"
          @resume="resumeGame"
          @stop="confirmStopGame"
        />

        <GamepadLegend v-if="gamepadActive" :cursor-mode="cursorEnabled" />
      </div>
    </template>
  </DesktopWindow>

  <ThemeEditor
    :open="themeEditorOpen"
    :vars="themeVars"
    :activePreset="activePreset"
    :presets="presets"
    @close="themeEditorOpen = false"
    @setVar="setVar"
    @applyPreset="applyPreset"
    @export="handleThemeExport"
    @import="handleThemeImport"
  />

  <OnScreenKeyboard />

  <ConfirmDialog />

  <LaunchOverlay
    :state="launchState"
    @dismiss="dismissLaunchState"
    @stop="confirmStopGame"
  />

  <div
    v-if="cursorEnabled"
    class="gamepad-cursor"
    :style="{ transform: `translate(${cursorX}px, ${cursorY}px)` }"
    aria-hidden="true"
  >
    <svg viewBox="0 0 24 24">
      <path d="M5 2 L5 19 L10 14.5 L13 21.5 L16 20 L13 13.5 L19.5 13 Z" />
    </svg>
  </div>

  <SplashScreen
    :visible="showSplash"
    @done="showSplash = false"
  />
</template>

<script setup>
import { defineAsyncComponent, ref, computed, nextTick, onMounted, onUnmounted, watch } from 'vue'

// Desktop UI components
import DesktopWindow from './components/DesktopWindow.vue'
import DesktopSidebar from './components/DesktopSidebar.vue'
import ThemeEditor from './components/ThemeEditor.vue'
import SplashScreen from './components/SplashScreen.vue'
import OnScreenKeyboard from './components/OnScreenKeyboard.vue'
import ConfirmDialog from './components/ConfirmDialog.vue'
import LaunchOverlay from './components/LaunchOverlay.vue'
import RunningGameBar from './components/RunningGameBar.vue'
import GamepadLegend from './components/GamepadLegend.vue'

// Gamepad, focus and on-screen-keyboard support
import { emitGamepadAction, gamepadActive, gamepadConnected, useGamepad } from './composables/useGamepad.js'
import {
  collectFocusables,
  confirmFocused,
  focusElement,
  focusScopeStack,
  navigateFocus,
  rememberFocus,
  restoreFocus,
  scrollActiveScope,
} from './composables/useFocusNav.js'
import {
  cancelOsk,
  editInputWithOsk,
  isTextEntryElement,
  openOsk,
  oskOpen,
} from './composables/useOsk.js'
import { confirmOpen, rejectConfirm, requestConfirm } from './composables/useConfirm.js'
import {
  clickAtCursor,
  contextMenuAtCursor,
  cursorEnabled,
  cursorX,
  cursorY,
  elementAtCursor,
  moveCursor,
  scrollAtCursor,
  setCursorEnabled,
} from './composables/useGamepadCursor.js'
import { bigScreenSettings } from './composables/useBigScreenSettings.js'
import {
  dismissLaunchState,
  elapsedSeconds,
  initGameSession,
  disposeGameSession,
  launchState,
  runningGame,
  statFor,
  stopTrackedGame,
} from './composables/useGameSession.js'
import { useTheme } from './composables/useTheme.js'
import { useLaunchHelpers } from './composables/useLaunchHelpers.js'
import {
  DESKTOP_SETTINGS_UPDATED,
  desktopSettings,
  loadDesktopSettings,
  requestNotificationPermission,
} from './composables/useDesktopSettings.js'
import { useI18n } from './i18n/index.js'

// Icon components
import IconApps from './icons/IconApps.vue'
import IconDashboard from './icons/IconDashboard.vue'
import IconDevices from './icons/IconDevices.vue'
import IconStream from './icons/IconStream.vue'
import IconTools from './icons/IconTools.vue'
import IconSettings from './icons/IconSettings.vue'
import IconPower from './icons/IconPower.vue'
import IconPalette from './icons/IconPalette.vue'

// View components
const AppsView = defineAsyncComponent(() => import('./views/AppsView.vue'))
const DashboardView = defineAsyncComponent(() => import('./views/DashboardView.vue'))
const DevicesView = defineAsyncComponent(() => import('./views/DevicesView.vue'))
const StreamView = defineAsyncComponent(() => import('./views/StreamView.vue'))
const ToolsView = defineAsyncComponent(() => import('./views/ToolsView.vue'))
const SettingsView = defineAsyncComponent(() => import('./views/SettingsView.vue'))

// Icon assets
import sunshineIcon from '../../assets/sunshine.ico'

// i18n
const { t } = useI18n()

// Application configuration
const appTitle = 'FOUNDATION DESKTOP'

// Theme
const { themeVars, activePreset, presets, setVar, applyPreset, exportTheme, importTheme } = useTheme()
const themeEditorOpen = ref(false)
const showSplash = ref(true)
const { helperPanelOpen } = useLaunchHelpers(t)

function handleThemeExport() {
  const json = exportTheme()
  navigator.clipboard.writeText(json).catch(() => {})
}

async function handleThemeImport() {
  // 用屏幕键盘而不是 prompt()：原生弹窗接不到手柄输入
  const json = await openOsk({ title: `${t.value.nav.theme} JSON` })
  if (json) importTheme(json)
}

// Navigation state; the app library is the landing view.
const activeNav = ref('apps')

// Primary navigation
const navItems = computed(() => [
  { id: 'apps', label: t.value.nav.apps, icon: IconApps, disabled: false },
  { id: 'dashboard', label: t.value.nav.dashboard, icon: IconDashboard, disabled: false },
  { id: 'devices', label: t.value.nav.devices, icon: IconDevices, disabled: false },
  { id: 'stream', label: t.value.nav.stream, icon: IconStream, disabled: false },
  { id: 'tools', label: t.value.nav.tools, icon: IconTools, disabled: false },
])

// Secondary navigation
const bottomNavItems = computed(() => [
  { id: 'theme', label: t.value.nav.theme, icon: IconPalette, disabled: false },
  { id: 'settings', label: t.value.nav.settings, icon: IconSettings, disabled: false },
  { id: 'exit', label: t.value.nav.exit, icon: IconPower, disabled: false },
])

// View map
const viewMap = {
  apps: AppsView,
  dashboard: DashboardView,
  devices: DevicesView,
  stream: StreamView,
  tools: ToolsView,
  settings: SettingsView,
}

const currentView = computed(() => viewMap[activeNav.value] || DashboardView)

const runningStat = computed(() => statFor(runningGame.value?.appName))

// Tauri invoke
const invoke = ref(null)
let settingsListener = null
let desktopNavigateUnlisten = null

function applyRuntimeSettings() {
  document.documentElement.dataset.fdDevMode = desktopSettings.value.devMode ? 'true' : 'false'
  if (desktopSettings.value.notifications) {
    requestNotificationPermission(desktopSettings.value).catch(() => {})
  }
}

async function setupDesktopNavigationEvents() {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    desktopNavigateUnlisten = await listen('desktop-navigate', (event) => {
      const target = String(event.payload || '')
      if (viewMap[target]) {
        activeNav.value = target
      }
    })
  } catch {
    // event API unavailable outside Tauri
  }
}

onMounted(async () => {
  try {
    const tauri = await import('@tauri-apps/api/core')
    invoke.value = tauri.invoke
  } catch (e) {
    console.log('Tauri invoke not available:', e)
  }
  await loadDesktopSettings()
  applyRuntimeSettings()
  settingsListener = () => applyRuntimeSettings()
  window.addEventListener(DESKTOP_SETTINGS_UPDATED, settingsListener)
  await setupDesktopNavigationEvents()
  await initGameSession()
})

onUnmounted(() => {
  if (settingsListener) window.removeEventListener(DESKTOP_SETTINGS_UPDATED, settingsListener)
  if (desktopNavigateUnlisten) desktopNavigateUnlisten()
  disposeGameSession()
})

// Navigation click handling
async function handleNavClick(item) {
  if (item.disabled) return
  if (item.id === 'exit') {
    if (invoke.value) {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window')
        await getCurrentWindow().close()
      } catch (e) {
        window.close()
      }
    }
    return
  }
  if (item.id === 'theme') {
    themeEditorOpen.value = !themeEditorOpen.value
    return
  }
  activeNav.value = item.id
}

// Running game session
async function resumeGame() {
  try {
    const { invoke: tauriApiInvoke } = await import('@tauri-apps/api/core')
    await tauriApiInvoke('focus_running_game')
  } catch {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().minimize()
    } catch {
      // Tauri APIs are unavailable in browser tests.
    }
  }
}

async function confirmStopGame() {
  const name = runningGame.value?.appName || launchState.value?.message || ''
  if (!name) return
  const accepted = await requestConfirm({
    title: t.value.gameSession.stopRunning,
    message: t.value.gameSession.stopConfirm.replace('{name}', name),
    confirmLabel: t.value.gameSession.stopRunning,
    danger: true,
  })
  if (accepted) stopTrackedGame()
}

// Remember focus independently for each asynchronously loaded view.
async function restoreFocusForView(viewKey) {
  const maxAttempts = 12
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    await nextTick()
    const main = document.querySelector('.desktop-window-main')
    if (main && collectFocusables(main).length > 0) break
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
  restoreFocus(viewKey)
}

watch(activeNav, async (next, previous) => {
  if (!gamepadConnected.value) return
  if (previous) rememberFocus(previous)
  await restoreFocusForView(next)
})

function hasOverlay() {
  return (
    oskOpen.value ||
    confirmOpen.value ||
    focusScopeStack.value.length > 0 ||
    themeEditorOpen.value ||
    helperPanelOpen.value
  )
}

const navigableIds = computed(() =>
  [...navItems.value, ...bottomNavItems.value]
    .filter((item) => !item.disabled && viewMap[item.id])
    .map((item) => item.id)
)

function switchNav(step) {
  const ids = navigableIds.value
  if (ids.length === 0) return
  const index = ids.indexOf(activeNav.value)
  if (index === -1) {
    activeNav.value = ids[0]
    return
  }
  const next = index + step
  if (next >= 0 && next < ids.length) activeNav.value = ids[next]
}

function handleConfirm() {
  if (cursorEnabled.value) {
    const target = elementAtCursor()
    if (bigScreenSettings.value.oskAutoOpen && isTextEntryElement(target)) {
      target.focus({ preventScroll: true })
      editInputWithOsk(target)
      return
    }
    clickAtCursor()
    return
  }

  const focused = document.activeElement
  if (bigScreenSettings.value.oskAutoOpen && isTextEntryElement(focused)) {
    editInputWithOsk(focused)
    return
  }
  confirmFocused()
}

async function requestExitShell() {
  const accepted = await requestConfirm({
    title: t.value.gamepadLegend.exitTitle,
    message: t.value.gamepadLegend.exitMessage,
    confirmLabel: t.value.gamepadLegend.exitConfirm,
    danger: false,
  })
  if (!accepted) return
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch {
    window.close()
  }
}

function handleBack(deep) {
  if (oskOpen.value) {
    cancelOsk()
    return
  }
  if (confirmOpen.value) {
    rejectConfirm()
    return
  }
  if (launchState.value && launchState.value.status !== 'launching') {
    dismissLaunchState()
    return
  }
  if (focusScopeStack.value.length > 0 || themeEditorOpen.value || helperPanelOpen.value) {
    if (themeEditorOpen.value) {
      themeEditorOpen.value = false
      return
    }
    if (helperPanelOpen.value) {
      helperPanelOpen.value = false
      return
    }
    emitGamepadAction(deep ? 'backRoot' : 'back')
    return
  }
  if (cursorEnabled.value) {
    setCursorEnabled(false)
    return
  }
  if (deep || activeNav.value !== 'apps') {
    activeNav.value = 'apps'
    return
  }
  requestExitShell()
}

function focusSidebar() {
  const active = document.querySelector('.desktop-sidebar .nav-item.active')
  focusElement(active || document.querySelector('.desktop-sidebar .nav-item'))
}

function handleAction(action) {
  switch (action) {
    case 'confirm':
      handleConfirm()
      return
    case 'back':
      handleBack(false)
      return
    case 'backRoot':
      handleBack(true)
      return
    case 'cursorToggle':
      if (bigScreenSettings.value.gamepadCursorEnabled) {
        setCursorEnabled(!cursorEnabled.value)
      }
      return
  }

  if (cursorEnabled.value && action === 'menu') {
    contextMenuAtCursor()
    return
  }
  if (hasOverlay()) {
    emitGamepadAction(action)
    return
  }
  switch (action) {
    case 'tabPrev':
      switchNav(-1)
      return
    case 'tabNext':
      switchNav(1)
      return
    case 'home':
      activeNav.value = 'apps'
      return
    default:
      emitGamepadAction(action)
  }
}

useGamepad({
  onNavigate(direction) {
    navigateFocus(direction)
  },
  onScroll(deltaY, deltaX) {
    if (cursorEnabled.value) scrollAtCursor(deltaY, deltaX)
    else scrollActiveScope(deltaY, deltaX)
  },
  onCursorMove(dx, dy) {
    moveCursor(dx, dy)
  },
  onAction: handleAction,
  isCursorMode: () => cursorEnabled.value,
  enabled: () => !runningGame.value || document.hasFocus(),
})

watch(
  gamepadActive,
  (active) => {
    document.documentElement.dataset.fdGamepad = active ? 'true' : 'false'
  },
  { immediate: true }
)

watch(
  gamepadActive,
  (active) => {
    if (!active) {
      if (cursorEnabled.value) setCursorEnabled(false)
      return
    }
    if (!document.activeElement || document.activeElement === document.body) focusSidebar()
  },
  { flush: 'sync' }
)
</script>

<style lang="less" scoped>
.desktop-footer-dock {
  position: absolute;
  left: 140px;
  right: 0;
  bottom: 12px;
  z-index: 900;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  pointer-events: none;

  > * {
    pointer-events: auto;
  }
}

.gamepad-cursor {
  position: fixed;
  top: 0;
  left: 0;
  width: 26px;
  height: 26px;
  z-index: 30000;
  pointer-events: none;
  will-change: transform;

  svg {
    width: 100%;
    height: 100%;
    fill: var(--fd-text-primary);
    stroke: var(--fd-bg-deep);
    stroke-width: 1.4;
  }
}
</style>
