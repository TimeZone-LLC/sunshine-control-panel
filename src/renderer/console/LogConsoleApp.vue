<template>
  <div class="grid h-full min-h-0 grid-rows-[40px_auto_auto_minmax(0,1fr)_32px] bg-background font-mono text-xs text-foreground">
    <header class="flex items-center justify-between gap-3 border-b border-border bg-card px-3">
      <div class="flex min-w-0 items-center gap-3">
        <h1 class="font-sans text-[13px] font-semibold">{{ logText.title }}</h1>
        <span v-if="notice" class="truncate text-[11px]" :class="noticeTone === 'danger' ? 'text-destructive' : 'text-muted-foreground'">
          {{ notice }}
        </span>
      </div>
      <div class="flex shrink-0 items-center gap-1">
        <Button variant="outline" size="sm" :disabled="loading" @click="loadLogs">Refresh</Button>
        <Button variant="outline" size="sm" @click="exportLogs('txt')">Export text</Button>
        <Button variant="outline" size="sm" @click="exportLogs('json')">Export JSON</Button>
        <Button variant="destructive" size="sm" @click="clearLogs">Clear</Button>
      </div>
    </header>

    <div class="flex min-h-8 flex-wrap items-center gap-x-4 gap-y-1 border-b border-border bg-card px-3 py-1 font-sans">
      <span class="text-muted-foreground">Level</span>
      <label v-for="level in levels" :key="level" class="flex h-6 items-center gap-1.5 px-1 text-xs hover:bg-accent">
        <input v-model="filters[level]" type="checkbox" class="size-3 accent-foreground" />
        <span :class="levelClass(level)">{{ level }}</span>
      </label>
      <span class="ml-auto text-muted-foreground">Source</span>
      <select v-model="filters.file" class="h-6 min-w-44 border border-input bg-background px-1 font-mono text-xs outline-none focus-visible:outline-2 focus-visible:outline-ring">
        <option value="">All files</option>
        <option v-for="file in availableFiles" :key="file" :value="file">{{ file }}</option>
      </select>
    </div>

    <div class="flex min-h-8 items-center gap-2 border-b border-border bg-card px-3 py-1">
      <div class="flex h-6 min-w-0 flex-1 items-center border border-input bg-background focus-within:outline-2 focus-within:outline-ring">
        <input
          v-model="searchKeyword"
          type="search"
          class="h-full min-w-0 flex-1 bg-transparent px-2 font-mono text-xs outline-none placeholder:text-muted-foreground"
          :placeholder="logText.searchPlaceholder"
          @input="handleSearchInput"
        />
        <Button v-if="searchKeyword" variant="ghost" size="icon" class="size-5 border-0" :aria-label="logText.clearSearch" @click="clearSearch">
          <X :stroke-width="1.5" />
        </Button>
      </div>
      <span v-if="searchKeyword" class="shrink-0 text-muted-foreground">
        {{ filteredLogs.length }} matches
      </span>
    </div>

    <div ref="logContainer" class="relative min-h-0 overflow-auto bg-background" @scroll="handleLogScroll">
      <div v-if="filteredLogs.length === 0" class="grid h-full place-items-center font-sans text-muted-foreground">
        {{ loading ? logText.loadingLogs : searchKeyword ? logText.noMatch : logText.noLogs }}
      </div>
      <div v-else class="relative min-h-full" :style="{ height: `${totalHeight}px` }">
        <div class="absolute inset-x-0 top-0" :style="{ transform: `translateY(${visibleOffset}px)` }">
          <div
            v-for="{ log, virtualIndex } in visibleLogs"
            :key="`${virtualIndex}-${log.timestamp}-${log.message}`"
            class="grid h-6 grid-cols-[150px_48px_minmax(100px,220px)_minmax(0,1fr)] items-center gap-2 overflow-hidden border-b border-border px-3 hover:bg-accent"
            :class="levelClass(log.level)"
          >
            <span class="truncate text-muted-foreground">{{ log.timestamp }}</span>
            <span class="truncate">{{ log.level }}</span>
            <span class="truncate text-muted-foreground">{{ log.file }}<template v-if="log.line">:{{ log.line }}</template></span>
            <span class="truncate" v-html="highlightMessage(log.message)"></span>
          </div>
        </div>
      </div>
    </div>

    <footer class="flex items-center gap-5 border-t border-border bg-card px-3 font-sans text-[11px] text-muted-foreground">
      <span>Total <b class="font-mono font-normal text-foreground">{{ stats.total }}</b></span>
      <span class="text-destructive">Errors <b class="font-mono font-normal">{{ stats.error }}</b></span>
      <span class="text-warning">Warnings <b class="font-mono font-normal">{{ stats.warn }}</b></span>
      <span>Info <b class="font-mono font-normal text-foreground">{{ stats.info }}</b></span>
    </footer>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watchEffect } from 'vue'
import { X } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { confirm as dialogConfirm, message as dialogMessage } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button/index.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()
const logText = computed(() => t.value.logConsole)
const allLogs = ref([])
const loading = ref(false)
const logContainer = ref(null)
const searchKeyword = ref('')
const committedSearchKeyword = ref('')
const scrollTop = ref(0)
const viewportHeight = ref(1)
const notice = ref('')
const noticeTone = ref('neutral')
const levels = ['error', 'warn', 'info', 'debug', 'trace']
const filters = ref({ error: true, warn: true, info: true, debug: false, trace: false, file: '' })

const LOG_ROW_HEIGHT = 24
const LOG_OVERSCAN = 8
const SEARCH_DEBOUNCE_MS = 120
let searchTimer = null
let noticeTimer = null
let unsubscribe = null

watchEffect(() => {
  document.title = logText.value.title
})

const logSummary = computed(() => {
  const files = new Set()
  const counts = { total: allLogs.value.length, error: 0, warn: 0, info: 0 }
  for (const log of allLogs.value) {
    if (log.file) files.add(log.file)
    if (log.level in counts) counts[log.level] += 1
  }
  return { files: [...files].sort(), stats: counts }
})
const availableFiles = computed(() => logSummary.value.files)
const stats = computed(() => logSummary.value.stats)

const filteredLogs = computed(() => {
  const enabled = levels.filter((level) => filters.value[level])
  const keyword = committedSearchKeyword.value.trim().toLowerCase()
  return allLogs.value.filter((log) => {
    if (!enabled.includes(log.level)) return false
    if (filters.value.file && log.file !== filters.value.file) return false
    if (!keyword) return true
    return [log.message, log.timestamp, log.level, log.file || '']
      .some((value) => String(value).toLowerCase().includes(keyword))
  })
})

const totalHeight = computed(() => filteredLogs.value.length * LOG_ROW_HEIGHT)
const visibleStartIndex = computed(() => Math.max(0, Math.floor(scrollTop.value / LOG_ROW_HEIGHT) - LOG_OVERSCAN))
const visibleEndIndex = computed(() => Math.min(
  filteredLogs.value.length,
  Math.ceil((scrollTop.value + viewportHeight.value) / LOG_ROW_HEIGHT) + LOG_OVERSCAN,
))
const visibleOffset = computed(() => visibleStartIndex.value * LOG_ROW_HEIGHT)
const visibleLogs = computed(() => filteredLogs.value.slice(visibleStartIndex.value, visibleEndIndex.value).map((log, index) => ({
  log,
  virtualIndex: visibleStartIndex.value + index,
})))

function levelClass(level) {
  if (level === 'error') return 'text-destructive'
  if (level === 'warn') return 'text-warning'
  if (level === 'debug' || level === 'trace') return 'text-muted-foreground'
  return 'text-foreground'
}

function escapeHtml(text) {
  const element = document.createElement('div')
  element.textContent = String(text)
  return element.innerHTML
}

function highlightMessage(message) {
  const keyword = committedSearchKeyword.value.trim()
  if (!keyword) return escapeHtml(message)
  const escaped = keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return escapeHtml(message).replace(new RegExp(`(${escaped})`, 'gi'), '<mark class="bg-foreground text-background">$1</mark>')
}

function showNotice(text, tone = 'neutral') {
  clearTimeout(noticeTimer)
  notice.value = text
  noticeTone.value = tone
  noticeTimer = window.setTimeout(() => { notice.value = '' }, 3500)
}

function handleSearchInput() {
  clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => {
    committedSearchKeyword.value = searchKeyword.value
    if (logContainer.value) logContainer.value.scrollTop = 0
  }, SEARCH_DEBOUNCE_MS)
}

function clearSearch() {
  clearTimeout(searchTimer)
  searchKeyword.value = ''
  committedSearchKeyword.value = ''
  if (logContainer.value) logContainer.value.scrollTop = 0
}

function handleLogScroll() {
  scrollTop.value = logContainer.value?.scrollTop || 0
}

function updateViewportHeight() {
  viewportHeight.value = logContainer.value?.clientHeight || 1
}

async function loadLogs() {
  loading.value = true
  try {
    allLogs.value = await invoke('get_all_logs')
    showNotice(`${allLogs.value.length} entries loaded`)
  } catch (error) {
    showNotice(String(error), 'danger')
  } finally {
    loading.value = false
  }
}

async function clearLogs() {
  if (!await dialogConfirm(logText.value.confirmClear, { title: logText.value.clearLogsTitle, kind: 'warning' })) return
  try {
    await invoke('clear_logs')
    allLogs.value = []
    showNotice(logText.value.logsCleared)
  } catch (error) {
    await dialogMessage(String(error), { title: logText.value.dialogError, kind: 'error' })
  }
}

async function exportLogs(format) {
  try {
    const result = await invoke('export_logs', { format })
    showNotice(result || logText.value.exportSuccess)
  } catch (error) {
    if (!String(error).toLowerCase().includes('cancel')) showNotice(String(error), 'danger')
  }
}

function scrollToBottom() {
  nextTick(() => {
    if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight
  })
}

onMounted(async () => {
  if (!('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
    showNotice('Native log APIs are unavailable in browser preview.')
    return
  }
  await loadLogs()
  updateViewportHeight()
  window.addEventListener('resize', updateViewportHeight)
  scrollToBottom()
  unsubscribe = await listen('log-entry', (event) => {
    allLogs.value.push(event.payload)
    if (allLogs.value.length > 10000) allLogs.value = allLogs.value.slice(-10000)
    scrollToBottom()
  })
})

onUnmounted(() => {
  clearTimeout(searchTimer)
  clearTimeout(noticeTimer)
  window.removeEventListener('resize', updateViewportHeight)
  unsubscribe?.()
})
</script>
