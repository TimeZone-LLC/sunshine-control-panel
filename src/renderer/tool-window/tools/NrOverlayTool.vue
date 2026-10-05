<template>
  <section ref="surface" class="nr-overlay" :class="{ expanded, settled }" :style="{ '--panel-alpha': (expanded ? opacity : Math.max(35, opacity - 12)) / 100 }">
    <header class="nr-head">
      <span class="drag-handle" data-tauri-drag-region @pointerdown="touchDrag.onTouchWindowDragStart" :title="text.drag"><svg viewBox="0 0 16 24" aria-hidden="true"><circle v-for="(point, i) in [[5,6],[11,6],[5,12],[11,12],[5,18],[11,18]]" :key="i" :cx="point[0]" :cy="point[1]" r="1.5" /></svg></span>
      <button class="pill" :aria-expanded="expanded" :aria-label="text.states[state] + (state === 'active' ? ' · ' + (pipeline?.nr_scale_percent ?? 100) + '%' : '')" @click="expanded = !expanded">
        <span class="dot" :class="state" />
        <span>{{ expanded ? 'DLSS NR' : compactLabel }}</span>
        <ArrowDown class="chevron" :class="{ rotated: expanded }" aria-hidden="true" />
      </button>
      <button v-if="expanded" class="close" :aria-label="text.close" :title="text.close" @click="$emit('close')"><Close aria-hidden="true" /></button>
    </header>
    <div class="nr-reveal" :inert="!expanded" :aria-hidden="!expanded">
      <div class="nr-clip">
        <div class="nr-body">
          <select v-if="pipelines.length > 1 || (selectionExpired && pipelines.length)" v-model="selectedId" :aria-label="text.session" :disabled="busy">
            <option :value="null" disabled>{{ text.choose }}</option>
            <option v-for="item in pipelines" :key="item.id" :value="item.id">{{ text.session }} #{{ item.id }} · {{ nrOutputLabel(item) }}</option>
          </select>
          <div class="switch-row">
            <span>{{ text.enhancement }}</span>
            <button class="switch" role="switch" :aria-checked="effectActive" :aria-label="text.enhancement" :class="{ on: effectActive }" :disabled="!canToggle" @click="toggle"><span /></button>
          </div>
          <div class="live-state" aria-live="polite"><span class="dot" :class="state" />{{ text.states[state] }}</div>
          <p v-if="state === 'blocked'" class="notice">{{ pipeline?.backend !== 'none' ? text.hdrBusy : text.unsupported }}</p>
          <p v-else-if="state === 'degraded'" class="notice">{{ text.fallback }}<br><small>{{ pipeline?.nr_reason }}</small></p>
          <p v-else-if="['warming_up', 'stopping', 'scaling'].includes(state)" class="notice">{{ text.wait }}</p>
          <p v-if="error" class="notice error">{{ error }} <button @click="refresh">{{ text.retry }}</button></p>
          <div v-if="supportsControls" class="scale-control">
            <label class="parameter-row"><span>{{ text.scale }}</span><output>{{ draftScale }}%</output>
              <input type="range" min="20" max="100" step="5" :value="draftScale" :disabled="!canToggle"
                :aria-label="text.scale" @input="editSlider('scale', $event)" @change="commitSlider('scale')" @pointercancel="cancelEdit">
            </label>
            <div class="scale-size">{{ editing === 'scale' || !pipeline.nr_requested_enabled ? text.previewSize : text.processing }} · {{ processingSize }}</div>
            <p class="hint">{{ text.scaleHint }}</p>
            <label class="parameter-row"><span>{{ text.intensity }}</span><output>{{ draftIntensity }}%</output>
              <input type="range" min="0" max="100" step="5" :value="draftIntensity" :disabled="!canToggle"
                :aria-label="text.intensity" @input="editSlider('intensity', $event)" @change="commitSlider('intensity')" @pointercancel="cancelEdit">
            </label>
            <p class="hint">{{ text.applyHint }}</p>
            <details class="advanced">
              <summary>{{ text.advanced }}</summary>
              <template v-if="pipeline.nr_live_controls_version >= 3">
                <div class="motion-label">{{ text.style }}</div>
                <div class="motion-options style-options" role="group" :aria-label="text.style">
                  <button v-for="style in [0, 1, 2, 3, 4]" :key="style" :disabled="!canToggle"
                    :aria-pressed="pipeline.nr_requested_style === style" :class="{ selected: pipeline.nr_requested_style === style }"
                    @click="setOption({ style })">{{ style }}</button>
                </div>
                <label class="parameter-row skin-control"><span>{{ text.skinStructure }}</span><output>{{ draftSkin }}%</output>
                  <input type="range" min="0" max="100" step="5" :value="draftSkin" :disabled="!canToggle"
                    :aria-label="text.skinStructure" @input="editSlider('skin', $event)" @change="commitSlider('skin')" @pointercancel="cancelEdit">
                </label>
                <label class="advanced-row"><span>{{ text.autoMask }}</span>
                  <input type="checkbox" :checked="pipeline.nr_requested_auto_mask" :disabled="!canToggle"
                    @change="setOption({ autoMask: $event.target.checked })">
                </label>
              </template>
              <label class="advanced-row"><span>{{ text.uiCorrection }}</span>
                <input type="checkbox" :checked="pipeline.nr_requested_ui_correction" :disabled="!canToggle"
                  @change="setOption({ uiCorrection: $event.target.checked })">
              </label>
              <div class="motion-label">{{ text.motionQuality }}</div>
              <div class="motion-options" role="group" :aria-label="text.motionQuality">
                <button v-for="(label, quality) in text.motionOptions" :key="quality" :disabled="!canToggle"
                  :aria-pressed="pipeline.nr_requested_motion_quality === quality"
                  :class="{ selected: pipeline.nr_requested_motion_quality === quality }"
                  @click="setOption({ motionQuality: quality })">{{ label }}</button>
              </div>
            </details>
            <p v-if="pipeline.nr_settings_failure_reason" class="notice">{{ text.settingsFailed }}</p>
          </div>
          <p v-else-if="pipeline" class="hint">{{ text.updateHost }}</p>
          <div class="signal-row"><span>{{ online && pipeline ? nrOutputLabel(pipeline) : '—' }}</span><span v-if="online && pipeline" class="badge">{{ text.preserved }}</span></div>
          <p v-if="online && pipeline?.dv_profile && pipeline.dv_state !== 'active'" class="notice">{{ pipeline.dv_state === 'waiting' ? text.dvWaiting : text.dvFallback }}</p>
          <p v-else-if="online && pipeline && pipeline.hdr_mode !== 'sdr' && !pipeline?.dv_profile" class="hint">{{ text.transfer }} · {{ pipeline?.hdr_mode.toUpperCase() }}</p>
          <p class="hint" aria-live="polite">{{ saveState === 'saved' ? text.saved : text.remembered }}</p>
          <button v-if="supportsControls" class="reset-defaults" :disabled="!canToggle" @click="resetDefaults">{{ text.resetDefaults }}</button>
          <label class="opacity-row"><span>{{ text.opacity }}</span><output>{{ opacity }}%</output><input v-model.number="opacity" @change="saveOpacity" type="range" min="35" max="95" :aria-label="text.opacity"></label>
          <footer><kbd v-if="shortcut">{{ shortcut }}</kbd><span>{{ shortcut ? text.shortcut : text.shortcutUnavailable }}</span></footer>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { ArrowDown, Close } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { currentMonitor, getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from '../../desktop/i18n/index.js'
import { nrShortcutLabel, enhancementControlsText, enhancementError } from '../../composables/enhancementControls.js'
import { nrOverlayText } from '../../composables/nrOverlayMessages.js'
import { nrOverlayState, overlayOpacity, nrProcessingSize, nrPipelines, nrOutputLabel } from '../../composables/nrOverlayState.js'

import { useTouchWindowDrag } from '../../composables/useTouchWindowDrag.js'

const touchDrag = useTouchWindowDrag(null, { restoreMaximized: false, onFinish: () => { if (!disposed) void fitWindow() } })
defineEmits(['close'])
const { locale } = useI18n()
const text = computed(() => nrOverlayText(locale.value))
const expanded = ref(false)
const opacity = ref(62)
async function saveOpacity() {
  try { applyPreferences(await invoke('nr_overlay_save_settings', { patch: { opacity: Number(opacity.value) } })) }
  catch (error) { actionError.value = enhancementError(enhancementControlsText(locale.value), error) }
}
function applyPreferences(status) {
  if (!status?.settings || disposed) return
  opacity.value = overlayOpacity(status.settings.opacity)
  shortcut.value = nrShortcutLabel(status)
  if (status.target !== null && status.target !== undefined && status.target !== selectedId.value) { selectedId.value = status.target; selectionExpired.value = false }
}
const surface = ref(null)
const pipelines = ref([])
const selectedId = ref(null)
const selectionExpired = ref(false)
const online = ref(false)
const busy = ref(false)
const errorCode = ref('')
const actionError = ref('')
const error = computed(() => actionError.value || text.value[errorCode.value] || '')
const saveState = ref('')
const shortcut = ref('')
watch(selectedId, id => { if (id !== null && !disposed) void invoke('nr_overlay_select_session', { id }).catch(error => { actionError.value = enhancementError(enhancementControlsText(locale.value), error) }) })
const pipeline = computed(() => pipelines.value.find(item => item.id === selectedId.value))
const state = computed(() => nrOverlayState(pipeline.value, online.value))
const compactLabel = computed(() => state.value === 'active' ? `NR · ${pipeline.value?.nr_scale_percent ?? 100}%` : text.value.states[state.value])
const settled = ref(false)
let settleTimer
watch(state, (next, previous) => {
  clearTimeout(settleTimer)
  settled.value = ['active', 'disabled'].includes(next) && ['warming_up', 'stopping', 'scaling'].includes(previous)
  if (settled.value) settleTimer = setTimeout(() => { settled.value = false }, 1200)
})
const effectActive = computed(() => online.value && pipeline.value?.nr_state === 'active')
const canToggle = computed(() => online.value && pipeline.value?.nr_toggle_supported && !busy.value && !['warming_up', 'stopping', 'scaling'].includes(state.value))
const supportsControls = computed(() => pipeline.value?.nr_live_controls_version >= 2)
const draftScale = ref(100), draftIntensity = ref(100), draftSkin = ref(0)
const editing = ref('')
let editingSession = null
function syncDrafts() {
  if (editing.value !== 'scale') draftScale.value = pipeline.value?.nr_requested_scale_percent ?? 100
  if (editing.value !== 'skin') draftSkin.value = Math.round((pipeline.value?.nr_requested_skin_structure_strength ?? 0) * 100)
  if (editing.value !== 'intensity') draftIntensity.value = Math.round((pipeline.value?.nr_requested_intensity ?? 1) * 100)
}
watch(() => [selectedId.value, pipeline.value?.nr_requested_scale_percent, pipeline.value?.nr_requested_intensity, pipeline.value?.nr_requested_skin_structure_strength], syncDrafts)
watch(selectedId, () => { editing.value = ''; editingSession = null; saveState.value = ''; syncDrafts() }, { flush: 'sync' })
watch(online, value => { if (!value) cancelEdit() })
const processingSize = computed(() => nrProcessingSize(pipeline.value,
  editing.value === 'scale' || !pipeline.value?.nr_requested_enabled ? draftScale.value : pipeline.value.nr_scale_percent))
function editSlider(kind, event) {
  if (!canToggle.value) return
  editing.value = kind
  editingSession = selectedId.value
  if (kind === 'scale') draftScale.value = Number(event.target.value)
  else if (kind === 'skin') draftSkin.value = Number(event.target.value)
  else draftIntensity.value = Number(event.target.value)
}
function cancelEdit() { editing.value = ''; editingSession = null; syncDrafts() }
async function commitSlider(kind) {
  if (!canToggle.value || editingSession !== selectedId.value || editing.value !== kind) { cancelEdit(); return }
  const patch = kind === 'scale' ? { scalePercent: draftScale.value } : kind === 'skin' ? { skinStructureStrength: draftSkin.value / 100 } : { intensity: draftIntensity.value / 100 }
  await setOption(patch)
  cancelEdit()
}
async function setOption(patch) {
  if (!canToggle.value || !supportsControls.value) return
  await sendRequest(pipeline.value.nr_requested_enabled, patch)
}
async function resetDefaults() {
  if (!canToggle.value) return
  await sendRequest(false, { scalePercent: 100, intensity: 1, style: 0, skinStructureStrength: 0, autoMask: false, uiCorrection: false, motionQuality: 0 })
}
let disposed = false, refreshing = false, timer, observer, unlisten = []
async function refresh() {
  if (refreshing || disposed) return
  refreshing = true
  try {
    const result = await invoke('nr_live_status')
    if (disposed) return
    const next = nrPipelines(result?.pipelines)
    if (selectedId.value !== null && !next.some(item => item.id === selectedId.value)) {
      selectedId.value = null
      selectionExpired.value = true
    }
    pipelines.value = next
    // Multiple streams require explicit selection; do not target another user silently.
    if (!selectionExpired.value && selectedId.value === null && next.length === 1) selectedId.value = next[0].id
    online.value = true
    if (errorCode.value === 'disconnected') errorCode.value = ''
  } catch {
    if (!disposed) { online.value = false; errorCode.value = 'disconnected' }
  } finally { refreshing = false }
}
async function poll() { await refresh(); if (!disposed) timer = setTimeout(poll, 500) }
async function toggle() {
  if (!canToggle.value) return
  await sendRequest(!pipeline.value.nr_requested_enabled)
}
async function sendRequest(enabled, patch = {}) {
  const id = pipeline.value.id
  busy.value = true
  saveState.value = ''
  errorCode.value = ''; actionError.value = ''
  try {
    await invoke('nr_live_set_enabled', { id, enabled, ...patch })
    await refresh()
    try { await invoke('nr_live_remember', { id }); if (!disposed && selectedId.value === id) saveState.value = 'saved' }
    catch { if (!disposed && selectedId.value === id) actionError.value = text.value.saveFailed }
  } catch (reason) {
    if (!disposed && selectedId.value === id) errorCode.value = String(reason).includes('nr_session_ended') ? 'ended' : 'failed'
  } finally { busy.value = false }
}
let resizing = false, resizeAgain = false, resizeRetry
async function fitWindow() {
  if (!surface.value || disposed || touchDrag.active) return
  if (resizing) { resizeAgain = true; return }
  resizing = true
  try {
    const generation = touchDrag.generation
    const win = getCurrentWindow()
    const [scale, monitor] = await Promise.all([win.scaleFactor(), currentMonitor()])
    if (disposed || touchDrag.active || generation !== touchDrag.generation) return
    if (monitor) surface.value.style.setProperty('--panel-max-height', `${Math.max(120, Math.min(640, monitor.size.height / scale - 48))}px`)
    const width = Math.ceil(surface.value.getBoundingClientRect().width) + 8
    const height = Math.ceil(surface.value.getBoundingClientRect().height) + 8
    clearTimeout(resizeRetry)
    if (await invoke('resize_tool_window', { width, height }) === false && !disposed) {
      resizeRetry = setTimeout(() => void fitWindow(), 100)
    }
  } catch (reason) { console.warn('NR overlay resize failed', reason) }
  finally { resizing = false; if (resizeAgain) { resizeAgain = false; void fitWindow() } }
}
onMounted(async () => {
  observer = new ResizeObserver(() => void fitWindow())
  observer.observe(surface.value)
  for (const [name, handler] of [
    ['nr-settings-changed', e => applyPreferences(e.payload)],
    ['nr-action-error', e => { actionError.value = enhancementError(enhancementControlsText(locale.value), e.payload) }],
    ['nr-save-error', e => { if (e.payload === selectedId.value) { saveState.value = ''; actionError.value = text.value.saveFailed } }],
  ]) {
    try { const stop = await listen(name, handler); if (disposed) stop(); else unlisten.push(stop) } catch {}
  }
  try { applyPreferences(await invoke('nr_overlay_settings')) } catch {}
  if (!disposed) void poll()
})
onUnmounted(() => { disposed = true; clearTimeout(resizeRetry); clearTimeout(timer); clearTimeout(settleTimer); observer?.disconnect(); unlisten.forEach(stop => stop()) })

</script>

<style scoped>
.nr-overlay {
  width: 182px;
  margin: 4px;
  color: var(--text);
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 0;
  font: 13px/1.5 var(--font-sans);
  box-shadow: none;
  box-sizing: border-box;
  overflow: hidden;
}

.nr-overlay.expanded { width: 292px; }
button, select, input { font: inherit; }
button { cursor: pointer; color: inherit; }
button:focus-visible, input:focus-visible, select:focus-visible {
  outline: 2px solid var(--text);
  outline-offset: -2px;
}
button:disabled { cursor: default; opacity: .5; }

.nr-head {
  min-height: 32px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  gap: 5px;
  color: var(--text-muted);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-soft);
}

.drag-handle {
  app-region: drag;
  -webkit-app-region: drag;
  display: grid;
  place-items: center;
  width: 22px;
  height: 30px;
  flex: 0 0 22px;
  cursor: grab;
  touch-action: none;
  user-select: none;
}
.drag-handle:active { cursor: grabbing; }
.drag-handle svg { width: 16px; height: 24px; fill: currentColor; pointer-events: none; }

.pill {
  display: flex;
  align-items: center;
  gap: 7px;
  flex: 1;
  min-width: 0;
  padding: 4px 0;
  color: var(--text);
  text-align: left;
  white-space: nowrap;
  font-weight: 650;
  background: none;
  border: 0;
}
.chevron { width: 14px; height: 14px; flex: 0 0 14px; margin-left: auto; }
.nr-reveal { display: none; }
.expanded .nr-reveal { display: block; }
.nr-clip { min-height: 0; max-height: var(--panel-max-height, 640px); overflow-y: auto; scrollbar-width: thin; }
.nr-body { padding: 10px 12px 12px; }

.close {
  flex: 0 0 24px;
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: 1px solid transparent;
  background: transparent;
}
.close:hover { background: var(--bg-raised); border-color: var(--border); }
.close svg { width: 14px; height: 14px; }

.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 10px 0;
  font-weight: 650;
}
.switch {
  display: flex;
  justify-content: flex-start;
  width: 44px;
  height: 24px;
  padding: 2px;
  border: 1px solid var(--border-strong);
  border-radius: 0;
  background: var(--bg-surface);
  box-shadow: none;
}
.switch > span { display: block; width: 18px; height: 18px; background: var(--text-muted); }
.switch.on { justify-content: flex-end; background: var(--bg-raised); }
.switch.on > span { background: var(--text); }

.live-state { display: flex; align-items: center; gap: 8px; margin-bottom: 10px; font-size: 12px; }
.dot { display: inline-block; flex: 0 0 7px; width: 7px; height: 7px; background: var(--text-faint); }
.dot.active { background: var(--text-muted); }
.dot.degraded, .dot.blocked { border: 1px solid var(--border-strong); background: var(--bg-raised); }
.dot.warming_up, .dot.stopping, .dot.scaling {
  width: 8px;
  height: 8px;
  border: 2px solid var(--border-strong);
  border-top-color: var(--text);
  border-radius: 50%;
  background: transparent;
  animation: spin 1s linear infinite;
}

.scale-control { border-top: 1px solid var(--border); padding-top: 10px; margin-bottom: 12px; }
.parameter-row, .opacity-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--text);
  font-size: 12px;
}
.parameter-row output, .scale-size { font-family: var(--font-mono); color: var(--text-muted); }
.parameter-row input, .opacity-row input { width: 100%; accent-color: var(--text-muted); }
.advanced { border-top: 1px solid var(--border-soft); padding-top: 8px; color: var(--text-muted); font-size: 12px; }
.advanced summary { cursor: pointer; }
.advanced-row { display: flex; align-items: center; justify-content: space-between; margin-top: 10px; }
.advanced-row input { width: 16px; height: 16px; accent-color: var(--text-muted); }

.motion-label { margin: 10px 0 6px; }
.motion-options { display: grid; grid-template-columns: repeat(4, 1fr); gap: 4px; }
.style-options { grid-template-columns: repeat(5, 1fr); }
.skin-control { margin-top: 12px; }
.motion-options button, .reset-defaults {
  min-height: 28px;
  padding: 4px 6px;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 0;
  background: var(--bg-surface);
}
.motion-options button.selected { background: var(--bg-raised); border-color: var(--border-strong); }
.motion-options button:hover:not(:disabled), .reset-defaults:hover:not(:disabled) { border-color: var(--border-strong); }

.scale-size { margin-top: 8px; font-size: 12px; }
.signal-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--border);
  font-weight: 650;
}
.badge { padding: 2px 6px; color: var(--text-muted); border: 1px solid var(--border); font-size: 10px; }
.hint { margin: 7px 0 12px; color: var(--text-muted); font-size: 11px; }
.reset-defaults { margin: 0 0 12px; }
.parameter-row input[type='range'], .opacity-row input[type='range'] { height: 18px; }

footer {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-top: 12px;
  padding-top: 10px;
  color: var(--text-muted);
  border-top: 1px solid var(--border);
  font-size: 11px;
}
kbd {
  padding: 2px 5px;
  color: var(--text);
  border: 1px solid var(--border-strong);
  border-radius: 0;
  background: var(--bg-raised);
  font: 500 11px/1.5 var(--font-mono);
}
.notice { margin: 7px 0 10px; color: var(--text-muted); font-size: 12px; overflow-wrap: anywhere; }
.notice.error { color: var(--el-color-danger); }
.notice button { border: 0; color: inherit; background: none; text-decoration: underline; }
select {
  width: 100%;
  min-height: 28px;
  margin-bottom: 8px;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 0;
  background: var(--bg-surface);
}

.nr-overlay:not(.expanded) { width: 182px; border-color: var(--border-soft); }
.nr-overlay:not(.expanded) .nr-head { border-bottom: 0; }
.nr-overlay:not(.expanded) .drag-handle, .nr-overlay:not(.expanded) .chevron { visibility: hidden; }
.nr-overlay:not(.expanded):hover .drag-handle,
.nr-overlay:not(.expanded):hover .chevron,
.nr-overlay:not(.expanded):focus-within .drag-handle,
.nr-overlay:not(.expanded):focus-within .chevron { visibility: visible; }

@media (pointer: coarse) {
  .drag-handle { width: 36px; height: 40px; flex-basis: 36px; }
}

@keyframes spin { to { transform: rotate(360deg); } }
@media (prefers-reduced-motion: reduce) { .dot { animation: none !important; } }
</style>
