<template>
  <div
    id="toolbar-container"
    :class="{ 'menu-open': menuVisible }"
    @click.self="handleOutsideClick"
    @pointerdown.self="onContainerDragStart"
    @contextmenu.prevent
  >
    <!-- Expanded tool menu -->
    <div v-if="menuVisible" class="toolbar-menu" @click.stop>
      <div
        class="menu-header"
        @pointerdown="onDragStart"
        @click.stop="onHeaderClick"
      >
        {{ t.toolbar.title }}
      </div>
      <button
        v-for="item in menuItems"
        :key="item.id"
        type="button"
        class="menu-item"
        :class="{ danger: item.danger }"
        :title="item.label"
        @click="handleMenuItem(item.id)"
      >
        <span class="menu-icon" v-html="item.icon"></span>
        <span class="menu-label">{{ item.label }}</span>
      </button>
    </div>

    <!-- Collapsed launcher -->
    <button
      v-else
      type="button"
      class="toolbar-icon"
      :aria-label="t.toolbar.title"
      @pointerdown="onDragStart"
      @click.stop="onIconClick"
      @contextmenu.prevent="toggleMenu"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          fill="currentColor"
          d="M3 3h8v8H3V3zm10 0h8v8h-8V3zM3 13h8v8H3v-8zm10 0h8v8h-8v-8z"
        />
      </svg>
    </button>
  </div>
</template>

<script setup>
import { ref, computed, onUnmounted, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cursorPosition } from '@tauri-apps/api/window'
import { PhysicalPosition } from '@tauri-apps/api/dpi'
import { useI18n } from '../desktop/i18n/index.js'

const { t } = useI18n()

const menuVisible = ref(false)

const menuItems = computed(() => [
  {
    id: 'main',
    label: t.value.toolbar.controlPanel,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>',
  },
  {
    id: 'vdd',
    label: t.value.toolbar.virtualDisplay,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M20 18c1.1 0 1.99-.9 1.99-2L22 6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2H0v2h24v-2h-4zM4 6h16v10H4V6z"/></svg>',
  },
  {
    id: 'dpi',
    label: t.value.toolbar.adjustDpi,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14zM5 7h5v5H5zm6 0h8v2h-8zm0 3h8v2h-8zM5 13h5v5H5zm6 0h8v2h-8z"/></svg>',
  },
  {
    id: 'bitrate',
    label: t.value.toolbar.bitrateAdjust,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M9 3L5 6.99h3V14h2V6.99h3L9 3zm7 14.01V10h-2v7.01h-3L15 21l4-3.99h-3z"/></svg>',
  },
  {
    id: 'shortcuts',
    label: t.value.toolbar.shortcutGuide,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z"/></svg>',
  },
  {
    id: 'toolbar_settings',
    label: t.value.toolbar.toolbarSettings,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94 0 .31.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>',
  },
  {
    id: 'close',
    label: t.value.toolbar.close,
    icon: '<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>',
    danger: true,
  },
])

const toggleMenu = () => {
  menuVisible.value = !menuVisible.value
}

const handleOutsideClick = () => {
  // Clicking the transparent margin closes the menu.
  if (menuVisible.value) {
    menuVisible.value = false
  }
}

const onHeaderClick = () => {
  if (!hasMoved) menuVisible.value = false
}

const handleMenuItem = async (action) => {
  menuVisible.value = false

  try {
    await invoke('handle_toolbar_menu_action', { action })
  } catch (error) {
    console.error('Menu action failed:', error)
  }
}

// Custom dragging. PointerEvent handles mouse and touch through one path,
// replacing data-tauri-drag-region.
const appWindow = getCurrentWindow()
let isDragging = false
let hasMoved = false
let dragPointerType = ''
let dragPointerId = null
let dragPointerTarget = null
let dragEnding = false
let dragDisposed = false
let dragGeneration = 0
let dragStartScreenX = 0
let dragStartScreenY = 0

let touchStartClientX = 0
let touchStartClientY = 0
let touchLatestClientX = 0
let touchLatestClientY = 0
let touchBasePhysicalX = Number.NaN
let touchBasePhysicalY = Number.NaN
let touchPendingPhysicalX = Number.NaN
let touchPendingPhysicalY = Number.NaN
let touchScaleFactor = 1
let touchScaleFactorVersion = 0
let touchInitialized = false
let touchPreparing = false
let touchPreparationPromise = null
let touchScaleRebasing = false
let touchScaleRebasePromise = null
let touchIsSettingPos = false
let touchFailed = false
let touchRafId = null
let unlistenDragScaleChanged = null
let dragScaleListenerPromise = null
let nativeDragPollTimer = null
let nativeDragPollResolve = null
let toolbarPositionSaveGeneration = 0
let toolbarPositionSavePromise = Promise.resolve()
const DRAG_THRESHOLD = 3
const NATIVE_DRAG_POLL_INTERVAL_MS = 32

const normalizeDragScaleFactor = (value) => (
  Number.isFinite(value) && value > 0 ? value : 1
)

const isCurrentDrag = (generation, pointerId) => (
  !dragDisposed &&
  generation === dragGeneration &&
  dragPointerId === pointerId
)

const removeDragListeners = () => {
  document.removeEventListener('pointermove', onDragMove)
  document.removeEventListener('pointerup', onDragEnd)
  document.removeEventListener('pointercancel', onDragEnd)
}

const releaseDragPointerCapture = () => {
  if (dragPointerTarget && dragPointerId !== null) {
    try {
      if (dragPointerTarget.hasPointerCapture(dragPointerId)) {
        dragPointerTarget.releasePointerCapture(dragPointerId)
      }
    } catch {}
  }
  dragPointerTarget = null
}

const cancelNativeDragPoll = () => {
  if (nativeDragPollTimer !== null) {
    clearTimeout(nativeDragPollTimer)
    nativeDragPollTimer = null
  }
  if (nativeDragPollResolve) {
    const resolve = nativeDragPollResolve
    nativeDragPollResolve = null
    resolve()
  }
}

const clearDragState = ({ preserveMoved = false } = {}) => {
  dragGeneration += 1
  if (touchRafId !== null) {
    cancelAnimationFrame(touchRafId)
    touchRafId = null
  }
  cancelNativeDragPoll()
  removeDragListeners()
  releaseDragPointerCapture()

  isDragging = false
  dragEnding = false
  dragPointerType = ''
  dragPointerId = null
  touchInitialized = false
  touchPreparing = false
  touchPreparationPromise = null
  touchScaleRebasing = false
  touchScaleRebasePromise = null
  touchIsSettingPos = false
  touchFailed = false
  touchBasePhysicalX = Number.NaN
  touchBasePhysicalY = Number.NaN
  touchPendingPhysicalX = Number.NaN
  touchPendingPhysicalY = Number.NaN
  if (!preserveMoved) {
    hasMoved = false
  }
}

const resetMovedAfterClick = () => {
  const generation = dragGeneration
  requestAnimationFrame(() => {
    if (generation === dragGeneration) {
      hasMoved = false
    }
  })
}

const finishDrag = (generation, pointerId, preserveMoved = false) => {
  if (!isCurrentDrag(generation, pointerId)) return

  clearDragState({ preserveMoved })
  if (preserveMoved) {
    resetMovedAfterClick()
  }
}

const commitTouchPosition = (physicalX, physicalY) => (
  appWindow.setPosition(new PhysicalPosition(
    Math.round(physicalX),
    Math.round(physicalY),
  ))
)

const queueToolbarPositionSave = (position) => {
  const generation = ++toolbarPositionSaveGeneration
  toolbarPositionSavePromise = toolbarPositionSavePromise
    .catch(() => {})
    .then(async () => {
      if (generation !== toolbarPositionSaveGeneration) return

      await invoke('save_toolbar_position', { x: position.x, y: position.y })
    })
    .catch(() => {})
}

const markTouchDragFailed = (generation, pointerId) => {
  if (!isCurrentDrag(generation, pointerId)) return

  touchFailed = true
  if (touchRafId !== null) {
    cancelAnimationFrame(touchRafId)
    touchRafId = null
  }
}

const waitForNativeDragPoll = () => new Promise((resolve) => {
  nativeDragPollResolve = resolve
  nativeDragPollTimer = setTimeout(() => {
    nativeDragPollTimer = null
    nativeDragPollResolve = null
    resolve()
  }, NATIVE_DRAG_POLL_INTERVAL_MS)
})

const finishNativeDrag = async (generation, pointerId) => {
  // startDragging() resolves after dispatching the native command, not when
  // the Windows move loop ends. Keep the session active until the button lifts.
  while (isCurrentDrag(generation, pointerId)) {
    const pressed = await invoke('is_primary_mouse_button_pressed')
    if (!isCurrentDrag(generation, pointerId)) return
    if (pressed !== true) break

    await waitForNativeDragPoll()
  }
  if (!isCurrentDrag(generation, pointerId)) return

  const finalPos = await appWindow.outerPosition()
  if (!isCurrentDrag(generation, pointerId)) return

  queueToolbarPositionSave(finalPos)
  finishDrag(generation, pointerId, true)
}

const rebaseTouchDragForScaleChange = (scaleFactor) => {
  touchScaleFactorVersion += 1
  touchScaleFactor = normalizeDragScaleFactor(scaleFactor)
  if (
    dragPointerType !== 'touch' ||
    dragPointerId === null ||
    dragEnding ||
    touchFailed ||
    !touchInitialized ||
    touchScaleRebasePromise
  ) {
    return
  }

  const generation = dragGeneration
  const pointerId = dragPointerId
  touchScaleRebasing = true
  if (touchRafId !== null) {
    cancelAnimationFrame(touchRafId)
    touchRafId = null
  }

  let rebasePromise
  rebasePromise = (async () => {
    while (touchIsSettingPos) {
      await new Promise((resolve) => setTimeout(resolve, 0))
      if (!isCurrentDrag(generation, pointerId)) return
    }

    await new Promise((resolve) => requestAnimationFrame(resolve))
    if (!isCurrentDrag(generation, pointerId) || dragEnding) return

    const position = await appWindow.outerPosition()
    if (!isCurrentDrag(generation, pointerId) || dragEnding) return

    touchBasePhysicalX = position.x
    touchBasePhysicalY = position.y
    touchPendingPhysicalX = position.x
    touchPendingPhysicalY = position.y
    touchStartClientX = touchLatestClientX
    touchStartClientY = touchLatestClientY
  })()
    .catch(() => {
      markTouchDragFailed(generation, pointerId)
    })
    .finally(() => {
      if (isCurrentDrag(generation, pointerId)) {
        touchScaleRebasing = false
      }
      if (touchScaleRebasePromise === rebasePromise) {
        touchScaleRebasePromise = null
      }
    })

  touchScaleRebasePromise = rebasePromise
}

const ensureDragScaleListener = () => {
  if (dragDisposed || unlistenDragScaleChanged || dragScaleListenerPromise) return

  dragScaleListenerPromise = appWindow
    .onScaleChanged(({ payload }) => {
      rebaseTouchDragForScaleChange(payload.scaleFactor)
    })
    .then((unlisten) => {
      dragScaleListenerPromise = null
      if (dragDisposed) {
        unlisten()
        return
      }
      unlistenDragScaleChanged = unlisten
    })
    .catch(() => {
      dragScaleListenerPromise = null
    })
}

const queueTouchPosition = () => {
  // WebView2 touch coordinates are relative to the moving viewport. Keep only
  // one position IPC in flight so the next pointer sample uses the new baseline.
  if (
    dragPointerId === null ||
    dragEnding ||
    touchFailed ||
    touchPreparing ||
    touchScaleRebasing ||
    touchIsSettingPos ||
    !touchInitialized ||
    !Number.isFinite(touchBasePhysicalX) ||
    !Number.isFinite(touchBasePhysicalY)
  ) {
    return
  }

  updatePendingTouchPosition()

  if (touchRafId === null) {
    touchRafId = requestAnimationFrame(touchApplyPosition)
  }
}

const updatePendingTouchPosition = () => {
  if (
    !touchInitialized ||
    !Number.isFinite(touchBasePhysicalX) ||
    !Number.isFinite(touchBasePhysicalY)
  ) {
    return false
  }

  touchPendingPhysicalX = touchBasePhysicalX +
    (touchLatestClientX - touchStartClientX) * touchScaleFactor
  touchPendingPhysicalY = touchBasePhysicalY +
    (touchLatestClientY - touchStartClientY) * touchScaleFactor
  return Number.isFinite(touchPendingPhysicalX) &&
    Number.isFinite(touchPendingPhysicalY)
}

const prepareTouchDrag = async (generation, pointerId) => {
  const initialScaleFactorVersion = touchScaleFactorVersion
  const [position, scaleFactor] = await Promise.all([
    appWindow.outerPosition(),
    appWindow.scaleFactor(),
  ])
  if (!isCurrentDrag(generation, pointerId)) return

  if (touchScaleFactorVersion === initialScaleFactorVersion) {
    touchScaleFactor = normalizeDragScaleFactor(scaleFactor)
  }
  touchBasePhysicalX = position.x
  touchBasePhysicalY = position.y
  touchPendingPhysicalX = position.x
  touchPendingPhysicalY = position.y
  touchInitialized = true

  if (hasMoved && !dragEnding) {
    queueTouchPosition()
  }
}

const onContainerDragStart = (e) => {
  if (menuVisible.value) {
    onDragStart(e)
  }
}

const onDragStart = (e) => {
  if (
    e.button !== 0 ||
    isDragging ||
    dragPointerId !== null ||
    (e.pointerType === 'touch' && !e.isPrimary)
  ) {
    return
  }

  const generation = ++dragGeneration
  const pointerId = e.pointerId
  e.preventDefault()
  isDragging = true
  hasMoved = false
  dragEnding = false
  dragPointerType = e.pointerType
  dragPointerId = pointerId
  dragPointerTarget = e.currentTarget
  dragStartScreenX = e.screenX
  dragStartScreenY = e.screenY

  try {
    dragPointerTarget.setPointerCapture(dragPointerId)
  } catch {}

  document.addEventListener('pointermove', onDragMove, { passive: false })
  document.addEventListener('pointerup', onDragEnd)
  document.addEventListener('pointercancel', onDragEnd)

  if (e.pointerType === 'touch') {
    ensureDragScaleListener()
    touchStartClientX = e.clientX
    touchStartClientY = e.clientY
    touchLatestClientX = e.clientX
    touchLatestClientY = e.clientY
    touchBasePhysicalX = Number.NaN
    touchBasePhysicalY = Number.NaN
    touchPendingPhysicalX = Number.NaN
    touchPendingPhysicalY = Number.NaN
    touchInitialized = false
    touchPreparing = true
    touchIsSettingPos = false
    touchFailed = false
    touchScaleRebasing = false

    let preparationPromise
    preparationPromise = prepareTouchDrag(generation, pointerId)
      .catch(() => {
        markTouchDragFailed(generation, pointerId)
      })
      .finally(() => {
        if (isCurrentDrag(generation, pointerId)) {
          touchPreparing = false
        }
        if (touchPreparationPromise === preparationPromise) {
          touchPreparationPromise = null
        }
      })
    touchPreparationPromise = preparationPromise
  }
}

const touchApplyPosition = async () => {
  touchRafId = null
  if (
    !isDragging ||
    dragEnding ||
    touchFailed ||
    touchPreparing ||
    touchScaleRebasing ||
    touchIsSettingPos ||
    !touchInitialized ||
    !Number.isFinite(touchPendingPhysicalX) ||
    !Number.isFinite(touchPendingPhysicalY)
  ) {
    return
  }

  const generation = dragGeneration
  const pointerId = dragPointerId
  const nextPhysicalX = touchPendingPhysicalX
  const nextPhysicalY = touchPendingPhysicalY
  touchIsSettingPos = true

  try {
    await commitTouchPosition(nextPhysicalX, nextPhysicalY)
    if (!isCurrentDrag(generation, pointerId)) return

    touchBasePhysicalX = nextPhysicalX
    touchBasePhysicalY = nextPhysicalY
  } catch {
    markTouchDragFailed(generation, pointerId)
  } finally {
    if (isCurrentDrag(generation, pointerId)) {
      touchIsSettingPos = false
    }
  }
}

const onDragMove = (e) => {
  if (
    !isDragging ||
    e.pointerId !== dragPointerId ||
    e.pointerType !== dragPointerType ||
    dragEnding
  ) {
    return
  }

  if (dragPointerType === 'mouse' || dragPointerType === 'pen') {
    const dx = e.screenX - dragStartScreenX
    const dy = e.screenY - dragStartScreenY
    if (!hasMoved && Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return
    hasMoved = true
    e.preventDefault()
    const generation = dragGeneration
    const pointerId = dragPointerId
    isDragging = false
    dragEnding = true
    removeDragListeners()
    releaseDragPointerCapture()
    appWindow.startDragging()
      .then(() => finishNativeDrag(generation, pointerId))
      .catch(() => {
        finishDrag(generation, pointerId, true)
      })
    return
  }

  touchLatestClientX = e.clientX
  touchLatestClientY = e.clientY
  const deltaX = touchLatestClientX - touchStartClientX
  const deltaY = touchLatestClientY - touchStartClientY
  if (
    !hasMoved &&
    Math.abs(deltaX) < DRAG_THRESHOLD &&
    Math.abs(deltaY) < DRAG_THRESHOLD
  ) {
    return
  }

  hasMoved = true
  e.preventDefault()
  queueTouchPosition()
}

const onDragEnd = async (e) => {
  if (
    dragPointerId === null ||
    e.pointerId !== dragPointerId ||
    e.pointerType !== dragPointerType
  ) {
    return
  }

  const generation = dragGeneration
  const pointerId = dragPointerId
  const wasDragged = isDragging && hasMoved
  // A sample received during setPosition belongs to the previous viewport
  // coordinate frame and cannot be safely reapplied after that call completes.
  const canRefreshFinalTouchPosition =
    dragPointerType === 'touch' &&
    !touchIsSettingPos
  if (dragPointerType === 'touch') {
    touchLatestClientX = e.clientX
    touchLatestClientY = e.clientY
  }

  isDragging = false
  dragEnding = true
  removeDragListeners()
  releaseDragPointerCapture()
  if (touchRafId !== null) {
    cancelAnimationFrame(touchRafId)
    touchRafId = null
  }

  if (wasDragged && dragPointerType === 'touch') {
    if (touchPreparationPromise) {
      await touchPreparationPromise
      if (!isCurrentDrag(generation, pointerId)) return
    }
    if (touchScaleRebasePromise) {
      await touchScaleRebasePromise
      if (!isCurrentDrag(generation, pointerId)) return
    }
    while (touchIsSettingPos) {
      await new Promise((resolve) => setTimeout(resolve, 0))
      if (!isCurrentDrag(generation, pointerId)) return
    }

    if (
      !touchFailed &&
      canRefreshFinalTouchPosition &&
      isCurrentDrag(generation, pointerId)
    ) {
      updatePendingTouchPosition()
    }

    if (
      !touchFailed &&
      touchInitialized &&
      Number.isFinite(touchPendingPhysicalX) &&
      Number.isFinite(touchPendingPhysicalY)
    ) {
      try {
        await commitTouchPosition(touchPendingPhysicalX, touchPendingPhysicalY)
        if (!isCurrentDrag(generation, pointerId)) return

        const finalPos = await appWindow.outerPosition()
        if (!isCurrentDrag(generation, pointerId)) return

        queueToolbarPositionSave(finalPos)
      } catch {}
    }
  }

  finishDrag(generation, pointerId, wasDragged)
}

const onIconClick = () => {
  if (!hasMoved) toggleMenu()
}

// Window hit testing: the transparent margin passes clicks through to the
// window or desktop underneath.
// A transparent Tauri window swallows mouse events across its whole surface by
// default, and CSS pointer-events:none only affects the DOM, so the switch has
// to happen at the OS level through setIgnoreCursorEvents.
//
// The catch: while ignoreCursorEvents is true the DOM receives no mouse events
// at all, so the cursor has to be polled globally (cursorPosition plus
// outerPosition) to tell when it enters the launcher.
//
// Hit rules:
//   - cursor over the 80x80 launcher -> ignore=false (clickable and draggable)
//   - menu open: cursor anywhere in the 240x240 window -> ignore=false
//   - otherwise (empty margin, or outside the window) -> ignore=true
let cursorIgnoreDesired = false
let cursorIgnoreApplied = false
let cursorIgnoreUpdate = null
let hitTestTimer = null
let hitTestEnabled = false
const HIT_TEST_ACTIVE_INTERVAL_MS = 80
const HIT_TEST_IDLE_INTERVAL_MS = 250

const flushCursorIgnore = async () => {
  let failed = false
  while (hitTestEnabled && cursorIgnoreApplied !== cursorIgnoreDesired) {
    const target = cursorIgnoreDesired
    try {
      await appWindow.setIgnoreCursorEvents(target)
      cursorIgnoreApplied = target
    } catch (error) {
      console.warn('[ToolbarHitTest] setIgnoreCursorEvents failed', target, error)
      failed = true
      break
    }
  }
  cursorIgnoreUpdate = null

  // The target may have changed while the last IPC call was in flight.
  if (!failed && hitTestEnabled && cursorIgnoreApplied !== cursorIgnoreDesired) {
    cursorIgnoreUpdate = flushCursorIgnore()
  }
}

const setCursorIgnore = (ignore) => {
  cursorIgnoreDesired = ignore
  if (!cursorIgnoreUpdate && cursorIgnoreApplied !== cursorIgnoreDesired) {
    cursorIgnoreUpdate = flushCursorIgnore()
  }
}

const hitTestTick = async () => {
  // Stay non-transparent until the drag session ends so the native drag or the
  // touch wrap-up is not interrupted.
  if (dragPointerId !== null) {
    setCursorIgnore(false)
    return
  }
  try {
    const [winPos, winSize, cur] = await Promise.all([
      appWindow.outerPosition(),
      appWindow.outerSize(),
      cursorPosition(),
    ])
    if (dragPointerId !== null) {
      setCursorIgnore(false)
      return
    }

    // PhysicalPosition is in physical pixels.
    const relX = cur.x - winPos.x
    const relY = cur.y - winPos.y
    const inWindow = relX >= 0 && relY >= 0 && relX < winSize.width && relY < winSize.height
    if (!inWindow) {
      setCursorIgnore(true)
      return
    }
    if (menuVisible.value) {
      setCursorIgnore(false)
      return
    }
    const dpr = window.devicePixelRatio || 1
    const cx = winSize.width / 2
    const cy = winSize.height / 2
    const halfIcon = 40 * dpr
    const inIcon = Math.abs(relX - cx) <= halfIcon && Math.abs(relY - cy) <= halfIcon
    setCursorIgnore(!inIcon)
  } catch (e) {
    console.warn('[ToolbarHitTest] tick failed', e)
  }
}

const initBubbleClickThrough = () => {
  // Start pass-through; the poll decides when to take the cursor back.
  hitTestEnabled = true
  setCursorIgnore(true)
  const scheduleNextHitTest = (delay) => {
    if (!hitTestEnabled) return
    hitTestTimer = setTimeout(async () => {
      if (!hitTestEnabled) return
      await hitTestTick()
      if (!hitTestEnabled) return
      const active = dragPointerId !== null || menuVisible.value || !cursorIgnoreApplied
      scheduleNextHitTest(active ? HIT_TEST_ACTIVE_INTERVAL_MS : HIT_TEST_IDLE_INTERVAL_MS)
    }, delay)
  }
  scheduleNextHitTest(0)
}


onMounted(() => {
  initBubbleClickThrough()
})

onUnmounted(() => {
  dragDisposed = true
  clearDragState()
  if (unlistenDragScaleChanged) {
    unlistenDragScaleChanged()
    unlistenDragScaleChanged = null
  }
  hitTestEnabled = false
  if (hitTestTimer) {
    clearTimeout(hitTestTimer)
    hitTestTimer = null
  }
})
</script>

<style scoped lang="less">
#toolbar-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-sizing: border-box;
  // Collapsed: only the launcher is interactive, the margin stays transparent.
  pointer-events: none;
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;

  &.menu-open {
    pointer-events: auto;
    touch-action: none;
  }
}

.toolbar-icon {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  margin: 0;
  cursor: pointer;
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  color: var(--text);
  position: relative;
  z-index: 100;
  pointer-events: auto; // Always interactive, overriding the container.
  touch-action: none; // Suppress the browser default touch gestures.
  transition: background-color 120ms ease, border-color 120ms ease;

  &:hover {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  svg {
    width: 28px;
    height: 28px;
    display: block;
  }
}

.toolbar-menu {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  pointer-events: auto;
  z-index: 50;
}

.menu-header {
  flex: 0 0 auto;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border);
  color: var(--text-dim);
  font-size: 11px;
  font-weight: 400;
  text-transform: none;
  letter-spacing: normal;
  cursor: move;
  touch-action: none;
  user-select: none;
}

.menu-item {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
  background: transparent;
  border: 0;
  border-bottom: 1px solid var(--border);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: background-color 120ms ease, color 120ms ease;

  &:last-child {
    border-bottom: 0;
  }

  &:hover {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: -1px;
  }

  &.danger {
    color: var(--error);
  }
}

.menu-icon {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;

  :deep(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }
}

.menu-label {
  flex: 1 1 auto;
  font-size: 12px;
  font-weight: 400;
  text-transform: none;
  letter-spacing: normal;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
