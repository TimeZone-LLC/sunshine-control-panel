<template>
  <SectionPanel :title="t.rtssTool.monitoring">
    <template #icon><DataAnalysis /></template>
    <template #actions>
      <ToggleSwitch :modelValue="active" @update:modelValue="toggleMonitoring" />
    </template>

    <!-- Metric selection -->
    <div class="metric-groups">
      <div v-for="group in metricGroups" :key="group.id" class="metric-group">
        <div class="metric-group-label">{{ group.label }}</div>
        <div class="metric-chips">
          <button
            v-for="m in group.metrics"
            :key="m.id"
            class="metric-chip"
            :class="{ active: config.metrics.includes(m.id) }"
            @click="toggleMetric(m.id)"
          >
            {{ m.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Style configuration -->
    <div class="monitor-style">
      <div class="style-row">
        <label>{{ t.rtssTool.headerText }}</label>
        <input v-model="config.header_text" class="style-input" placeholder="Foundation Sunshine" />
      </div>
      <div class="style-row cjk-hint">
        <small>{{ t.rtssTool.cjkHint || 'CJK text requires "Vector 3D" to be enabled in RTSS Setup together with a CJK font such as Microsoft YaHei' }}</small>
      </div>
      <div class="style-row">
        <label>{{ t.rtssTool.updateInterval }}</label>
        <select v-model.number="config.interval_ms" class="fmt-select wide">
          <option :value="500">0.5s</option>
          <option :value="1000">1s</option>
          <option :value="2000">2s</option>
          <option :value="5000">5s</option>
        </select>
      </div>
      <div class="style-row colors-row">
        <div class="color-field">
          <label>{{ t.rtssTool.titleColor }}</label>
          <input type="color" :value="'#' + config.title_color" @input="e => config.title_color = e.target.value.slice(1)" />
        </div>
        <div class="color-field">
          <label>{{ t.rtssTool.labelColor }}</label>
          <input type="color" :value="'#' + config.label_color" @input="e => config.label_color = e.target.value.slice(1)" />
        </div>
        <div class="color-field">
          <label>{{ t.rtssTool.valueColor }}</label>
          <input type="color" :value="'#' + config.value_color" @input="e => config.value_color = e.target.value.slice(1)" />
        </div>
      </div>
    </div>

    <!-- OSD preview -->
    <div v-if="active && snapshot.osd_text" class="osd-preview">
      <div class="osd-preview-label">{{ t.rtssTool.osdPreview }}</div>
      <pre class="osd-preview-text">{{ snapshot.osd_text }}</pre>
    </div>
  </SectionPanel>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { DataAnalysis } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../desktop/i18n/index.js'
import SectionPanel from '../../components/SectionPanel.vue'
import ToggleSwitch from '../../components/ToggleSwitch.vue'

const { t } = useI18n()
const emit = defineEmits(['message'])

const STORAGE_KEY = 'rtss-monitoring-config'

const DEFAULT_CONFIG = {
  interval_ms: 1000,
  metrics: ['session_state', 'stream_fps', 'stream_bitrate', 'host_perf_p95'],
  title_color: 'FFD700',
  label_color: 'AAAAAA',
  value_color: '00FF00',
  font_size: 0,
  header_text: 'Foundation Sunshine',
}

function loadPersistedConfig() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      return { ...DEFAULT_CONFIG, ...parsed }
    }
  } catch {}
  return { ...DEFAULT_CONFIG }
}

const active = ref(false)
const snapshot = reactive({ active: false, osd_text: '', metrics: {} })
const config = reactive(loadPersistedConfig())

const availableMetrics = ref([])
let pollTimer = null

// Persist the config to localStorage.
watch(config, (v) => {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(v)) } catch {}
}, { deep: true })

const metricGroups = computed(() => {
  const groups = [
    { id: 'session', label: 'Streaming', metrics: [] },
    { id: 'host_perf', label: 'Host performance', metrics: [] },
    { id: 'process', label: 'Process', metrics: [] },
  ]
  for (const m of availableMetrics.value) {
    const grp = groups.find(g => g.id === m.group)
    if (grp) grp.metrics.push({ id: m.id, label: m.label_en })
  }
  return groups.filter(g => g.metrics.length > 0)
})

// ─── Methods ───
function toggleMetric(id) {
  const idx = config.metrics.indexOf(id)
  if (idx >= 0) config.metrics.splice(idx, 1)
  else config.metrics.push(id)
  if (active.value) startMonitoring()
}

async function toggleMonitoring(val) {
  if (val) await startMonitoring()
  else await stopMonitoring()
}

async function startMonitoring() {
  try {
    // Deep-copy the config so the reactive proxy does not break serialisation.
    const rawConfig = JSON.parse(JSON.stringify(config))
    await invoke('rtss_start_monitoring', { config: rawConfig })
    active.value = true
    startPoll()
    emit('message', t.value.rtssTool.monitoringStarted, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  }
}

async function stopMonitoring() {
  try {
    await invoke('rtss_stop_monitoring')
    active.value = false
    stopPoll()
    snapshot.active = false
    snapshot.osd_text = ''
    emit('message', t.value.rtssTool.monitoringStopped, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  }
}

function startPoll() {
  stopPoll()
  pollTimer = setInterval(async () => {
    try {
      const snap = await invoke('rtss_get_monitoring_status')
      Object.assign(snapshot, snap)
      if (!snap.active) { active.value = false; stopPoll() }
    } catch (e) { /* ignore */ }
  }, 2000)
}

function stopPoll() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

async function loadState() {
  try {
    availableMetrics.value = await invoke('rtss_get_available_metrics')
  } catch (e) { /* ignore */ }
  try {
    const snap = await invoke('rtss_get_monitoring_status')
    if (snap.active) {
      active.value = true
      Object.assign(snapshot, snap)
      // Adopt the config the backend is running, when there is one.
      if (snap.config) {
        Object.assign(config, snap.config)
      }
      startPoll()
    }
  } catch (e) { /* ignore */ }
}

onMounted(() => loadState())
onUnmounted(() => stopPoll())

defineExpose({ loadState })
</script>

<style lang="less" scoped>
.metric-groups {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric-group-label {
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 3px;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 600;
}

.metric-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.metric-chip {
  padding: 4px 12px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 11px;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { background: var(--bg-hover); color: var(--text); }
  &.active {
    background: var(--bg-active);
    border-color: var(--border-strong);
    color: var(--text);
    font-weight: 600;
  }
}

.monitor-style {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 4px;
}

.style-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;

  > label {
    color: var(--text-dim);
    min-width: 65px;
    flex-shrink: 0;
  }
}

.style-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 12px;
  outline: none;

  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
  &::placeholder { color: var(--text-faint); }
}

.colors-row {
  flex-wrap: wrap;
  gap: 10px;
}

.color-field {
  display: flex;
  align-items: center;
  gap: 5px;

  label {
    font-size: 11px;
    color: var(--text-dim);
    min-width: unset;
  }

  input[type="color"] {
    width: 24px;
    height: 24px;
    border: 1px solid var(--border);
    background: none;
    cursor: pointer;
    padding: 0;

    &::-webkit-color-swatch-wrapper { padding: 2px; }
    &::-webkit-color-swatch { border: none; }
  }
}

.fmt-select {
  padding: 4px 8px;
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  font-size: 12px;
  cursor: pointer;
  outline: none;

  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
  &.wide { min-width: 80px; }
  option { background: var(--bg-panel); color: var(--text); }
}

.osd-preview {
  background: var(--bg-input);
  border: 1px solid var(--border);
  padding: 8px 12px;
}

.osd-preview-label {
  font-size: 10px;
  color: var(--text-dim);
  margin-bottom: 4px;
  text-transform: none;
  letter-spacing: normal;
  font-weight: 600;
}

.osd-preview-text {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--on);
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  line-height: 1.6;
}
</style>
