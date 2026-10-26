<template>
  <div class="tool-container" :class="{ 'embedded': embedded }">
    <div v-if="!embedded" class="tool-header">
      <h2>{{ t.rtssTool.title }}</h2>
      <button class="close-btn" @click="$emit('close')">×</button>
    </div>

    <div class="tool-content">
      <RtssStatusBar
        :status="status"
        :loading="statusLoading"
        @refresh="loadStatus"
        @message="showMessage"
      />

      <template v-if="status.running">
        <div class="two-col-grid">
          <RtssFpsSection
            v-if="status.cli_path"
            @message="showMessage"
          />

          <RtssOsdEditor
            :has-cli="!!status.cli_path"
            @message="showMessage"
          />

          <RtssOsdSettings
            v-if="status.cli_path"
            @message="showMessage"
          />

          <RtssMonitoring
            @message="showMessage"
          />

          <HwInfoMonitor
            @message="showMessage"
          />
        </div>
      </template>

      <!-- Messages -->
      <div v-if="message" :class="['message', messageType]">
        {{ message }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../i18n/index.js'

import RtssStatusBar from './rtss/RtssStatusBar.vue'
import RtssFpsSection from './rtss/RtssFpsSection.vue'
import RtssOsdEditor from './rtss/RtssOsdEditor.vue'
import RtssOsdSettings from './rtss/RtssOsdSettings.vue'
import RtssMonitoring from './rtss/RtssMonitoring.vue'
import HwInfoMonitor from './rtss/HwInfoMonitor.vue'

const { t } = useI18n()

defineProps({
  embedded: { type: Boolean, default: false },
})
defineEmits(['close'])

const statusLoading = ref(true)
const status = ref({
  running: false,
  version: '',
  osd_slot_count: 0,
  app_count: 0,
  cli_path: '',
  hooks_dll_path: '',
})

const message = ref('')
const messageType = ref('')

async function loadStatus() {
  statusLoading.value = true
  try {
    status.value = await invoke('get_rtss_status')
  } catch (e) {
    console.error('Failed to read the RTSS status:', e)
    status.value.running = false
  } finally {
    statusLoading.value = false
  }
}

function showMessage(msg, type) {
  message.value = msg
  messageType.value = type
  setTimeout(() => { message.value = '' }, 4000)
}

onMounted(() => {
  loadStatus()
})
</script>

<style lang="less" scoped>
.tool-container {
  width: 100%;
  color: var(--text);
  font-size: 14px;

  &.embedded {
    .tool-header { display: none; }
    .tool-content { padding: 0; }
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

  h2 {
    margin: 0;
    font-size: @font-size-base;
    font-weight: @font-weight-medium;
    text-align: left;
    letter-spacing: normal;
  }
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
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: -1px;
  }
}

.tool-content {
  padding: @spacing-md @pad-x @spacing-lg;
  display: flex;
  flex-direction: column;
  gap: @spacing-md;
  max-width: 880px;
  margin: 0 auto;
  width: 100%;
}

.two-col-grid {
  column-count: 2;
  column-gap: 10px;

  > * {
    break-inside: avoid;
    margin-bottom: 10px;
  }
}

.message {
  padding: @pad-y @pad-x;
  font-size: @font-size-sm;
  font-weight: @font-weight-normal;
  text-align: left;

  &.success {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text);
  }

  &.error {
    background: var(--bg-panel);
    border: 1px solid var(--error);
    color: var(--error);
  }
}

</style>
