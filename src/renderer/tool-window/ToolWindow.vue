<template>
  <div class="tool-window-overlay" :class="{ 'monitor-overlay': isMonitorTool }" @click.self="handleOverlayClick">
    <div class="tool-panel" @click.stop>
      <component :is="currentTool" v-if="currentTool" @close="closeWindow" />

      <div v-else class="loading-state">
        <div class="spinner"></div>
        <p>{{ t.toolWindow.loading }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, shallowRef, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from '../desktop/i18n/index.js'

const { t } = useI18n()

const currentTool = shallowRef(null)
const toolType = shallowRef('dpi')
const isMonitorTool = computed(() => ['performance', 'nr'].includes(toolType.value))

const closeWindow = async () => {
  try {
    const window = getCurrentWindow()
    if (toolType.value === 'nr') await invoke('nr_overlay_set_visible', { visible: false })
    else await window.close()
  } catch (error) {
    console.error('Failed to close the window:', error)
  }
}

const getToolType = () => {
  const params = new URLSearchParams(window.location.search)
  return params.get('tool') || 'dpi'
}

// Close on Escape
const handleKeyDown = (e) => {
  if (e.key === 'Escape') {
    closeWindow()
  }
}

const handleOverlayClick = () => {
  if (!isMonitorTool.value) {
    closeWindow()
  }
}

onMounted(async () => {
  toolType.value = getToolType()

  // Listen for keyboard events
  window.addEventListener('keydown', handleKeyDown)

  try {
    switch (toolType.value) {
      case 'dpi':
        currentTool.value = defineAsyncComponent(() => import('./tools/DpiAdjusterTool.vue'))
        break
      case 'bitrate':
        currentTool.value = defineAsyncComponent(() => import('./tools/BitrateTool.vue'))
        break
      case 'shortcuts':
        currentTool.value = defineAsyncComponent(() => import('./tools/ShortcutsTool.vue'))
        break
      case 'rtss':
        currentTool.value = defineAsyncComponent(() => import('./tools/RtssOsdTool.vue'))
        break
      case 'nr':
        currentTool.value = defineAsyncComponent(() => import('./tools/NrOverlayTool.vue'))
        break
      case 'performance':
        currentTool.value = defineAsyncComponent(() => import('./tools/HostPerformanceTool.vue'))
        break
      case 'toolbar_settings':
        currentTool.value = defineAsyncComponent(() => import('./tools/ToolbarSettingsTool.vue'))
        break
      default:
        console.error('Unknown tool type:', toolType.value)
    }
  } catch (error) {
    console.error('Failed to load the tool:', error)
  }
})

// Clean up the listener
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<style lang="less" scoped>
@import "@/components/spinner.less";

.tool-window-overlay {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--scrim);
  overflow: hidden;
  cursor: pointer;
}

.tool-window-overlay.monitor-overlay {
  align-items: flex-start;
  justify-content: flex-end;
  padding: 0;
  box-sizing: border-box;
  background: transparent;
  cursor: default;
}

.tool-window-overlay.monitor-overlay .tool-panel {
  width: 100vw;
  max-width: none;
  max-height: none;
  background: transparent;
  border: 0;
  overflow: visible;
}

/* A solid panel with a hairline border: the tool has to stay legible on top
   of game content, and a border is the only elevation the system allows. */
.tool-panel {
  max-width: 96vw;
  max-height: 96vh;
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  overflow: hidden;
  cursor: default;
}

.loading-state {
  padding: 32px;
  text-align: center;
  color: var(--text);
  width: 320px;
}

.spinner {
  .gloss-spinner();
  display: block;
  margin: 0 auto;
}

.loading-state p {
  margin-top: 10px;
  font-size: var(--font-size);
  font-weight: 400;
  color: var(--text-dim);
}
</style>
