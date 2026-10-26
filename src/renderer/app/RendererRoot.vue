<template>
  <component :is="activeRenderer" />
  <FeedbackHost />
</template>

<script setup>
import { computed, defineAsyncComponent } from 'vue'
import FeedbackHost from '@/components/ui/compact/FeedbackHost.vue'

const mode = new URLSearchParams(window.location.search).get('mode') || 'main'
document.documentElement.dataset.rendererMode = mode

const renderers = {
  main: defineAsyncComponent(() => import('./SunshineApp.vue')),
  about: defineAsyncComponent(() => import('./views/AboutView.vue')),
  pin: defineAsyncComponent(() => import('./views/PinView.vue')),
  console: defineAsyncComponent(() => import('@/console/LogConsoleApp.vue')),
  overlay: defineAsyncComponent(() => import('./NrOverlayWindow.vue')),
}

const activeRenderer = computed(() => renderers[mode] || renderers.main)
</script>
