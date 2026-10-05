<template>
  <div class="tool-container">
    <div class="tool-header">
      <h2>{{ t.toolbarTool.title }}</h2>
      <button class="close-btn" :aria-label="t.toolbarTool.close" @click="$emit('close')">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            fill="currentColor"
            d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
          />
        </svg>
      </button>
    </div>

    <div class="tool-body">
      <div class="row">
        <div class="row-info">
          <div class="row-name">{{ t.toolbarTool.shortcut }}</div>
          <div class="row-desc">{{ t.toolbarTool.shortcutDesc }}</div>
          <div v-if="shortcutUnavailable" class="row-desc warn">
            {{ t.toolbarTool.shortcutUnavailable }}
          </div>
          <div v-else-if="shortcutStateError" class="row-desc warn">
            {{ t.toolbarTool.shortcutStateError }}
          </div>
        </div>
        <div class="row-control">
          <ToggleSwitch
            :model-value="shortcutEnabled"
            :disabled="shortcutPending"
            :aria-label="t.toolbarTool.shortcut"
            @update:model-value="onShortcutToggle"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../desktop/i18n/index.js'
import ToggleSwitch from '../components/ToggleSwitch.vue'

defineEmits(['close'])

const { t } = useI18n()

const shortcutEnabled = ref(true)
const shortcutRegistered = ref(false)
const shortcutPending = ref(true)
const shortcutStateError = ref(false)

const shortcutUnavailable = computed(
  () => !shortcutPending.value
    && !shortcutStateError.value
    && shortcutEnabled.value
    && !shortcutRegistered.value,
)

async function loadShortcutStatus() {
  shortcutPending.value = true
  shortcutStateError.value = false
  try {
    const status = await invoke('load_toolbar_shortcut_status')
    shortcutEnabled.value = Boolean(status.enabled)
    shortcutRegistered.value = Boolean(status.registered)
  } catch (error) {
    console.warn('[ToolbarSettings] Failed to load the shortcut state:', error)
    shortcutStateError.value = true
  } finally {
    shortcutPending.value = false
  }
}

async function onShortcutToggle(enabled) {
  const previous = shortcutEnabled.value
  shortcutPending.value = true
  shortcutStateError.value = false
  try {
    const status = await invoke('set_toolbar_shortcut_enabled', { enabled })
    shortcutEnabled.value = Boolean(status.enabled)
    shortcutRegistered.value = Boolean(status.registered)
  } catch (error) {
    console.warn('[ToolbarSettings] Failed to save the shortcut state:', error)
    shortcutEnabled.value = previous
    shortcutStateError.value = true
  } finally {
    shortcutPending.value = false
  }
}

onMounted(() => {
  void loadShortcutStatus()
})
</script>

<style scoped lang="less">
.tool-container {
  width: 460px;
  max-width: 96vw;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  color: var(--text);
  font-family: var(--font-sans);
}

.tool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: @spacing-md;
  height: @row-height;
  padding: 0 @pad-x;
  border-bottom: 1px solid var(--border);

  h2 {
    margin: 0;
    font-size: @font-size-base;
    font-weight: @font-weight-medium;
    text-transform: none;
    letter-spacing: normal;
    color: var(--text);
  }
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  border: 1px solid var(--border-strong);
  color: var(--text);
  cursor: pointer;
  transition: background-color 120ms ease;

  &:hover {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  svg {
    width: 14px;
    height: 14px;
    display: block;
  }
}

.tool-body {
  padding: @spacing-md @pad-x;
}

.row {
  display: flex;
  align-items: flex-start;
  gap: @spacing-md;
  padding: @spacing-md;
  border: 1px solid var(--border);
}

.row-info {
  flex: 1 1 auto;
  min-width: 0;
}

.row-name {
  font-size: @font-size-base;
  font-weight: @font-weight-medium;
  color: var(--text);
}

.row-desc {
  margin-top: @spacing-xs;
  font-size: @font-size-sm;
  line-height: @line-height-normal;
  color: var(--text-dim);

  &.warn {
    color: var(--warn);
  }
}

.row-control {
  flex: 0 0 auto;
  padding-top: 2px;
}
</style>
