<template>
  <main class="grid min-h-full place-items-center bg-background p-5 text-foreground">
    <section class="w-full max-w-[520px] border border-border bg-card">
      <header class="border-b border-border px-4 py-3">
        <h1 class="text-sm font-semibold">Sunshine host</h1>
        <p class="mt-0.5 text-xs text-muted-foreground">Local runtime information</p>
      </header>

      <dl class="grid grid-cols-[140px_minmax(0,1fr)] text-[13px]">
        <template v-for="row in rows" :key="row.label">
          <dt class="border-b border-r border-border bg-muted px-3 py-2 text-muted-foreground">
            {{ row.label }}
          </dt>
          <dd class="min-w-0 break-words border-b border-border px-3 py-2 font-mono">
            {{ row.value }}
          </dd>
        </template>
      </dl>

      <footer class="flex justify-end p-3">
        <Button variant="outline" size="sm" @click="closeWindow">Close</Button>
      </footer>
    </section>
  </main>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button/index.js'

const systemInfo = ref(null)
const unknown = 'Unknown'
const rows = computed(() => {
  const info = systemInfo.value
  if (!info) return [
    { label: 'Operating system', value: 'Loading' },
    { label: 'Architecture', value: 'Loading' },
  ]
  return [
    { label: 'Operating system', value: info.os_version || unknown },
    { label: 'Architecture', value: info.arch || unknown },
    { label: 'Processor', value: info.cpu_model || unknown },
    { label: 'Memory', value: info.memory_total ? `${(info.memory_total / 1073741824).toFixed(1)} GB` : unknown },
  ]
})

async function closeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch {
    window.close()
  }
}

onMounted(async () => {
  document.title = 'About Sunshine host'
  try {
    systemInfo.value = await invoke('get_system_info')
  } catch {
    systemInfo.value = { arch: navigator.platform || unknown }
  }
  requestAnimationFrame(() => {
    invoke('resize_about_window', { height: Math.ceil(document.body.scrollHeight + 24) }).catch(() => {})
  })
})
</script>
