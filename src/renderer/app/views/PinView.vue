<template>
  <main class="grid min-h-full place-items-center bg-background/95 p-5 text-foreground">
    <section class="w-full max-w-[420px] border border-border bg-card">
      <header class="border-b border-border px-4 py-3">
        <h1 class="text-sm font-semibold">Pair a client</h1>
        <p class="mt-0.5 text-xs text-muted-foreground">
          Enter the four-digit PIN shown by Moonlight.
        </p>
      </header>

      <div class="p-4">
        <div v-if="selectedPairing" class="mb-3 border border-border bg-background px-3 py-2">
          <label v-if="pendingPairings.length > 1" class="grid gap-1 text-xs text-muted-foreground">
            Pairing request
            <select
              v-model="selectedPairingId"
              class="h-8 border border-input bg-background px-2 text-xs text-foreground outline-none focus-visible:outline-2 focus-visible:outline-ring"
              :disabled="busy"
              @change="handlePairingSelection"
            >
              <option v-for="pairing in pendingPairings" :key="pairing.id" :value="pairing.id">
                {{ pairingRequestLabel(pairing) }}
              </option>
            </select>
          </label>
          <div v-else class="grid gap-0.5 text-xs">
            <span class="text-muted-foreground">Pairing request</span>
            <span>{{ pairingRequestLabel(selectedPairing) }}</span>
          </div>
        </div>

        <div class="grid grid-cols-4 gap-2" @paste="handlePaste">
          <input
            v-for="(_, index) in digits"
            :key="index"
            :ref="(element) => setInputRef(element, index)"
            v-model="digits[index]"
            class="h-14 min-w-0 border border-input bg-background text-center font-mono text-xl text-foreground outline-none focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-45"
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="1"
            :disabled="busy || !ready || !selectedPairing"
            :aria-label="`PIN digit ${index + 1}`"
            @input="handleInput(index)"
            @keydown.backspace="handleBackspace(index)"
          />
        </div>

        <div class="mt-3 min-h-5 text-xs" :class="statusClass" role="status" aria-live="polite">
          {{ status }}
        </div>
      </div>

      <footer class="flex justify-end border-t border-border p-3">
        <Button variant="outline" size="sm" :disabled="busy" @click="cancelPairing">Cancel</Button>
      </footer>
    </section>
  </main>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button/index.js'
import {
  approvePendingPairing,
  cancelPendingPairing,
  choosePendingPairing,
  getPendingPairings,
  pairingRequestLabel,
} from './pinPairing.js'

const POLL_INTERVAL_MS = 2000
const digits = ref(['', '', '', ''])
const inputRefs = []
const proxyUrl = ref('http://localhost:48081')
const clientName = ref('Moonlight client')
const defaultClientName = ref('Moonlight client')
const pendingPairings = ref([])
const selectedPairingId = ref('')
const ready = ref(false)
const busy = ref(false)
const refreshing = ref(false)
const status = ref('Starting the local pairing service...')
const statusTone = ref('neutral')
const selectedPairing = computed(
  () => pendingPairings.value.find((pairing) => pairing.id === selectedPairingId.value) ?? null,
)
const statusClass = computed(() => ({
  'text-muted-foreground': statusTone.value === 'neutral',
  'text-destructive': statusTone.value === 'danger',
  'text-foreground': statusTone.value === 'success',
}))
let pollTimer = null
let disposed = false

function setInputRef(element, index) {
  if (element) inputRefs[index] = element
}

function focusInput(index) {
  nextTick(() => inputRefs[Math.max(0, Math.min(3, index))]?.focus())
}

function clearDigits() {
  digits.value = ['', '', '', '']
}

function describeSelectedPairing() {
  if (!selectedPairing.value) {
    status.value = 'Waiting for Moonlight to request pairing...'
    statusTone.value = 'neutral'
    return
  }

  clientName.value = selectedPairing.value.name || defaultClientName.value
  status.value = `Enter the PIN for ${pairingRequestLabel(selectedPairing.value)}.`
  statusTone.value = 'neutral'
  focusInput(0)
}

function handlePairingSelection() {
  clearDigits()
  describeSelectedPairing()
}

async function refreshPendingPairings({ silent = false } = {}) {
  if (!ready.value || refreshing.value || disposed) return false
  refreshing.value = true
  const previousSelection = selectedPairingId.value
  try {
    const pairings = await getPendingPairings(proxyUrl.value)
    if (disposed) return false
    pendingPairings.value = pairings
    const selection = choosePendingPairing(pairings, previousSelection)
    selectedPairingId.value = selection?.id ?? ''

    if (selectedPairingId.value !== previousSelection) clearDigits()
    if (!silent || selectedPairingId.value !== previousSelection) describeSelectedPairing()
    return true
  } catch (error) {
    if (!silent || !selectedPairing.value) {
      status.value = error instanceof Error ? error.message : 'The local pairing service is unavailable.'
      statusTone.value = 'danger'
    }
    return false
  } finally {
    refreshing.value = false
  }
}

function handleInput(index) {
  digits.value[index] = digits.value[index].replace(/\D/g, '').slice(-1)
  if (digits.value[index] && index < 3) focusInput(index + 1)
  if (digits.value.every(Boolean)) submitPin()
}

function handleBackspace(index) {
  if (!digits.value[index] && index > 0) focusInput(index - 1)
}

function handlePaste(event) {
  const pin = event.clipboardData?.getData('text').replace(/\D/g, '').slice(0, 4) || ''
  if (!pin) return
  event.preventDefault()
  digits.value = Array.from({ length: 4 }, (_, index) => pin[index] || '')
  focusInput(Math.min(pin.length, 3))
  if (pin.length === 4) submitPin()
}

async function closeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch {
    window.close()
  }
}

async function submitPin() {
  const pin = digits.value.join('')
  const pairingId = selectedPairingId.value
  if (!ready.value || busy.value || !pairingId || !/^\d{4}$/.test(pin)) return
  busy.value = true
  status.value = 'Pairing...'
  statusTone.value = 'neutral'
  try {
    await approvePendingPairing(proxyUrl.value, {
      pairingId,
      pin,
      name: clientName.value,
    })
    status.value = 'Client paired.'
    statusTone.value = 'success'
    if (pollTimer) window.clearInterval(pollTimer)
    window.setTimeout(closeWindow, 1200)
  } catch (error) {
    clearDigits()
    await refreshPendingPairings({ silent: true })
    status.value = error instanceof Error ? error.message : String(error)
    statusTone.value = 'danger'
    busy.value = false
    if (selectedPairing.value) focusInput(0)
  }
}

async function cancelPairing() {
  if (busy.value) return
  busy.value = true
  const pairingId = selectedPairingId.value
  try {
    if (pairingId) await cancelPendingPairing(proxyUrl.value, pairingId)
  } catch {
    // Closing the local prompt remains available if the request already expired.
  } finally {
    await closeWindow()
  }
}

onMounted(async () => {
  document.title = 'PIN pairing'
  if (!('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
    status.value = 'Native host APIs are unavailable in browser preview.'
    statusTone.value = 'danger'
    return
  }
  try {
    await invoke('wait_for_proxy_ready')
    proxyUrl.value = await invoke('get_proxy_url_command')
    const response = await fetch(`${proxyUrl.value}/api/config`)
    if (!response.ok) throw new Error(`Local host returned ${response.status}`)
    const config = await response.json()
    defaultClientName.value = config.pair_name || config.sunshine_name || defaultClientName.value
    clientName.value = defaultClientName.value
    ready.value = true
    await refreshPendingPairings()
    pollTimer = window.setInterval(() => refreshPendingPairings({ silent: true }), POLL_INTERVAL_MS)
  } catch (error) {
    status.value = error instanceof Error ? error.message : 'The local pairing service is unavailable.'
    statusTone.value = 'danger'
  }
})

onBeforeUnmount(() => {
  disposed = true
  if (pollTimer) window.clearInterval(pollTimer)
})
</script>
