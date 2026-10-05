<template>
  <Teleport to="body">
    <Transition name="cover-modal">
      <div v-if="open" class="cover-modal-mask" @click.self="$emit('close')">
        <div ref="modalRef" class="cover-modal">
          <div class="modal-header">
            <h3><Picture /> Update cover art — {{ appName }}</h3>
            <button class="modal-close" @click="$emit('close')">&times;</button>
          </div>

          <!-- Search bar -->
          <div class="search-bar">
            <input
              v-model="searchQuery"
              class="search-input"
              placeholder="Search Steam games..."
              @keyup.enter="doSearch"
            />
            <button class="search-btn" @click="doSearch" :disabled="searching">
              {{ searching ? 'Searching...' : 'Search' }}
            </button>
          </div>

          <!-- Search results -->
          <div v-if="error" class="search-error">{{ error }}</div>

          <div v-if="candidates.length > 0" class="candidates-grid">
            <div
              v-for="c in candidates"
              :key="c.steam_id"
              class="candidate-card"
              :class="{ selected: selected?.steam_id === c.steam_id, uploading: uploadingId === c.steam_id }"
              @click="selectAndUpload(c)"
            >
              <img :src="c.header_url" class="candidate-img" loading="lazy" />
              <div class="candidate-name">{{ c.name }}</div>
              <div v-if="uploadingId === c.steam_id" class="uploading-overlay">
                <div class="upload-spinner"></div>
              </div>
            </div>
          </div>

          <div v-else-if="!searching && !error && searched" class="no-results">
            No matches found
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch } from 'vue'
import { Picture } from '@element-plus/icons-vue'
import { tauriInvoke } from '../composables/useTauri'
import { useModalFocusScope } from '../composables/useFocusNav.js'

const props = defineProps({
  open: { type: Boolean, required: true },
  appName: { type: String, default: '' },
  proxyUrl: { type: String, default: '' },
})

const emit = defineEmits(['close', 'updated'])

const modalRef = ref(null)
useModalFocusScope(modalRef, () => props.open)

const searchQuery = ref('')
const candidates = ref([])
const selected = ref(null)
const searching = ref(false)
const searched = ref(false)
const error = ref('')
const uploadingId = ref(null)

// Search for the application name automatically when the modal opens.
watch(() => props.open, (v) => {
  if (v && props.appName) {
    searchQuery.value = props.appName
    doSearch()
  } else if (!v) {
    candidates.value = []
    selected.value = null
    error.value = ''
    searched.value = false
    uploadingId.value = null
  }
})

async function doSearch() {
  if (!searchQuery.value.trim()) return
  searching.value = true
  error.value = ''
  candidates.value = []
  searched.value = false
  selected.value = null

  try {
    candidates.value = await tauriInvoke('search_steam_covers', {
      query: searchQuery.value.trim(),
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    searching.value = false
    searched.value = true
  }
}

async function selectAndUpload(candidate) {
  if (uploadingId.value) return
  selected.value = candidate
  uploadingId.value = candidate.steam_id

  try {
    await tauriInvoke('upload_steam_cover', {
      headerUrl: candidate.header_url,
      appName: props.appName,
      proxyUrl: props.proxyUrl,
    })
    emit('updated', props.appName)
    emit('close')
  } catch (e) {
    error.value = `Upload failed: ${e}`
  } finally {
    uploadingId.value = null
  }
}
</script>

<style lang="less" scoped>
@import "@/components/spinner.less";

.cover-modal-mask {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: var(--scrim);
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-modal {
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  width: 680px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--row-height);
  padding: 0 4px 0 10px;
  border-bottom: 1px solid var(--border);

  h3 {
    margin: 0;
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text);
    display: flex;
    align-items: center;
    gap: 6px;

    :deep(svg) {
      width: 12px;
      height: 12px;
      color: var(--text-dim);
    }
  }

  .modal-close {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-size: var(--font-size);
    line-height: 1;
    cursor: pointer;
    width: 20px;
    height: 20px;

    &:hover {
      color: var(--text);
      background: var(--bg-hover);
    }
  }
}

.search-bar {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);

  .search-input {
    flex: 1;
    height: 24px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    padding: 0 8px;
    color: var(--text);
    font-size: var(--font-size-sm);
    outline: none;

    &::placeholder { color: var(--text-faint); }

    &:focus { border-color: var(--border-strong); }
  }

  .search-btn {
    height: 24px;
    padding: 0 10px;
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text);
    font-size: var(--font-size-sm);
    cursor: pointer;
    white-space: nowrap;

    &:hover:not(:disabled) { background: var(--bg-hover); border-color: var(--border-strong); }
    &:disabled { color: var(--text-faint); cursor: default; }
  }
}

.search-error {
  padding: 6px 10px;
  color: var(--error);
  font-size: var(--font-size-sm);
}

.candidates-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  padding: 8px 10px 10px;
  overflow-y: auto;
  max-height: 50vh;
}

.candidate-card {
  position: relative;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid var(--border);

  &:hover {
    border-color: var(--border-strong);
  }

  &.selected {
    border-color: var(--border-strong);
  }

  &.uploading {
    pointer-events: none;
  }

  .candidate-img {
    width: 100%;
    aspect-ratio: 460 / 215;
    object-fit: cover;
    display: block;
  }

  .candidate-name {
    padding: 3px 6px;
    background: var(--bg);
    border-top: 1px solid var(--border);
    color: var(--text);
    font-size: var(--font-size-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .uploading-overlay {
    position: absolute;
    inset: 0;
    background: var(--scrim);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .upload-spinner {
    .gloss-spinner();
  }
}

.no-results {
  padding: 16px 10px;
  color: var(--text-faint);
  font-size: var(--font-size-sm);
}

.cover-modal-enter-active,
.cover-modal-leave-active {
  transition: none;
}
</style>
