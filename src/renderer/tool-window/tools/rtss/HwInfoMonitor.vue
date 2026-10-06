<template>
  <SectionPanel :title="t.hwinfo.title">
    <template #icon><Monitor /></template>
    <template #actions>
      <button class="refresh-btn" @click="loadSensors" :disabled="loading">
        <Refresh />
      </button>
      <ToggleSwitch :modelValue="injecting" @update:modelValue="toggleInject" :disabled="!available" />
    </template>

    <!-- Unavailable notice -->
    <div v-if="!available && !loading" class="unavailable-hint">
      <WarningFilled class="warn-icon" />
      <span>{{ t.hwinfo.unavailable }}</span>
      <div class="hint-sub">{{ t.hwinfo.enableHint }}</div>
    </div>

    <!-- Loading -->
    <div v-else-if="loading" class="loading-hint">{{ t.hwinfo.loading }}</div>

    <!-- Sensor selection -->
    <template v-else>
      <input
        v-model="searchQuery"
        class="search-input"
        :placeholder="t.hwinfo.searchPlaceholder"
      />

      <div class="sensor-list">
        <div
          v-for="group in filteredGroups"
          :key="group.name"
          class="sensor-group"
        >
          <div class="sensor-group-header" @click="group.collapsed = !group.collapsed">
            <span class="collapse-arrow">{{ group.collapsed ? '▶' : '▼' }}</span>
            {{ group.name }}
            <span class="sensor-count">{{ group.readings.length }}</span>
          </div>
          <div v-show="!group.collapsed" class="sensor-readings">
            <label
              v-for="r in group.readings"
              :key="r.index"
              class="reading-item"
              :class="{ selected: selectedIds.has(r.index) }"
            >
              <input type="checkbox" :checked="selectedIds.has(r.index)" @change="toggleReading(r.index)" />
              <span class="reading-label">{{ r.label }}</span>
              <span class="reading-value">{{ r.value.toFixed(1) }} {{ r.unit }}</span>
            </label>
          </div>
        </div>
        <div v-if="filteredGroups.length === 0" class="no-match">{{ t.hwinfo.noMatch }}</div>
      </div>

      <div class="selection-bar" v-if="selectedIds.size > 0">
        <span>{{ t.hwinfo.selectedCount.replace('{count}', selectedIds.size) }}</span>
      </div>
    </template>
  </SectionPanel>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { Monitor, Refresh, WarningFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../desktop/i18n/index.js'
import SectionPanel from '../../components/SectionPanel.vue'
import ToggleSwitch from '../../components/ToggleSwitch.vue'

const { t } = useI18n()
const emit = defineEmits(['message'])

const available = ref(false)
const loading = ref(true)
const injecting = ref(false)
const searchQuery = ref('')
const selectedIds = ref(new Set())
const sensorGroups = ref([])
let pollTimer = null

// Sensor grouping
function buildGroups(data) {
  const groups = []
  if (!data) return groups
  for (const sensor of data.sensors) {
    const readings = data.readings
      .map((r, i) => ({ ...r, index: i }))
      .filter(r => r.sensor_index === data.sensors.indexOf(sensor))
    if (readings.length > 0) {
      groups.push({
        name: sensor.name_user || sensor.name_original,
        collapsed: true,
        readings: readings.map(r => ({
          index: r.index,
          label: r.label_user || r.label_original,
          unit: r.unit,
          value: r.value,
          type: r.reading_type,
        })),
      })
    }
  }
  return groups
}

const filteredGroups = computed(() => {
  const q = searchQuery.value.toLowerCase()
  if (!q) return sensorGroups.value
  return sensorGroups.value
    .map(g => ({
      ...g,
      collapsed: false,
      readings: g.readings.filter(r =>
        r.label.toLowerCase().includes(q) || g.name.toLowerCase().includes(q)
      ),
    }))
    .filter(g => g.readings.length > 0)
})

function toggleReading(index) {
  const s = new Set(selectedIds.value)
  if (s.has(index)) s.delete(index)
  else s.add(index)
  selectedIds.value = s
}

async function loadSensors() {
  loading.value = true
  try {
    const ok = await invoke('hwinfo_check_available')
    available.value = ok
    if (ok) {
      const data = await invoke('hwinfo_get_sensors')
      sensorGroups.value = buildGroups(data)
    }
  } catch (e) {
    available.value = false
  }
  loading.value = false
}

async function toggleInject(val) {
  if (val) {
    await startInject()
  } else {
    stopInject()
  }
}

async function startInject() {
  if (selectedIds.value.size === 0) {
    emit('message', 'Select the sensors to monitor first', 'warning')
    return
  }
  injecting.value = true
  pollAndInject()
  pollTimer = setInterval(pollAndInject, 2000)
}

function stopInject() {
  injecting.value = false
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
  // Clear the HWiNFO data from the OSD.
  try { invoke('rtss_set_osd', { text: '' }).catch(() => {}) } catch {}
}

async function pollAndInject() {
  try {
    const ids = [...selectedIds.value]
    const readings = await invoke('hwinfo_get_readings', { readingIds: ids })
    // Build the OSD text.
    const lines = readings.map(r => {
      const label = r.label_user || r.label_original
      const val = r.value.toFixed(1)
      return `${label}: ${val} ${r.unit}`
    })
    const osdText = lines.join('\n')
    await invoke('rtss_set_osd', { text: osdText })
    // Update the displayed values as well.
    await refreshValues()
  } catch (e) {
    emit('message', String(e), 'error')
    stopInject()
  }
}

async function refreshValues() {
  try {
    const data = await invoke('hwinfo_get_sensors')
    sensorGroups.value = buildGroups(data)
  } catch {}
}

onMounted(() => loadSensors())
onUnmounted(() => {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
})
</script>

<style lang="less" scoped>
.unavailable-hint {
  text-align: center;
  padding: 12px;
  color: var(--text-dim);
  font-size: 12px;
  .warn-icon { width: 16px; height: 16px; color: var(--warn); vertical-align: middle; margin-right: 4px; }
  .hint-sub { font-size: 11px; color: var(--text-faint); margin-top: 4px; }
}

.loading-hint {
  text-align: center;
  padding: 12px;
  color: var(--text-dim);
  font-size: 12px;
}

.search-input {
  width: 100%;
  padding: 4px 8px;
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  font-size: 11px;
  margin-bottom: 6px;
  outline: none;
  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
}

.sensor-list {
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid var(--border);
  background: var(--bg-input);
}

.sensor-group-header {
  padding: 4px 8px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-dim);
  background: var(--bg-panel);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  user-select: none;
  &:hover { background: var(--bg-panel); }
  .collapse-arrow { font-size: 8px; width: 10px; }
  .sensor-count {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-faint);
  }
}

.reading-item {
  display: flex;
  align-items: center;
  padding: 4px 8px 4px 20px;
  font-size: 11px;
  cursor: pointer;
  gap: 6px;
  color: var(--text-dim);
  &:hover { background: var(--bg-panel); }
  &.selected { background: var(--bg-active); }
  input[type="checkbox"] { accent-color: var(--text-dim); width: 12px; height: 12px; }
  .reading-label { flex: 1; }
  .reading-value { font-family: var(--font-mono); font-size: 10px; color: var(--text-dim); }
}

.selection-bar {
  margin-top: 6px;
  font-size: 11px;
  color: var(--text);
  text-align: center;
}

.no-match {
  padding: 12px;
  text-align: center;
  font-size: 11px;
  color: var(--text-faint);
}

.refresh-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  &:hover { color: var(--text); }
  &:disabled { color: var(--text-faint); }
  svg, :deep(svg) { width: 14px; height: 14px; }
}
</style>
