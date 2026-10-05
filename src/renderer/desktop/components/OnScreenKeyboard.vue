<template>
  <Teleport to="body">
    <Transition name="osk">
      <div v-if="state.open" ref="overlayRef" class="osk-overlay" @keydown="onKeydown">
        <div class="osk-panel">
          <div class="osk-header">
            <span class="osk-title">{{ state.title || t.osk.title }}</span>
            <span class="osk-counter" v-if="state.maxLength">
              {{ draft.length }} / {{ state.maxLength }}
            </span>
          </div>

          <div class="osk-preview" :class="{ empty: !draft }">
            <span class="osk-preview-text">{{ previewText }}</span>
            <span class="osk-caret"></span>
          </div>

          <div class="osk-rows">
            <div v-for="(row, rowIndex) in rows" :key="'row-' + rowIndex" class="osk-row">
              <button
                v-for="key in row"
                :key="key"
                type="button"
                class="osk-key"
                data-focusable
                :data-focus-key="'osk-' + key"
                @click="append(key)"
              >
                {{ shifted ? key.toUpperCase() : key }}
              </button>
            </div>
          </div>

          <div class="osk-row osk-actions">
            <button
              v-if="state.mode === 'text'"
              type="button"
              class="osk-key wide"
              :class="{ toggled: shifted }"
              data-focusable
              data-focus-key="osk-shift"
              @click="shifted = !shifted"
            >
              {{ t.osk.shift }}
            </button>
            <button
              v-if="state.mode === 'text'"
              type="button"
              class="osk-key wide"
              :class="{ toggled: symbols }"
              data-focusable
              data-focus-key="osk-symbols"
              @click="symbols = !symbols"
            >
              {{ symbols ? t.osk.letters : t.osk.symbols }}
            </button>
            <button
              v-if="state.mode === 'text'"
              type="button"
              class="osk-key grow"
              data-focusable
              data-focus-key="osk-space"
              @click="append(' ')"
            >
              {{ t.osk.space }}
            </button>
            <button
              type="button"
              class="osk-key wide"
              data-focusable
              data-focus-key="osk-backspace"
              @click="backspace"
            >
              ⌫
            </button>
            <button
              type="button"
              class="osk-key wide"
              data-focusable
              data-focus-key="osk-clear"
              @click="draft = ''"
            >
              {{ t.osk.clear }}
            </button>
          </div>

          <div class="osk-row osk-commit">
            <button
              type="button"
              class="osk-key cancel"
              data-focusable
              data-focus-key="osk-cancel"
              @click="cancel"
            >
              {{ t.osk.cancel }}
            </button>
            <button
              type="button"
              class="osk-key confirm"
              data-focusable
              data-focus-key="osk-confirm"
              @click="commit"
            >
              {{ t.osk.confirm }}
            </button>
          </div>

          <div class="osk-hint">{{ t.osk.hint }}</div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { nextTick, ref, computed, watch, onUnmounted } from 'vue'
import { cancelOsk, commitOsk, oskState } from '../composables/useOsk.js'
import { popFocusScope, pushFocusScope } from '../composables/useFocusNav.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const state = oskState
const overlayRef = ref(null)
const draft = ref('')
const shifted = ref(false)
const symbols = ref(false)

const LETTER_ROWS = [
  ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
  ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
  ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
  ['z', 'x', 'c', 'v', 'b', 'n', 'm', '-', '_'],
]

const SYMBOL_ROWS = [
  ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'],
  ['-', '_', '=', '+', '[', ']', '{', '}', ';', ':'],
  ["'", '"', ',', '.', '<', '>', '/', '?', '\\', '|', '~', '`'],
]

const NUMBER_ROWS = [
  ['1', '2', '3'],
  ['4', '5', '6'],
  ['7', '8', '9'],
  ['0'],
]

/** 预览框只显示尾部，长文本从左侧截断，光标始终可见。 */
const PREVIEW_TAIL = 60
const previewText = computed(() => {
  if (!draft.value) return state.value.placeholder || t.value.osk.placeholder
  if (draft.value.length <= PREVIEW_TAIL) return draft.value
  return `…${draft.value.slice(-PREVIEW_TAIL)}`
})

const rows = computed(() => {
  if (state.value.mode === 'number') return NUMBER_ROWS
  return symbols.value ? SYMBOL_ROWS : LETTER_ROWS
})

let disposeScope = null

watch(
  () => state.value.open,
  async (open) => {
    if (open) {
      draft.value = state.value.value
      shifted.value = false
      symbols.value = false
      await nextTick()
      if (overlayRef.value) disposeScope = pushFocusScope(overlayRef.value)
    } else {
      disposeScope?.()
      disposeScope = null
    }
  },
  { immediate: true }
)

onUnmounted(() => {
  disposeScope?.()
})

function atLimit() {
  return state.value.maxLength > 0 && draft.value.length >= state.value.maxLength
}

function append(character) {
  if (atLimit()) return
  const value = shifted.value ? character.toUpperCase() : character
  draft.value += value
  // Shift 只作用于一个字符，和手机键盘一致
  if (shifted.value) shifted.value = false
}

function backspace() {
  draft.value = draft.value.slice(0, -1)
}

function commit() {
  commitOsk(draft.value)
}

function cancel() {
  cancelOsk()
}

/** 物理键盘依然可用：串流用手柄，本机调试用键盘。 */
function onKeydown(event) {
  if (event.key === 'Escape') {
    event.preventDefault()
    cancel()
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    commit()
    return
  }
  if (event.key === 'Backspace') {
    event.preventDefault()
    backspace()
    return
  }
  if (event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey) {
    if (state.value.mode === 'number' && !/[0-9]/.test(event.key)) return
    event.preventDefault()
    if (!atLimit()) draft.value += event.key
  }
}
</script>

<style lang="less" scoped>
.osk-overlay {
  position: fixed;
  inset: 0;
  z-index: 20000;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0 0 4vh;
  background: var(--scrim);
}

.osk-panel {
  width: min(920px, 92vw);
  padding: var(--space-4);
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.osk-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;

  .osk-title {
    font-size: var(--font-size);
    font-weight: 600;
    color: var(--text);
  }

  .osk-counter {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }
}

.osk-preview {
  min-height: 40px;
  padding: var(--space-2) var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--border);
  font-family: var(--font-mono);
  font-size: 16px;
  color: var(--text);
  display: flex;
  align-items: center;
  gap: 2px;
  overflow: hidden;

  &.empty .osk-preview-text {
    color: var(--text-faint);
  }

  .osk-preview-text {
    white-space: pre;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .osk-caret {
    width: 2px;
    height: 20px;
    background: var(--text);
    flex-shrink: 0;
  }
}

.osk-rows {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.osk-row {
  display: flex;
  gap: var(--space-2);
  justify-content: center;
}

.osk-key {
  min-width: 48px;
  height: 40px;
  padding: 0 var(--space-3);
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  font: inherit;
  cursor: pointer;
  transition: background-color var(--transition), color var(--transition), border-color var(--transition);

  &:hover,
  &:focus-visible {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  &.wide {
    min-width: 84px;
  }

  &.grow {
    flex: 1;
  }

  &.toggled,
  &.confirm {
    background: var(--bg-active);
    border-color: var(--border-strong);
    font-weight: 600;
  }

  &.confirm,
  &.cancel {
    min-width: 160px;
  }
}

.osk-commit {
  margin-top: var(--space-1);
}

.osk-hint {
  text-align: center;
  font-size: var(--font-size-xs);
  color: var(--text-faint);
}
</style>
