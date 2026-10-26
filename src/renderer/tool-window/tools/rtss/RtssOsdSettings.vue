<template>
  <SectionPanel :title="t.rtssTool.osdSettings">
    <template #icon><Setting /></template>
    <template #actions>
      <button type="button" class="refresh-btn" :title="t.rtssTool.refresh" :aria-label="t.rtssTool.refresh" @click="load">↻</button>
    </template>

    <div class="prop-grid">
      <div class="prop-row">
        <label>{{ t.rtssTool.osdEnabled }}</label>
        <ToggleSwitch :modelValue="props.osd_enabled === 1" @update:modelValue="v => setProp('OSD', v ? '1' : '0')" />
      </div>
      <div class="prop-row">
        <label>{{ t.rtssTool.showOwnStats }}</label>
        <ToggleSwitch :modelValue="props.show_own_stats === 1" @update:modelValue="v => setProp('OSDShowOwnStatistics', v ? '1' : '0')" />
      </div>
      <div class="prop-row">
        <label>{{ t.rtssTool.osdZoom }}</label>
        <div class="zoom-btns">
          <button
            v-for="z in [1, 2, 3, 4]"
            :key="z"
            class="preset-btn small"
            :class="{ active: props.zoom === z }"
            @click="setProp('OnScreenDisplayZoom', String(z))"
          >
            {{ z }}x
          </button>
        </div>
      </div>
      <div class="prop-row">
        <label>{{ t.rtssTool.osdPosition }}</label>
        <div class="pos-inputs">
          <div class="pos-field">
            <span class="pos-label">X</span>
            <input
              type="number"
              :value="props.position_x"
              @change="e => setProp('OnScreenDisplayX', e.target.value)"
              class="pos-input"
              min="0"
            />
          </div>
          <div class="pos-field">
            <span class="pos-label">Y</span>
            <input
              type="number"
              :value="props.position_y"
              @change="e => setProp('OnScreenDisplayY', e.target.value)"
              class="pos-input"
              min="0"
            />
          </div>
        </div>
      </div>
      <div class="prop-row">
        <label>{{ t.rtssTool.coordSpace }}</label>
        <select
          :value="props.coordinate_space"
          @change="e => setProp('OSDCoordinateSpace', e.target.value)"
          class="fmt-select wide"
        >
          <option :value="0">{{ t.rtssTool.coordFramebuffer }}</option>
          <option :value="1">{{ t.rtssTool.coordScreen }}</option>
        </select>
      </div>
    </div>
  </SectionPanel>
</template>

<script setup>
import { reactive, onMounted } from 'vue'
import { Setting } from '../../../components/ui/compact/icons.js'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n/index.js'
import SectionPanel from '../../components/SectionPanel.vue'
import ToggleSwitch from '../../components/ToggleSwitch.vue'

const { t } = useI18n()
const emit = defineEmits(['message'])

const props = reactive({
  osd_enabled: null,
  show_own_stats: null,
  position_x: null,
  position_y: null,
  zoom: null,
  coordinate_space: null,
})

const PROP_MAP = {
  OSD: 'osd_enabled',
  OSDShowOwnStatistics: 'show_own_stats',
  OnScreenDisplayX: 'position_x',
  OnScreenDisplayY: 'position_y',
  OnScreenDisplayZoom: 'zoom',
  OSDCoordinateSpace: 'coordinate_space',
}

async function load() {
  try {
    const data = await invoke('rtss_get_osd_properties', { profile: null })
    Object.assign(props, data)
  } catch (e) {
    console.warn('Failed to read the OSD properties:', e)
  }
}

async function setProp(key, value) {
  try {
    await invoke('rtss_set_osd_property', { key, value, profile: null })
    if (PROP_MAP[key]) {
      props[PROP_MAP[key]] = parseInt(value)
    }
    emit('message', t.value.rtssTool.propApplied, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  }
}

onMounted(() => load())

defineExpose({ load })
</script>

<style lang="less" scoped>
.refresh-btn {
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 16px;
  cursor: pointer;
  padding: 2px 6px;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { color: var(--text); background: var(--bg-hover); }
}

.prop-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.prop-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;

  > label {
    font-size: 12px;
    color: var(--text-dim);
    flex-shrink: 0;
  }
}

.zoom-btns {
  display: flex;
  gap: 4px;
}

.preset-btn {
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { background: var(--bg-hover); color: var(--text); }
  &.active { background: var(--bg-active); border-color: var(--border-strong); color: var(--text); }
  &.small { padding: 4px 8px; font-size: @font-size-sm; }
}

.pos-inputs {
  display: flex;
  gap: 8px;
}

.pos-field {
  display: flex;
  align-items: center;
  gap: 4px;
}

.pos-label {
  font-size: 11px;
  color: var(--text-dim);
  font-weight: 600;
}

.pos-input {
  width: 60px;
  padding: 4px 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  outline: none;
  text-align: center;

  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
  &::-webkit-inner-spin-button,
  &::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
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
</style>
