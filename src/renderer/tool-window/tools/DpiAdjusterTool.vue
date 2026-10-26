<template>
  <div class="tool-container" :class="{ 'embedded': embedded }">
    <div v-if="!embedded" class="tool-header">
      <h2>{{ t.dpiTool.title }}</h2>
      <button class="close-btn" @click="$emit('close')">×</button>
    </div>

    <div class="tool-content">
      <div class="dpi-display">
        <span class="dpi-value" v-if="!loading">{{ dpiValue }}%</span>
        <span class="dpi-value loading" v-else>{{ t.dpiTool.loading }}</span>
      </div>

      <div class="slider-container">
        <input
          type="range"
          v-model.number="dpiValue"
          min="100"
          max="300"
          step="25"
          class="dpi-slider"
        />
      </div>

      <div class="presets">
        <button 
          v-for="preset in presets" 
          :key="preset"
          @click="dpiValue = preset"
          :class="{ active: dpiValue === preset }"
          class="preset-btn"
        >
          {{ preset }}%
        </button>
      </div>

      <div class="actions">
        <button @click="applyDpi" class="apply-btn" :disabled="applying">
          {{ applying ? t.dpiTool.applying : t.dpiTool.apply }}
        </button>
      </div>

      <div v-if="message" :class="['message', messageType]">
        {{ message }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../../i18n/index.js';

const { t } = useI18n();

defineProps({
  embedded: {
    type: Boolean,
    default: false
  }
})

defineEmits(['close']);

const dpiValue = ref(100);
const applying = ref(false);
const message = ref('');
const messageType = ref('');
const loading = ref(true);

const presets = [100, 125, 150, 175, 200, 225, 250, 300];

const loadCurrentDpi = async () => {
  try {
    const currentDpi = await invoke('get_current_dpi');
    dpiValue.value = currentDpi;
    console.log('Current DPI:', currentDpi);
  } catch (error) {
    console.error('Failed to read the current DPI:', error);
  } finally {
    loading.value = false;
  }
};

const applyDpi = async () => {
  applying.value = true;
  message.value = '';
  
  try {
    await invoke('set_desktop_dpi', { dpi: dpiValue.value });
    message.value = t.value.dpiTool.success.replace('{value}', dpiValue.value);
    messageType.value = 'success';
    
    setTimeout(async () => {
      try {
        const currentDpi = await invoke('get_current_dpi');
        if (currentDpi !== dpiValue.value) {
          message.value = t.value.dpiTool.successPartial.replace('{value}', currentDpi);
          messageType.value = 'warning';
        }
      } catch (error) {
        console.error('Failed to re-read the DPI:', error);
      }
    }, 1000);
    
  } catch (error) {
    message.value = t.value.dpiTool.failed.replace('{error}', error);
    messageType.value = 'error';
  } finally {
    applying.value = false;
    
    setTimeout(() => {
      message.value = '';
    }, 5000);
  }
};

onMounted(() => {
  loadCurrentDpi();
});
</script>

<style lang="less" scoped>
// DPI adjuster. Same shape as the bitrate tool: value in monospace, presets as
// square cells, one plain apply button.

.tool-container {
  width: 100%;
  color: var(--text);

  &.embedded {
    width: 100%;

    .tool-header {
      display: none;
    }

    .tool-content {
      padding: 0;
      min-height: 236px;
      justify-content: center;
    }
  }
}

.tool-header {
  display: flex;
  align-items: center;
  height: @row-height;
  padding: 0 @pad-x;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  position: relative;
}

.tool-header h2 {
  margin: 0;
  font-size: @font-size-base;
  font-weight: @font-weight-medium;
  text-align: left;
}

.close-btn {
  position: absolute;
  top: 0;
  right: 0;
  width: @row-height;
  height: @row-height;
  border: none;
  border-left: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.close-btn:focus-visible {
  outline: 1px solid var(--border-strong);
  outline-offset: -1px;
}

.tool-content {
  padding: @spacing-md @pad-x;
  display: flex;
  flex-direction: column;
  gap: @spacing-md;
}

.dpi-display {
  padding: @spacing-md;
  background: var(--bg-panel);
  border: 1px solid var(--border);
}

.dpi-value {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: @font-weight-medium;
  line-height: 1.1;
}

.dpi-value.loading {
  font-family: var(--font-sans);
  font-size: @font-size-base;
  font-weight: @font-weight-normal;
  color: var(--text-dim);
}

.slider-container {
  padding: 0;
}

.dpi-slider {
  width: 100%;
  height: 4px;
  padding: 0;
  border: 0;
  background: var(--bg-active);
  outline: none;
  -webkit-appearance: none;
  appearance: none;
  cursor: pointer;
}

.dpi-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 10px;
  height: 16px;
  background: var(--text);
  cursor: pointer;
}

.dpi-slider::-moz-range-thumb {
  width: 10px;
  height: 16px;
  background: var(--text);
  border: none;
  cursor: pointer;
}

.dpi-slider:focus-visible {
  outline: 1px solid var(--border-strong);
  outline-offset: 2px;
}

.presets {
  display: flex;
  flex-wrap: wrap;
  gap: @spacing-xs;
}

.preset-btn {
  height: 22px;
  padding: 0 @spacing-md;
  border: 1px solid var(--border-strong);
  background: transparent;
  color: var(--text);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
}

.preset-btn:hover {
  background: var(--bg-hover);
}

.preset-btn:focus-visible {
  outline: 1px solid var(--border-strong);
  outline-offset: 0;
}

// Selected = a filled cell, not a hue.
.preset-btn.active {
  background: var(--bg-active);
  color: var(--text);
  border-color: var(--border-strong);
}

.actions {
  display: flex;
}

.apply-btn {
  height: @row-height;
  padding: 0 @spacing-lg;
  background: transparent;
  color: var(--text);
  border: 1px solid var(--border-strong);
  font-size: @font-size-base;
  font-weight: @font-weight-normal;
  cursor: pointer;
}

.apply-btn:hover:not(:disabled) {
  background: var(--bg-hover);
}

.apply-btn:active:not(:disabled) {
  background: var(--bg-active);
}

.apply-btn:focus-visible {
  outline: 1px solid var(--border-strong);
  outline-offset: 0;
}

.apply-btn:disabled {
  color: var(--text-faint);
  border-color: var(--border);
  cursor: not-allowed;
}

// Result line. Success is brightness; warn and error are the only hues.
.message {
  padding: @pad-y @pad-x;
  font-size: @font-size-sm;
  background: var(--bg-panel);
  border: 1px solid var(--border);
}

.message.success {
  color: var(--text);
}

.message.warning {
  color: var(--warn);
  border-color: var(--warn);
}

.message.error {
  color: var(--error);
  border-color: var(--error);
}

.loading-state {
  padding: @spacing-2xl;
  text-align: center;
  color: var(--text-dim);
}

.loading-state p {
  margin-top: @spacing-md;
  font-size: @font-size-base;
}
</style>
