<template>
  <main class="native-page text-foreground">
    <div class="border border-border bg-card">
      <section v-for="(group, groupIndex) in utilityGroups" :key="group.id" :class="groupIndex ? 'border-t border-border' : ''">
        <header class="border-b border-border bg-muted px-3 py-2">
          <h2 class="text-[12px] font-medium">{{ group.label }}</h2>
        </header>
        <div class="divide-y divide-border">
          <div
            v-for="tool in group.items"
            :key="tool.id"
            class="grid min-h-16 items-center gap-3 px-3 py-2.5 md:grid-cols-[minmax(0,1fr)_minmax(120px,220px)_auto]"
          >
            <div class="min-w-0">
              <div class="text-[13px] font-medium">{{ tool.label }}</div>
              <p class="mt-0.5 text-xs text-muted-foreground">{{ tool.description }}</p>
            </div>
            <output class="min-w-0 truncate font-mono text-[11px] text-muted-foreground">
              {{ results[tool.id] || '—' }}
            </output>
            <Button
              :variant="tool.destructive ? 'destructive' : 'outline'"
              size="sm"
              :disabled="busy === tool.id"
              @click="requestAction(tool)"
            >
              {{ busy === tool.id ? 'Working' : tool.actionLabel }}
            </Button>
          </div>
        </div>
      </section>
    </div>

    <div v-if="pending" class="fixed inset-0 z-50 grid place-items-center bg-background/85 p-4" role="presentation">
      <section class="w-full max-w-md border border-border bg-popover text-popover-foreground" role="alertdialog" aria-modal="true">
        <header class="border-b border-border px-4 py-3">
          <h2 class="text-sm font-semibold">{{ pending.confirmTitle }}</h2>
        </header>
        <p class="px-4 py-4 text-[13px] text-muted-foreground">{{ pending.confirmText }}</p>
        <footer class="flex justify-end gap-2 border-t border-border p-3">
          <Button variant="outline" size="sm" @click="pending = null">Cancel</Button>
          <Button :variant="pending.destructive ? 'destructive' : 'default'" size="sm" @click="runPending">
            {{ pending.actionLabel }}
          </Button>
        </footer>
      </section>
    </div>
  </main>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button/index.js'
import { controllerMeta, tools } from '@/tauri-adapter.js'

const busy = ref('')
const pending = ref(null)
const results = reactive({})

const utilityActions = [
  {
    id: 'controller-diagnostics',
    label: 'Controller diagnostics',
    description: 'Open the native controller state and latency inspector.',
    actionLabel: 'Open',
    run: async () => {
      await controllerMeta.launch()
      return 'Opened'
    },
  },
  {
    id: 'stylus-diagnostics',
    label: 'Stylus diagnostics',
    description: 'Open the native pen-input event inspector.',
    actionLabel: 'Open',
    run: async () => {
      await tools.openNativeTool('alkaidlab.stylus')
      return 'Opened'
    },
  },
  {
    id: 'clipboard',
    label: 'Clipboard bridge',
    description: 'Read the current host-to-client clipboard synchronization state.',
    actionLabel: 'Check',
    run: async () => {
      const status = await invoke('clipboard_sync_status')
      return typeof status === 'string' ? status : JSON.stringify(status)
    },
  },
  {
    id: 'graphics',
    label: 'Graphics driver',
    description: 'Restart the Windows graphics stack if capture or display output is stuck.',
    actionLabel: 'Restart',
    confirmTitle: 'Restart the graphics driver?',
    confirmText: 'Displays may go black briefly and active streams may be interrupted.',
    run: () => tools.restartGraphicsDriver(),
  },
  {
    id: 'sunshine',
    label: 'Sunshine service',
    description: 'Restart the host service without installing or updating anything.',
    actionLabel: 'Restart',
    confirmTitle: 'Restart Sunshine?',
    confirmText: 'Active streams will end while the host service restarts.',
    destructive: true,
    run: () => tools.restartSunshineService(),
  },
  {
    id: 'administrator',
    label: 'Administrator mode',
    description: 'Restart only this control panel with elevated permissions.',
    actionLabel: 'Elevate',
    confirmTitle: 'Restart as administrator?',
    confirmText: 'Windows will request administrator approval and this panel will close.',
    run: () => invoke('restart_as_admin'),
  },
]

const utilityGroups = [
  {
    id: 'diagnostics',
    label: 'Diagnostics',
    items: utilityActions.filter((tool) => ['controller-diagnostics', 'stylus-diagnostics', 'clipboard'].includes(tool.id)),
  },
  {
    id: 'recovery',
    label: 'Recovery and access',
    items: utilityActions.filter((tool) => ['graphics', 'sunshine', 'administrator'].includes(tool.id)),
  },
]

function requestAction(tool) {
  if (tool.confirmTitle) {
    pending.value = tool
    return
  }
  runAction(tool)
}

async function runAction(tool) {
  busy.value = tool.id
  results[tool.id] = 'Working'
  try {
    const value = await tool.run()
    results[tool.id] = value == null || value === '' ? 'Complete' : String(value)
  } catch (error) {
    results[tool.id] = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = ''
  }
}

async function runPending() {
  const tool = pending.value
  pending.value = null
  if (tool) await runAction(tool)
}
</script>
