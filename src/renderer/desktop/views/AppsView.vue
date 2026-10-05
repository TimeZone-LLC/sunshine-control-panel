<template>
  <div class="apps-view">
    <AppToolbar
      :filterTabs="filterTabs"
      :activeFilter="activeFilter"
      :searchQuery="searchQuery"
      :sortLabel="sortLabel"
      :gridSize="gridSize"
      :viewMode="viewMode"
      :totalCount="displayApps.length"
      @update:activeFilter="activeFilter = $event"
      @update:searchQuery="searchQuery = $event"
      @cycleSortMode="cycleSortMode"
      @cycleGridSize="cycleGridSize"
      @toggleViewMode="viewMode = viewMode === 'grid' ? 'list' : 'grid'"
    />

    <AppRecentStrip
      v-if="recentApps.length > 0 && !searchQuery && activeFilter === 'all'"
      :apps="recentApps"
      :getAppImageUrl="getAppImageUrl"
      :handleImageError="handleImageError"
      @launch="launchApp"
      @contextmenu="openContextMenu"
    />

    <!-- Loading state -->
    <div v-if="loading" class="apps-loading">
      <div class="loading-spinner"></div>
      <span>{{ t.apps.loading }}</span>
    </div>

    <!-- Load failure state -->
    <div v-else-if="loadFailed" class="apps-empty apps-error" role="alert">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="9"/><path d="M12 7v6"/><path d="M12 17h.01"/>
      </svg>
      <p>{{ t.apps.loadFailed }}</p>
      <button class="retry-btn" type="button" @click="loadApps">{{ t.apps.retry }}</button>
    </div>

    <!-- Empty state -->
    <div v-else-if="displayApps.length === 0 && !loading" class="apps-empty">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="2" y="2" width="8" height="8"/><rect x="14" y="2" width="8" height="8"/>
        <rect x="2" y="14" width="8" height="8"/><rect x="14" y="14" width="8" height="8"/>
      </svg>
      <p v-if="searchQuery">{{ t.apps.searchNoMatch }}</p>
      <p v-else-if="activeFilter === 'favorites'">{{ t.apps.noFavorites }}</p>
      <p v-else>{{ t.apps.noApps }}</p>
      <span class="empty-hint" v-if="!searchQuery && activeFilter === 'all'">{{ t.apps.addHint }}</span>
    </div>

    <AppGridView
      v-else-if="viewMode === 'grid'"
      :apps="displayApps"
      :gridSize="gridSize"
      :launchingApp="launchingApp"
      :isFavorite="isFavorite"
      :getAppImageUrl="getAppImageUrl"
      :handleImageError="handleImageError"
      :helperIds="getActiveHelperIds"
      :stats="gameStats"
      @launch="launchApp"
      @contextmenu="openContextMenu"
      @toggleFavorite="toggleFavorite"
    />

    <AppListView
      v-else
      :apps="displayApps"
      :launchingApp="launchingApp"
      :isFavorite="isFavorite"
      :getAppImageUrl="getAppImageUrl"
      :handleImageError="handleImageError"
      :stats="gameStats"
      @launch="launchApp"
      @contextmenu="openContextMenu"
    />

    <AppContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :isFavorited="isFavorite(contextMenu.app?.name)"
      :hasCmd="!!contextMenu.app?.cmd"
      :hasWorkingDir="!!contextMenu.app?.['working-dir']"
      @launch="ctxLaunch"
      @toggleFavorite="ctxToggleFavorite"
      @copyCmd="ctxCopyCmd"
      @openDir="ctxOpenDir"
      @configHelpers="ctxConfigHelpers"
      @updateCover="ctxUpdateCover"
      @close="closeContextMenu"
    />

    <LaunchHelperPanel
      :open="helperPanel.open"
      :appName="helperPanel.appName"
      :app="helperPanel.app"
      :proxyUrl="proxyUrl"
      @close="helperPanel.open = false"
      @saved="loadApps"
    />

    <CoverPickerModal
      :open="coverPicker.open"
      :appName="coverPicker.appName"
      :proxyUrl="proxyUrl"
      @close="coverPicker.open = false"
      @updated="onCoverUpdated"
    />

    <!-- Launch error notice -->
    <Transition name="toast">
      <div v-if="launchError" class="launch-error-toast">
        <span class="toast-icon">!</span>
        <span class="toast-msg">{{ launchError }}</span>
        <button class="toast-close" @click="launchError = ''">&times;</button>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useApps } from '../composables/useApps'
import { useLaunchHelpers } from '../composables/useLaunchHelpers'
import { tauriInvoke } from '../composables/useTauri'
import { gamepadConnected, onGamepadAction } from '../composables/useGamepad.js'
import { clearViewActions, setViewActions } from '../composables/useGamepadActions.js'
import { editInputWithOsk } from '../composables/useOsk.js'
import { focusElement } from '../composables/useFocusNav.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()
import AppToolbar from '../components/AppToolbar.vue'
import AppRecentStrip from '../components/AppRecentStrip.vue'
import AppGridView from '../components/AppGridView.vue'
import AppListView from '../components/AppListView.vue'
import AppContextMenu from '../components/AppContextMenu.vue'
import LaunchHelperPanel from '../components/LaunchHelperPanel.vue'
import CoverPickerModal from '../components/CoverPickerModal.vue'

const {
  loading,
  loadFailed,
  searchQuery,
  launchingApp,
  launchError,
  viewMode,
  gridSize,
  activeFilter,
  filterTabs,
  sortLabel,
  proxyUrl,
  recentApps,
  displayApps,
  gameStats,
  isFavorite,
  toggleFavorite,
  cycleSortMode,
  cycleGridSize,
  getAppImageUrl,
  handleImageError,
  invalidateAppImage,
  loadApps,
  launchApp,
} = useApps()

// Context menu
const contextMenu = ref({ visible: false, x: 0, y: 0, app: null })

// Launch helper panel
const helperPanel = ref({ open: false, appName: '', app: null })
const { getActiveHelperIds, helperPanelOpen } = useLaunchHelpers(t)

// Mirror the panel state into the shared composable so the DesktopApp back
// action can use it.
watch(() => helperPanel.value.open, (v) => { helperPanelOpen.value = v })
watch(helperPanelOpen, (v) => { if (!v) helperPanel.value.open = false })

function openContextMenu(event, app) {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    app,
  }
}

function closeContextMenu() {
  contextMenu.value.visible = false
}

function ctxLaunch() {
  if (contextMenu.value.app) launchApp(contextMenu.value.app)
  closeContextMenu()
}

function ctxToggleFavorite() {
  if (contextMenu.value.app) toggleFavorite(contextMenu.value.app.name)
  closeContextMenu()
}

function ctxConfigHelpers() {
  if (contextMenu.value.app) {
    helperPanel.value = {
      open: true,
      appName: contextMenu.value.app.name,
      app: contextMenu.value.app,
    }
  }
  closeContextMenu()
}

async function ctxCopyCmd() {
  if (contextMenu.value.app?.cmd) {
    try {
      await navigator.clipboard.writeText(contextMenu.value.app.cmd)
    } catch (e) {
      console.error('Copy failed:', e)
    }
  }
  closeContextMenu()
}

async function ctxOpenDir() {
  const dir = contextMenu.value.app?.['working-dir']
  if (dir) {
    try {
      const opened = await tauriInvoke('open_local_path', {
        path: dir,
        appName: contextMenu.value.app.name,
      })
      if (!opened) throw new Error('Open directory returned false')
    } catch (e) {
      console.error('Open dir failed:', e)
      launchError.value = `${t.value.appContext.openDirectory}: ${e}`
      setTimeout(() => { launchError.value = '' }, 4000)
    }
  }
  closeContextMenu()
}

// Cover picker modal
const coverPicker = ref({ open: false, appName: '' })

async function ctxUpdateCover() {
  const app = contextMenu.value.app
  closeContextMenu()
  if (!app) return

  coverPicker.value = { open: true, appName: app.name }
}

function onCoverUpdated(appName) {
  invalidateAppImage(appName)
}

function onDocClick() {
  closeContextMenu()
}

// === 手柄动作 ===

/** 当前焦点落在哪个应用卡片上。网格、列表、最近条都带 data-app-name。 */
function focusedApp() {
  const host = document.activeElement?.closest?.('[data-app-name]')
  if (!host) return null
  const name = host.dataset.appName
  return {
    element: host,
    app: displayApps.value.find((entry) => entry.name === name)
      || recentApps.value.find((entry) => entry.name === name)
      || null,
  }
}

/** Y 键：在焦点卡片下方打开上下文菜单，不再依赖鼠标坐标。 */
function openContextMenuForFocus() {
  const focused = focusedApp()
  if (!focused?.app) return false
  const rect = focused.element.getBoundingClientRect()
  contextMenu.value = {
    visible: true,
    x: rect.left,
    y: rect.bottom + 6,
    app: focused.app,
  }
  return true
}

function toggleFavoriteForFocus() {
  const focused = focusedApp()
  if (!focused?.app) return false
  toggleFavorite(focused.app.name)
  return true
}

async function focusSearchWithOsk() {
  const input = document.querySelector('[data-focus-key="apps-search"]')
  if (!input) return
  focusElement(input)
  await editInputWithOsk(input, { title: t.value.apps.searchPlaceholder })
}

function handleGamepadAction(action) {
  // B 键：从内到外逐层关闭本视图拥有的弹层
  if (action === 'back' || action === 'backRoot') {
    if (contextMenu.value.visible) closeContextMenu()
    else if (coverPicker.value.open) coverPicker.value.open = false
    else if (helperPanel.value.open) helperPanel.value.open = false
    return
  }

  // 弹层打开时其余按键留给弹层自己处理
  if (contextMenu.value.visible || coverPicker.value.open || helperPanel.value.open) return

  switch (action) {
    case 'menu':
      openContextMenuForFocus()
      break
    case 'favorite':
      toggleFavoriteForFocus()
      break
    case 'search':
      focusSearchWithOsk()
      break
  }
}

/**
 * 首屏把焦点放在第一张卡片上（有「最近启动」时就是上次玩的那个），
 * 这样手柄第一次按方向键是从有意义的位置开始移动，而不是跳到元素 0。
 * 没插手柄时不做，避免干扰鼠标用户。
 */
async function focusFirstApp() {
  if (!gamepadConnected.value) return
  await nextTick()
  if (document.activeElement && document.activeElement !== document.body) return
  const first = document.querySelector('.apps-view [data-app-name]')
  if (first) focusElement(first)
}

let disposeGamepadActions = null

onMounted(async () => {
  document.addEventListener('click', onDocClick)
  disposeGamepadActions = onGamepadAction(handleGamepadAction)
  setViewActions(['search', 'favorite', 'menu'])
  await loadApps()
  await focusFirstApp()
})

onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
  clearViewActions()
  disposeGamepadActions?.()
})
</script>

<style lang="less" scoped>
@import "@/components/spinner.less";

.apps-view {
  max-width: 1600px;
}

// === Loading and empty states ===
.apps-loading {
  display: flex;
  align-items: center;
  padding: 24px 0;
  gap: 8px;
  color: var(--text-dim);
  font-size: var(--font-size-sm);

  .loading-spinner {
    .gloss-spinner();
  }
}

.apps-empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding: 24px 0;
  gap: 6px;
  color: var(--text-dim);

  // No illustration in an empty state; the words are enough.
  .empty-icon { display: none; }

  p { margin: 0; font-size: var(--font-size); }

  .empty-hint { font-size: var(--font-size-sm); color: var(--text-faint); }

  .retry-btn {
    margin-top: 4px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    font-size: var(--font-size-sm);
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
}

.apps-error {
  color: var(--error);

  .retry-btn {
    color: var(--text);
  }
}

// === Launch error notice ===
.launch-error-toast {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 10px;
  background: var(--bg-panel);
  border-top: 1px solid var(--error);
  color: var(--error);
  font-size: var(--font-size-sm);

  .toast-icon {
    font-family: var(--font-mono);
    font-weight: 600;
    flex-shrink: 0;
  }

  .toast-msg {
    flex: 1;
    max-width: 900px;
    word-break: break-all;
    white-space: pre-line;
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    line-height: var(--line-height);
    color: var(--text);
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: var(--font-size);
    line-height: 1;
    padding: 0 4px;
    flex-shrink: 0;

    &:hover { color: var(--text); }
  }
}

.toast-enter-active,
.toast-leave-active {
  transition: none;
}
</style>
