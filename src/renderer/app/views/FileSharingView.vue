<template>
  <main class="native-page text-foreground">
    <section class="border border-border bg-card">
      <header class="flex items-center justify-between gap-4 border-b border-border p-3">
        <p class="max-w-2xl text-xs text-muted-foreground">
          Paired clients receive read-only access. Filesystem links are never followed.
        </p>
        <div class="flex shrink-0 gap-1">
          <Button variant="outline" size="sm" :disabled="busy" @click="loadMappings">Refresh</Button>
          <Button size="sm" :disabled="busy || !nativeRuntime" @click="addLocation">Expose location</Button>
        </div>
      </header>

      <div v-if="message" class="border-b border-border px-3 py-2 text-xs" :class="messageTone === 'danger' ? 'text-destructive' : 'text-muted-foreground'">
        {{ message }}
      </div>

      <div v-if="loading" class="p-6 text-center text-xs text-muted-foreground">Loading file access...</div>
      <div v-else-if="mappings.length === 0" class="p-6 text-center text-xs text-muted-foreground">
        No filesystem locations are exposed.
      </div>
      <div v-else class="divide-y divide-border">
        <div v-for="mapping in mappings" :key="mapping.id" class="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-3 p-3">
          <div class="min-w-0">
            <div class="truncate text-[13px] font-medium">{{ mapping.name || mapping.id }}</div>
            <div class="truncate font-mono text-[11px] text-muted-foreground" :title="mapping.path">{{ mapping.path }}</div>
          </div>
          <div class="flex gap-1">
            <Badge>Read only</Badge>
            <Badge>{{ mapping.clients?.length ? `${mapping.clients.length} clients` : 'All paired clients' }}</Badge>
          </div>
          <Button variant="destructive" size="sm" :disabled="busy" @click="pendingRemoval = mapping">Revoke</Button>
        </div>
      </div>

      <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-border p-3">
        <span class="text-xs text-muted-foreground">Windows Explorer action</span>
        <div class="flex gap-1">
          <Button variant="outline" size="sm" :disabled="busy || !nativeRuntime" @click="setExplorerMenu(true)">Install</Button>
          <Button variant="outline" size="sm" :disabled="busy || !nativeRuntime" @click="setExplorerMenu(false)">Remove</Button>
        </div>
      </footer>
    </section>

    <div v-if="pendingRemoval" class="fixed inset-0 z-50 grid place-items-center bg-background/85 p-4">
      <section class="w-full max-w-md border border-border bg-popover" role="alertdialog" aria-modal="true">
        <header class="border-b border-border px-4 py-3 text-sm font-semibold">Remove this access rule?</header>
        <p class="px-4 py-4 text-[13px] text-muted-foreground">
          Paired clients will no longer be able to read {{ pendingRemoval.name || pendingRemoval.path }}.
        </p>
        <footer class="flex justify-end gap-2 border-t border-border p-3">
          <Button variant="outline" size="sm" @click="pendingRemoval = null">Cancel</Button>
          <Button variant="destructive" size="sm" @click="removeMapping">Revoke</Button>
        </footer>
      </section>
    </div>
  </main>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { Badge } from '@/components/ui/badge/index.js'
import { Button } from '@/components/ui/button/index.js'
import { fileMapping } from '@/tauri-adapter.js'

const nativeRuntime = '__TAURI_INTERNALS__' in window || '__TAURI__' in window
const mappings = ref([])
const loading = ref(false)
const busy = ref(false)
const message = ref('')
const messageTone = ref('neutral')
const pendingRemoval = ref(null)

function showMessage(value, tone = 'neutral') {
  message.value = String(value || '')
  messageTone.value = tone
}

async function loadMappings() {
  if (!nativeRuntime) {
    showMessage('Native file-access APIs are unavailable in browser preview.')
    return
  }
  loading.value = true
  try {
    mappings.value = await fileMapping.list()
    showMessage(`${mappings.value.length} active access ${mappings.value.length === 1 ? 'rule' : 'rules'}`)
  } catch (error) {
    mappings.value = []
    showMessage(error, 'danger')
  } finally {
    loading.value = false
  }
}

async function addLocation() {
  busy.value = true
  try {
    const path = await open({ directory: true, multiple: false })
    if (!path) return
    const mapping = await fileMapping.quickShareFolder(path)
    await loadMappings()
    showMessage(`${mapping?.name || 'Location'} is now available to paired clients.`)
  } catch (error) {
    showMessage(error, 'danger')
  } finally {
    busy.value = false
  }
}

async function removeMapping() {
  const mapping = pendingRemoval.value
  pendingRemoval.value = null
  if (!mapping?.id) return
  busy.value = true
  try {
    await fileMapping.remove(mapping.id)
    mappings.value = mappings.value.filter((item) => item.id !== mapping.id)
    showMessage(`${mapping.name || 'Location'} is no longer available to paired clients.`)
  } catch (error) {
    showMessage(error, 'danger')
  } finally {
    busy.value = false
  }
}

async function setExplorerMenu(enabled) {
  busy.value = true
  try {
    if (enabled) await fileMapping.installMenu()
    else await fileMapping.uninstallMenu()
    showMessage(`Explorer integration ${enabled ? 'enabled' : 'disabled'}.`)
  } catch (error) {
    showMessage(error, 'danger')
  } finally {
    busy.value = false
  }
}

onMounted(loadMappings)
</script>
