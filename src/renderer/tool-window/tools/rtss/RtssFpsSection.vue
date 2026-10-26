<template>
  <SectionPanel :title="t.rtssTool.fpsLimiter">
    <template #icon><Aim /></template>
    <template #actions>
      <ToggleSwitch :modelValue="limiterEnabled" @update:modelValue="onToggleLimiter" />
    </template>

    <div class="fps-display">
      <span class="fps-value">{{ fpsValue }}</span>
      <span class="fps-label">FPS</span>
    </div>

    <div class="slider-container">
      <input
        type="range"
        v-model.number="fpsValue"
        :min="15"
        :max="360"
        :step="1"
        class="fps-slider"
        :disabled="applying"
      />
      <div class="slider-labels">
        <span>15</span>
        <span>60</span>
        <span>120</span>
        <span>240</span>
        <span>360</span>
      </div>
    </div>

    <div class="presets">
      <button
        v-for="preset in presets"
        :key="preset"
        @click="fpsValue = preset"
        :class="{ active: fpsValue === preset }"
        class="preset-btn"
      >
        {{ preset }}
      </button>
    </div>

    <div class="actions">
      <button @click="applyLimit" class="apply-btn" :disabled="applying">
        {{ applying ? t.rtssTool.applying : t.rtssTool.applyFps }}
      </button>
      <button @click="removeLimit" class="reset-btn" :disabled="applying">
        {{ t.rtssTool.removeFps }}
      </button>
    </div>
  </SectionPanel>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Aim } from '../../../components/ui/compact/icons.js'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n/index.js'
import SectionPanel from '../../components/SectionPanel.vue'
import ToggleSwitch from '../../components/ToggleSwitch.vue'

const { t } = useI18n()
const emit = defineEmits(['message'])

const fpsValue = ref(60)
const applying = ref(false)
const limiterEnabled = ref(true)
const presets = [30, 60, 90, 120, 144, 165, 240]

async function loadState() {
  try {
    const currentFps = await invoke('rtss_get_framerate_limit', { profile: null })
    if (currentFps > 0) fpsValue.value = currentFps
    const state = await invoke('rtss_get_limiter_status')
    limiterEnabled.value = state === 1
  } catch (e) {
    console.warn('Failed to read the framerate limiter state:', e)
  }
}

async function applyLimit() {
  applying.value = true
  try {
    await invoke('rtss_set_framerate_limit', { fps: fpsValue.value, profile: null })
    emit('message', t.value.rtssTool.fpsApplied.replace('{fps}', fpsValue.value), 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  } finally {
    applying.value = false
  }
}

async function removeLimit() {
  applying.value = true
  try {
    await invoke('rtss_set_framerate_limit', { fps: 0, profile: null })
    fpsValue.value = 0
    emit('message', t.value.rtssTool.fpsRemoved, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  } finally {
    applying.value = false
  }
}

async function onToggleLimiter() {
  try {
    const result = await invoke('rtss_toggle_limiter')
    limiterEnabled.value = result === '1'
    emit('message',
      limiterEnabled.value ? t.value.rtssTool.limiterEnabled : t.value.rtssTool.limiterDisabled,
      'success'
    )
  } catch (e) {
    emit('message', String(e), 'error')
  }
}

onMounted(() => loadState())

defineExpose({ loadState })
</script>

<style lang="less" scoped>
.fps-display {
  text-align: left;
  padding: 2px 0 0;
}

.fps-value {
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: @font-weight-medium;
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.fps-label {
  font-size: 13px;
  color: var(--text-dim);
  margin-left: 4px;
}

.slider-container {
  padding: 0 4px;
}

.fps-slider {
  width: 100%;
  height: 4px;
  padding: 0;
  border: 0;
  -webkit-appearance: none;
  appearance: none;
  background: var(--bg-active);
  outline: none;
  cursor: pointer;

  &::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 10px;
    height: 16px;
    background: var(--text);
    cursor: pointer;
  }

  &::-moz-range-thumb {
    width: 10px;
    height: 16px;
    background: var(--text);
    border: none;
    cursor: pointer;
  }

  &:focus-visible { outline: 1px solid var(--border-strong); outline-offset: 2px; }
  &:disabled { background: var(--bg-hover); cursor: not-allowed; }
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 4px;
  padding: 0 2px;
}

.presets {
  display: flex;
  flex-wrap: wrap;
  gap: @spacing-xs;
}

.preset-btn {
  height: 22px;
  padding: 0 @spacing-md;
  font-family: var(--font-mono);
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { background: var(--bg-hover); color: var(--text); }
  &:focus-visible { outline: 1px solid var(--border-strong); outline-offset: 0; }
  &.active { background: var(--bg-active); border-color: var(--border-strong); color: var(--text); }
}

.actions {
  display: flex;
  gap: @spacing-sm;
}

.apply-btn {
  height: 22px;
  padding: 0 @spacing-lg;
  border: 1px solid var(--border-strong);
  background: transparent;
  color: var(--text);
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover:not(:disabled) { background: var(--bg-hover); }
  &:focus-visible { outline: 1px solid var(--border-strong); outline-offset: 0; }
  &:disabled { color: var(--text-faint); border-color: var(--border); cursor: not-allowed; }
}

.reset-btn {
  height: 22px;
  padding: 0 @spacing-md;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover:not(:disabled) { background: var(--bg-hover); color: var(--text); }
  &:focus-visible { outline: 1px solid var(--border-strong); outline-offset: 0; }
  &:disabled { color: var(--text-faint); cursor: not-allowed; }
}
</style>
