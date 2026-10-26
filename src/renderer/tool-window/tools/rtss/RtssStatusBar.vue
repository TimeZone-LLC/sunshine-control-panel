<template>
  <!-- Status indicator -->
  <div class="status-bar" :class="{ online: status.running, offline: !status.running }">
    <span class="status-dot"></span>
    <span v-if="loading">{{ t.rtssTool.checking }}</span>
    <span v-else-if="status.running">
      RTSS v{{ status.version }} · {{ t.rtssTool.running }}
    </span>
    <span v-else>{{ t.rtssTool.notRunning }}</span>
    <button type="button" class="refresh-btn" :disabled="loading" :title="t.rtssTool.refresh" :aria-label="t.rtssTool.refresh" @click="$emit('refresh')">↻</button>
  </div>

  <!-- rtss-cli detection -->
  <div v-if="status.running && !status.cli_path" class="warning-box">
    <div class="warning-text">{{ t.rtssTool.cliNotFound }}</div>
    <button class="download-btn" @click="downloadCli" :disabled="downloading">
      {{ downloading ? t.rtssTool.downloading : t.rtssTool.autoDownload }}
    </button>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
  status: { type: Object, required: true },
  loading: { type: Boolean, default: false },
})
const emit = defineEmits(['refresh', 'message'])

const downloading = ref(false)

async function downloadCli() {
  downloading.value = true
  try {
    const path = await invoke('rtss_download_cli')
    emit('message', t.value.rtssTool.cliDownloaded, 'success')
    // Refresh so a freshly downloaded CLI is detected.
    emit('refresh')
  } catch (e) {
    emit('message', String(e), 'error')
  } finally {
    downloading.value = false
  }
}
</script>

<style lang="less" scoped>
.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
  background: var(--bg-input);
  border: 1px solid var(--border);

  &.online .status-dot { background: var(--on); }
  &.offline .status-dot { background: var(--error); }
}

.status-dot {
  width: 8px;
  height: 8px;
  flex-shrink: 0;
}

.refresh-btn {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 16px;
  cursor: pointer;
  padding: 2px 6px;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { color: var(--text); background: var(--bg-hover); }
  &:disabled { color: var(--text-faint); cursor: not-allowed; }
}

.warning-box {
  padding: 8px 12px;
  background: var(--bg-hover);
  border: 1px solid var(--warn);
  font-size: 12px;
  line-height: 1.4;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.warning-text {
  flex: 1;
  color: var(--text);
}

.download-btn {
  height: @control-height;
  padding: 0 12px;
  border: 1px solid var(--border-strong);
  background: var(--bg-active);
  color: var(--text);
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;
  white-space: nowrap;
  flex-shrink: 0;

  &:hover { background: var(--bg-hover); }
  &:disabled { color: var(--text-faint); border-color: var(--border); cursor: not-allowed; }
}
</style>
