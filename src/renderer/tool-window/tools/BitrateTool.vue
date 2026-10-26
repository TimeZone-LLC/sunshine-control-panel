<template>
  <div class="tool-container" :class="{ 'embedded': embedded }">
    <div v-if="!embedded" class="tool-header">
      <h2>{{ t.bitrateTool.title }}</h2>
      <button class="close-btn" @click="$emit('close')">×</button>
    </div>

    <div class="tool-content">
      <!-- Client selection -->
      <div class="section">
        <label class="section-label">{{ t.bitrateTool.selectClient }}</label>
        <select
          v-model="selectedClient"
          class="client-select"
          :disabled="isLoading || applying"
          @change="onClientChange"
        >
          <option value="">{{ t.bitrateTool.selectClientPlaceholder }}</option>
          <option v-for="session in activeSessions" :key="session.client_name" :value="session.client_name">
            {{ session.client_name }} ({{ session.width }}x{{ session.height }}@{{ session.fps }}fps)
          </option>
        </select>
        <button
          class="refresh-btn"
          :class="{ refreshing }"
          @click="loadSessions(true)"
          :disabled="isLoading || applying"
          :title="t.bitrateTool.refresh"
        >
          <RefreshRight class="refresh-icon" :class="{ spinning: refreshing }" />
        </button>
      </div>

      <!-- Loading state, shown only on the first load -->
      <div v-if="isLoading" class="loading-state">
        <p>{{ t.bitrateTool.loading }}</p>
      </div>

      <!-- No active session -->
      <template v-else-if="activeSessions.length === 0">
        <div class="empty-state">
          <p>{{ t.bitrateTool.noStream }}</p>
          <p class="subtitle">{{ t.bitrateTool.noStreamSub }}</p>
          <p v-if="allSessions.length > 0" class="subtitle warning-text">
            {{ t.bitrateTool.sessionsIdle.replace('{count}', allSessions.length) }}
          </p>
        </div>
      </template>

      <!-- Bitrate controls -->
      <template v-else-if="selectedClient">
        <div class="bitrate-controls">
          <!-- Current bitrate -->
          <div class="bitrate-display">
            <span class="bitrate-value">{{ formatBitrate(bitrateValue) }}</span>
            <span class="bitrate-label">{{ t.bitrateTool.targetBitrate }}</span>
            <div v-if="currentBitrate" class="current-bitrate">
              <span class="current-bitrate-label">{{ t.bitrateTool.currentBitrate }}</span>
              <span class="current-bitrate-value">{{ formatBitrate(currentBitrate) }}</span>
            </div>
          </div>

          <!-- Bitrate slider -->
          <div class="slider-container">
            <input
              type="range"
              v-model.number="bitrateValue"
              :min="BITRATE_LIMITS.MIN"
              :max="BITRATE_LIMITS.MAX"
              :step="BITRATE_LIMITS.STEP"
              class="bitrate-slider"
              :disabled="applying"
            />
            <div class="slider-labels">
              <span>{{ formatBitrate(BITRATE_LIMITS.MIN) }}</span>
              <span>{{ formatBitrate(BITRATE_LIMITS.MAX) }}</span>
            </div>
          </div>

          <!-- Presets -->
          <div class="presets">
            <button
              v-for="preset in BITRATE_PRESETS"
              :key="preset"
              @click="bitrateValue = preset"
              :class="{ active: bitrateValue === preset }"
              class="preset-btn"
              :disabled="applying"
            >
              {{ formatBitrate(preset) }}
            </button>
          </div>

          <!-- Custom value -->
          <div class="custom-input">
            <input
              type="number"
              v-model.number="bitrateValue"
              :min="BITRATE_LIMITS.MIN"
              :max="BITRATE_LIMITS.MAX"
              :step="BITRATE_LIMITS.STEP"
              class="bitrate-input"
              :disabled="applying"
              :placeholder="t.bitrateTool.inputPlaceholder"
            />
            <span class="input-label">Kbps</span>
          </div>

          <!-- Apply -->
          <div class="actions">
            <button @click="applyBitrate" class="apply-btn" :disabled="applying || !selectedClient">
              {{ applying ? t.bitrateTool.applying : t.bitrateTool.apply }}
            </button>
          </div>
        </div>
      </template>

      <!-- No client selected -->
      <div v-else class="empty-state">
        <p>{{ t.bitrateTool.selectFirst }}</p>
      </div>

      <!-- Messages -->
      <transition name="message-fade">
        <div v-if="message" :class="['message', messageType]">
          {{ message }}
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { sunshine } from '../../tauri-adapter.js'
import { RefreshRight } from '../../components/ui/compact/icons.js'
import { useI18n } from '../../i18n/index.js'

const { t } = useI18n()

// Constants
const BITRATE_LIMITS = Object.freeze({
  MIN: 1000,
  MAX: 800000,
  STEP: 1000,
})

const BITRATE_PRESETS = Object.freeze([5000, 10000, 20000, 50000, 100000, 200000])
const DEFAULT_BITRATE = 20000
const MESSAGE_TIMEOUT = 5000
const REFRESH_DELAY = 1000

defineProps({
  embedded: {
    type: Boolean,
    default: false
  }
})

defineEmits(['close'])

// Reactive state
const activeSessions = ref([])
const allSessions = ref([])
const selectedClient = ref('')
const bitrateValue = ref(DEFAULT_BITRATE)
const loading = ref(true)
const refreshing = ref(false)
const applying = ref(false)
const message = ref('')
const messageType = ref('')

// Computed
const isLoading = computed(() => loading.value && !refreshing.value)

const selectedSession = computed(() => {
  if (!selectedClient.value) return null
  return activeSessions.value.find((s) => s.client_name === selectedClient.value) ?? null
})

const currentBitrate = computed(() => selectedSession.value?.bitrate ?? null)

// Helpers
const formatBitrate = (kbps) => (kbps >= 1000 ? `${(kbps / 1000).toFixed(0)} Mbps` : `${kbps} Kbps`)

const isValidBitrate = (value) => value >= BITRATE_LIMITS.MIN && value <= BITRATE_LIMITS.MAX

let messageTimer = null
const showMessage = (msg, type = 'info', timeout = MESSAGE_TIMEOUT) => {
  if (messageTimer) clearTimeout(messageTimer)
  message.value = msg
  messageType.value = type
  messageTimer = setTimeout(() => {
    message.value = ''
    messageTimer = null
  }, timeout)
}

const getSessionBitrate = (clientName) => {
  const session = activeSessions.value.find((s) => s.client_name === clientName)
  return session?.bitrate ?? DEFAULT_BITRATE
}

// Session management
const loadSessions = async (isRefresh = false) => {
  if (isRefresh) {
    refreshing.value = true
  } else {
    loading.value = true
  }
  message.value = ''

  const previousClient = selectedClient.value

  try {
    const sessions = await sunshine.getActiveSessions()
    allSessions.value = sessions
    activeSessions.value = sessions.filter((s) => s.state !== 'STOPPED' && s.state !== 'STOPPING')

    if (activeSessions.value.length === 0) {
      selectedClient.value = ''
      return
    }

    // Restore or pick a client.
    const clientExists = activeSessions.value.some((s) => s.client_name === previousClient)
    selectedClient.value = previousClient && clientExists ? previousClient : activeSessions.value[0].client_name

    // Seed the slider with the current client's bitrate.
    bitrateValue.value = getSessionBitrate(selectedClient.value)
  } catch (error) {
    console.error('Failed to read the active sessions:', error)
    showMessage(t.value.bitrateTool.loadFailed.replace('{error}', error), 'error')
  } finally {
    loading.value = false
    refreshing.value = false
  }
}

const onClientChange = () => {
  if (selectedClient.value) {
    bitrateValue.value = getSessionBitrate(selectedClient.value)
  }
}

// Bitrate adjustment
const applyBitrate = async () => {
  if (!selectedClient.value) {
    showMessage(t.value.bitrateTool.selectFirstError, 'error', 3000)
    return
  }

  if (!isValidBitrate(bitrateValue.value)) {
    showMessage(
      t.value.bitrateTool.rangeError
        .replace('{min}', formatBitrate(BITRATE_LIMITS.MIN))
        .replace('{max}', formatBitrate(BITRATE_LIMITS.MAX)),
      'error',
      3000
    )
    return
  }

  applying.value = true
  message.value = ''

  try {
    const result = await sunshine.changeBitrate(selectedClient.value, bitrateValue.value)
    showMessage(`${result}`, 'success')
    setTimeout(() => loadSessions(true), REFRESH_DELAY)
  } catch (error) {
    console.error('Bitrate adjustment failed:', error)
    const errorMessage = error.toString()
    if (errorMessage.toLowerCase().includes('auth') || errorMessage.includes('401')) {
      showMessage(t.value.bitrateTool.authFailed, 'error')
    } else {
      showMessage(t.value.bitrateTool.applyFailed.replace('{error}', error), 'error')
    }
  } finally {
    applying.value = false
  }
}

onMounted(loadSessions)
</script>

<style lang="less" scoped>
// Bitrate tool. In-stream overlay: dense rows, one hairline between each,
// values in monospace. The only colour is --warn / --error on a message that
// actually reports a failure.

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

  h2 {
    margin: 0;
    font-size: @font-size-base;
    font-weight: @font-weight-medium;
    text-align: left;
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
  padding: @spacing-md @pad-x;
  display: flex;
  flex-direction: column;
  gap: @spacing-md;
}

.section {
  display: flex;
  align-items: center;
  gap: @spacing-sm;
  min-width: 0;
}

.section-label {
  font-size: @font-size-base;
  color: var(--text-dim);
  white-space: nowrap;
  flex-shrink: 0;
}

.client-select {
  flex: 1;
  min-width: 0;
  height: @row-height;
  padding: 0 @spacing-sm;
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: @font-size-sm;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover:not(:disabled),
  &:focus {
    border-color: var(--border-strong);
  }

  &:disabled {
    color: var(--text-faint);
    cursor: not-allowed;
  }

  option {
    background: var(--bg-panel);
    color: var(--text);
  }
}

.refresh-btn {
  width: @row-height;
  height: @row-height;
  flex-shrink: 0;
  border: 1px solid var(--border-strong);
  background: transparent;
  color: var(--text);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  &:disabled {
    color: var(--text-faint);
    cursor: not-allowed;
  }

  // The one motion the system keeps: a plain rotation while work is in flight.
  .spinning {
    animation: brutalist-spin 1s linear infinite;
  }

  .refresh-icon {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }
}


.loading-state,
.empty-state {
  text-align: center;
  padding: @spacing-lg @spacing-md;
  color: var(--text-dim);
}

.empty-state {
  min-height: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: @spacing-xs;

  p {
    margin: 0;
    font-size: @font-size-base;
    color: var(--text);
  }

  .subtitle {
    font-size: @font-size-sm;
    color: var(--text-dim);
  }

  .warning-text {
    color: var(--warn);
  }
}

.bitrate-controls {
  display: flex;
  flex-direction: column;
  gap: @spacing-md;
}

.bitrate-display {
  padding: @spacing-md;
  background: var(--bg-panel);
  border: 1px solid var(--border);
}

.bitrate-value {
  display: block;
  font-family: var(--font-mono);
  font-size: 28px;
  font-weight: @font-weight-medium;
  line-height: 1.1;
}

.bitrate-label {
  font-size: @font-size-sm;
  color: var(--text-dim);
}

.current-bitrate {
  display: flex;
  justify-content: space-between;
  gap: @spacing-sm;
  margin-top: @spacing-sm;
  padding-top: @spacing-sm;
  border-top: 1px solid var(--border);
  font-size: @font-size-sm;

  .current-bitrate-label {
    color: var(--text-dim);
  }

  .current-bitrate-value {
    font-family: var(--font-mono);
    color: var(--text);
  }
}

.slider-container {
  padding: 0;
}

.bitrate-slider {
  width: 100%;
  height: 4px;
  padding: 0;
  border: 0;
  background: var(--bg-active);
  outline: none;
  -webkit-appearance: none;
  appearance: none;
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

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 2px;
  }

  &:disabled {
    background: var(--bg-hover);
    cursor: not-allowed;
  }
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-top: @spacing-xs;
  font-family: var(--font-mono);
  font-size: @font-size-xs;
  color: var(--text-faint);
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

  &:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  // Selected = a filled cell, not a hue.
  &.active {
    background: var(--bg-active);
    color: var(--text);
    border-color: var(--border-strong);
  }

  &:disabled {
    color: var(--text-faint);
    border-color: var(--border);
    cursor: not-allowed;
  }
}

.custom-input {
  display: flex;
  align-items: center;
  gap: @spacing-sm;
}

.bitrate-input {
  flex: 1;
  height: @row-height;
  background: var(--bg-input);
  border: 1px solid var(--border);
  padding: 0 @spacing-sm;
  color: var(--text);
  font-family: var(--font-mono);
  font-size: @font-size-sm;
  outline: none;

  &:focus {
    border-color: var(--border-strong);
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  &:disabled {
    color: var(--text-faint);
    cursor: not-allowed;
  }
}

.input-label {
  font-family: var(--font-mono);
  font-size: @font-size-sm;
  color: var(--text-dim);
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

  &:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  &:active:not(:disabled) {
    background: var(--bg-active);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  &:disabled {
    color: var(--text-faint);
    border-color: var(--border);
    cursor: not-allowed;
  }
}

// Result line. Success is brightness; warn and error are the only hues, and
// they only appear when something actually went wrong.
.message {
  padding: @pad-y @pad-x;
  font-size: @font-size-sm;
  background: var(--bg-panel);
  border: 1px solid var(--border);

  &.success {
    color: var(--text);
  }

  &.warning {
    color: var(--warn);
    border-color: var(--warn);
  }

  &.error {
    color: var(--error);
    border-color: var(--error);
  }
}

.message-fade-enter-active,
.message-fade-leave-active {
  transition: background-color @transition-normal linear, color @transition-normal linear,
    border-color @transition-normal linear;
}
</style>
