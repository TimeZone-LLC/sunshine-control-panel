<template>
  <SectionPanel :title="t.rtssTool.osdControl">
    <template #icon><ChatLineSquare /></template>
    <template #actions>
      <button v-if="hasCli" class="overlay-btn" @click="onToggleOverlay">
        {{ t.rtssTool.toggleOverlay }}
      </button>
    </template>

    <!-- Formatting toolbar -->
    <div class="format-toolbar">
      <div class="format-group">
        <button class="fmt-btn" @click="insertTag('B')" title="Bold"><b>B</b></button>
        <button class="fmt-btn" @click="insertTag('I')" title="Italic"><i>I</i></button>
      </div>
      <div class="format-group">
        <label class="color-btn" :title="t.rtssTool.textColor">
          <span class="color-preview" :style="{ background: '#' + color }"></span>
          <span>A</span>
          <input type="color" :value="'#' + color" @input="onColorPick" class="hidden-input" />
        </label>
        <button class="fmt-btn" @click="insertColorTag" :title="t.rtssTool.insertColor"><Brush /></button>
      </div>
      <div class="format-group">
        <select v-model="fontSize" class="fmt-select" @change="insertSizeTag" :title="t.rtssTool.fontSize">
          <option :value="0">{{ t.rtssTool.defaultSize }}</option>
          <option :value="12">12px</option>
          <option :value="16">16px</option>
          <option :value="20">20px</option>
          <option :value="24">24px</option>
          <option :value="32">32px</option>
        </select>
      </div>
      <div class="format-group">
        <button type="button" class="fmt-btn fmt-tag" title="Color" aria-label="Color" @click="insertRawTag('<C=>')">&lt;C=&gt;</button>
        <button type="button" class="fmt-btn fmt-tag" title="Size" aria-label="Size" @click="insertRawTag('<S=>')">&lt;S=&gt;</button>
        <button type="button" class="fmt-btn fmt-tag" title="Newline" aria-label="Newline" @click="insertRawTag('\\n')">↵</button>
      </div>
    </div>

    <div class="osd-input-group">
      <label class="field-label">{{ t.rtssTool.osdText }}</label>
      <textarea
        ref="textarea"
        v-model="text"
        class="osd-textarea"
        :placeholder="t.rtssTool.osdPlaceholder"
        rows="4"
      ></textarea>
      <div class="osd-hint">{{ t.rtssTool.osdHint }}</div>
    </div>

    <div class="actions">
      <button @click="applyOsd" class="apply-btn" :disabled="applying">
        {{ applying ? t.rtssTool.applying : t.rtssTool.applyOsd }}
      </button>
      <button @click="clearOsd" class="reset-btn" :disabled="applying">
        {{ t.rtssTool.clearOsd }}
      </button>
    </div>
  </SectionPanel>
</template>

<script setup>
import { ref } from 'vue'
import { ChatLineSquare, Brush } from '../../../components/ui/compact/icons.js'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n/index.js'
import SectionPanel from '../../components/SectionPanel.vue'

const { t } = useI18n()
const emit = defineEmits(['message'])

defineProps({
  hasCli: { type: Boolean, default: false },
})

const text = ref('')
const applying = ref(false)
const textarea = ref(null)
const color = ref('FFFFFF')
const fontSize = ref(0)

// ─── Format tag insertion ───
function insertTag(tag) {
  const ta = textarea.value
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const val = text.value
  const selected = val.substring(start, end)
  const insertion = `<${tag}>${selected}<${tag}0>`
  text.value = val.substring(0, start) + insertion + val.substring(end)
  const newPos = start + tag.length + 2 + selected.length
  requestAnimationFrame(() => { ta.focus(); ta.setSelectionRange(newPos, newPos) })
}

function onColorPick(e) {
  color.value = e.target.value.replace('#', '').toUpperCase()
}

function insertColorTag() {
  insertRawTag(`<C=${color.value}>`)
}

function insertSizeTag() {
  if (fontSize.value > 0) insertRawTag(`<S=${fontSize.value}>`)
}

function insertRawTag(tag) {
  const ta = textarea.value
  if (!ta) return
  const start = ta.selectionStart
  const val = text.value
  text.value = val.substring(0, start) + tag + val.substring(start)
  const newPos = start + tag.length
  requestAnimationFrame(() => { ta.focus(); ta.setSelectionRange(newPos, newPos) })
}

// ─── OSD actions ───
async function applyOsd() {
  applying.value = true
  try {
    await invoke('rtss_set_osd', { text: text.value, owner: 'Sunshine' })
    emit('message', t.value.rtssTool.osdApplied, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  } finally {
    applying.value = false
  }
}

async function clearOsd() {
  applying.value = true
  try {
    await invoke('rtss_clear_osd', { owner: 'Sunshine' })
    text.value = ''
    emit('message', t.value.rtssTool.osdCleared, 'success')
  } catch (e) {
    emit('message', String(e), 'error')
  } finally {
    applying.value = false
  }
}

async function onToggleOverlay() {
  try {
    const result = await invoke('rtss_toggle_overlay')
    emit('message',
      result === '1' ? t.value.rtssTool.overlayShown : t.value.rtssTool.overlayHidden,
      'success'
    )
  } catch (e) {
    emit('message', String(e), 'error')
  }
}
</script>

<style lang="less" scoped>
.overlay-btn {
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;
  flex-shrink: 0;

  &:hover { background: var(--bg-hover); color: var(--text); }
}

// Format toolbar
.format-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 6px 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
}

.format-group {
  display: flex;
  align-items: center;
  gap: 3px;
  padding-right: 6px;
  border-right: 1px solid var(--border);

  &:last-child { border-right: none; padding-right: 0; }
}

.fmt-btn {
  padding: 4px 8px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;
  min-width: 28px;
  text-align: center;

  &:hover { background: var(--bg-hover); color: var(--text); }
}

.fmt-tag {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 6px;
}

.color-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  position: relative;

  &:hover { background: var(--bg-hover); color: var(--text); }
  &:focus-within { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
}

.color-preview {
  width: 14px;
  height: 14px;
  border: 1px solid var(--border-strong);
}

.hidden-input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  pointer-events: none;
}

.fmt-select {
  padding: 4px 6px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  outline: none;

  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
  option { background: var(--bg-panel); color: var(--text); }
}

// OSD input
.osd-input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-dim);
}

.osd-textarea {
  width: 100%;
  padding: 8px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.5;
  resize: vertical;
  outline: none;
  transition: border-color 120ms ease;
  box-sizing: border-box;

  &:focus-visible { border-color: var(--border-strong); outline: 2px solid var(--text); outline-offset: 0; }
  &::placeholder { color: var(--text-faint); }
}

.osd-hint {
  font-size: 11px;
  color: var(--text-faint);
}

// Actions
.actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.apply-btn {
  height: @control-height;
  padding: 0 20px;
  border: 1px solid var(--border-strong);
  background: var(--bg-active);
  color: var(--text);
  font-size: @font-size-base;
  font-weight: @font-weight-normal;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { background: var(--bg-hover); }
  &:disabled { color: var(--text-faint); border-color: var(--border); cursor: not-allowed; }
}

.reset-btn {
  height: @control-height;
  padding: 0 16px;
  border: 1px solid var(--border);
  background: var(--bg-panel);
  color: var(--text-dim);
  font-size: @font-size-base;
  font-weight: @font-weight-normal;
  cursor: pointer;
  transition: background-color 120ms ease, color 120ms ease, border-color 120ms ease;

  &:hover { background: var(--bg-hover); color: var(--text); }
  &:disabled { color: var(--text-faint); cursor: not-allowed; }
}
</style>
