<template>
  <NrOverlayTool @close="closeOverlay" />
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import NrOverlayTool from '@/tool-window/tools/NrOverlayTool.vue'

async function closeOverlay() {
  await invoke('nr_overlay_set_visible', { visible: false }).catch(() => {})
}
</script>
