<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="open" class="helper-overlay" @click="$emit('close')"></div>
    </Transition>

    <Transition name="slide">
      <div v-if="open" ref="panelRef" class="helper-panel" @click.stop @keydown.escape="$emit('close')">
        <div class="panel-header">
          <h2>{{ t.launchHelper.title }}</h2>
          <span class="app-name-tag">{{ appName }}</span>
          <button class="close-btn" @click="$emit('close')">&times;</button>
        </div>

        <div class="panel-body">
          <p class="panel-desc">{{ t.launchHelper.description }}<br/>
            <small>{{ t.launchHelper.orderHint }}</small>
          </p>

          <div
            v-for="(helper, idx) in editingHelpers"
            :key="helper.templateId"
            class="helper-card"
            :class="{ active: helper.enabled }"
          >
            <div class="helper-header">
              <!-- Reorder buttons, shown only when enabled -->
              <div v-if="helper.enabled" class="order-buttons">
                <button
                  class="order-btn"
                  :disabled="idx === 0"
                  @click.stop="moveHelper(idx, -1)"
                  :title="t.launchHelper.moveUp"
                  data-focusable
                >▲</button>
                <button
                  class="order-btn"
                  :disabled="idx === editingHelpers.length - 1"
                  @click.stop="moveHelper(idx, 1)"
                  :title="t.launchHelper.moveDown"
                  data-focusable
                >▼</button>
              </div>
              <div v-else class="order-placeholder"></div>

              <LaunchHelperIcon
                class="helper-icon"
                :template-id="helper.templateId"
                :size="20"
                @click="toggleHelper(helper.templateId)"
              />
              <div class="helper-info" @click="toggleHelper(helper.templateId)">
                <span class="helper-name">
                  {{ getTemplate(helper.templateId)?.name }}
                  <span v-if="getTemplate(helper.templateId)?.typeLabel" class="type-badge" :class="{ elevated: getTemplate(helper.templateId)?.elevated }" :title="getTemplate(helper.templateId)?.typeDesc">
                    {{ getTemplate(helper.templateId)?.typeLabel }}
                  </span>
                </span>
                <span class="helper-desc">{{ getTemplate(helper.templateId)?.description }}</span>
              </div>
              <div class="helper-switch" :class="{ on: helper.enabled }" @click="toggleHelper(helper.templateId)">
                <div class="switch-thumb"></div>
              </div>
            </div>

            <Transition name="expand">
              <div v-if="helper.enabled" class="helper-config">
                <div
                  v-for="param in getTemplate(helper.templateId)?.params"
                  :key="param.key"
                  class="config-field"
                >
                  <label class="field-label">
                    {{ param.label }}
                    <span v-if="param.required" class="required">*</span>
                  </label>
                  <div class="field-input-row">
                    <input
                      type="text"
                      class="field-input"
                      :class="{
                        'field-error': hasFieldError(helper.templateId, param.key),
                        'field-warn': pathWarnings[`${helper.templateId}.${param.key}`]
                      }"
                      :placeholder="param.placeholder || ''"
                      :value="getParamValue(helper.templateId, param.key)"
                      @input="setParamValue(helper.templateId, param.key, $event.target.value)"
                      data-focusable
                    />
                    <button
                      v-if="param.key === 'path' && hasTauri"
                      class="browse-btn"
                      @click="browseFile(helper.templateId, param.key)"
                      :title="t.launchHelper.browse"
                      data-focusable
                    ><FolderOpened /></button>
                  </div>
                  <div v-if="hasFieldError(helper.templateId, param.key)" class="field-error-msg">
                    {{ getFieldError(helper.templateId, param.key) }}
                  </div>
                  <div v-else-if="pathWarnings[`${helper.templateId}.${param.key}`]" class="field-path-warn">
                    {{ pathWarnings[`${helper.templateId}.${param.key}`] }}
                  </div>
                  <div v-else-if="getGlobalHint(helper.templateId, param.key)" class="field-hint">
                    {{ t.launchHelper.globalDefault }}{{ getGlobalHint(helper.templateId, param.key) }}
                  </div>
                </div>
              </div>
            </Transition>
          </div>

          <!-- Preview of the generated commands -->
          <div v-if="hasAnyEnabled" class="preview-section">

          <!-- Global path save notice -->
          <Transition name="fade">
            <div v-if="globalSaveNotice" class="global-save-notice">
              {{ globalSaveNotice }}
            </div>
          </Transition>
            <div class="preview-header" @click="showPreview = !showPreview">
              <span>{{ t.launchHelper.preview }}</span>
              <span class="preview-toggle">{{ showPreview ? '▼' : '▶' }}</span>
            </div>
            <div v-if="showPreview" class="preview-content">
              <div v-for="(cmd, i) in previewCommands" :key="i" class="preview-cmd">
                <div class="cmd-label">{{ cmd.label }}</div>
                <code class="cmd-code">{{ cmd.value }}</code>
              </div>
            </div>
          </div>
        </div>

        <div class="panel-footer">
          <button class="btn-secondary" @click="$emit('close')">{{ t.launchHelper.cancel }}</button>
          <button class="btn-primary" @click="handleSave" :disabled="saving">
            {{ saving ? t.launchHelper.saving : t.launchHelper.save }}
          </button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { FolderOpened } from '@element-plus/icons-vue'
import { useLaunchHelpers } from '../composables/useLaunchHelpers.js'
import { isTauriRuntime } from '../composables/useTauri.js'
import { useModalFocusScope } from '../composables/useFocusNav.js'
import { useI18n } from '../i18n/index.js'
import LaunchHelperIcon from './LaunchHelperIcon.vue'

const props = defineProps({
  open: { type: Boolean, default: false },
  appName: { type: String, default: '' },
  app: { type: Object, default: () => ({}) },
  proxyUrl: { type: String, default: '' },
})

const emit = defineEmits(['close', 'saved'])

// 手柄导航只在栈顶作用域内移动，避免焦点跑到面板背后的卡片上
const panelRef = ref(null)
useModalFocusScope(panelRef, () => props.open)

const { t } = useI18n()
const lhText = computed(() => t.value.launchHelper)

const {
  templates,
  getGlobalPath,
  setGlobalPath,
  getAppHelpers,
  setAppHelpers,
  validateHelpers,
  generateAppCommands,
  buildPreviewCommands,
} = useLaunchHelpers(t)

const hasTauri = ref(false)
const saving = ref(false)
const showPreview = ref(false)
const validationErrors = ref([]) // { templateId, paramKey, message }[]
const pathWarnings = ref({}) // { "templateId.paramKey": "message" }
const globalSaveNotice = ref('') // Notice shown when a global path is auto-saved.

// Local working copy of the helper list.
const editingHelpers = ref([])

// Load the configuration when the panel opens or the application changes.
watch(() => [props.open, props.appName], ([isOpen]) => {
  if (isOpen && props.appName) {
    const stored = getAppHelpers(props.appName)
    // Create one entry per template.
    editingHelpers.value = templates.value.map(tmpl => {
      const existing = stored.find(h => h.templateId === tmpl.id)
      return {
        templateId: tmpl.id,
        enabled: existing?.enabled || false,
        params: { ...(existing?.params || {}) },
      }
    })
  }
}, { immediate: true })

function isEnabled(templateId) {
  const h = editingHelpers.value.find(h => h.templateId === templateId)
  return h?.enabled || false
}

function getTemplate(templateId) {
  return templates.value.find(t => t.id === templateId)
}

function toggleHelper(templateId) {
  const h = editingHelpers.value.find(h => h.templateId === templateId)
  if (h) h.enabled = !h.enabled
}

function moveHelper(index, direction) {
  const target = index + direction
  if (target < 0 || target >= editingHelpers.value.length) return
  const arr = [...editingHelpers.value]
  ;[arr[index], arr[target]] = [arr[target], arr[index]]
  editingHelpers.value = arr
}

function getParamValue(templateId, paramKey) {
  const h = editingHelpers.value.find(h => h.templateId === templateId)
  return h?.params?.[paramKey] || getGlobalPath(templateId, paramKey)
}

function setParamValue(templateId, paramKey, value) {
  const h = editingHelpers.value.find(h => h.templateId === templateId)
  if (h) {
    h.params = { ...h.params, [paramKey]: value }
    // Only write back when the global path is empty, so an app-level edit does
    // not leak into other applications.
    if (paramKey === 'path' && value && !getGlobalPath(templateId, paramKey)) {
      setGlobalPath(templateId, paramKey, value)
      const tmpl = getTemplate(templateId)
      globalSaveNotice.value = (lhText.value.globalSaved || '').replace('{name}', tmpl?.name || templateId)
      setTimeout(() => { globalSaveNotice.value = '' }, 3000)
    }
    // Clear the validation error for this field.
    validationErrors.value = validationErrors.value.filter(
      e => !(e.templateId === templateId && e.paramKey === paramKey)
    )
    // Check asynchronously whether the path exists.
    if (paramKey === 'path' && value) {
      checkPathExists(templateId, paramKey, value)
    } else if (paramKey === 'path') {
      delete pathWarnings.value[`${templateId}.${paramKey}`]
    }
  }
}

let _pathCheckTimer = null
async function checkPathExists(templateId, paramKey, filePath) {
  clearTimeout(_pathCheckTimer)
  const key = `${templateId}.${paramKey}`
  _pathCheckTimer = setTimeout(async () => {
    if (!hasTauri.value) return
    try {
      const { exists } = await import('@tauri-apps/plugin-fs')
      const found = await exists(filePath)
      if (!found) {
        pathWarnings.value = { ...pathWarnings.value, [key]: lhText.value.fileNotFound || 'File not found' }
      } else {
        const { [key]: _, ...rest } = pathWarnings.value
        pathWarnings.value = rest
      }
    } catch {
      // The fs plugin is unavailable; ignore.
    }
  }, 500) // Debounced by 500ms.
}

function getGlobalHint(templateId, paramKey) {
  const h = editingHelpers.value.find(h => h.templateId === templateId)
  const localVal = h?.params?.[paramKey]
  const globalVal = getGlobalPath(templateId, paramKey)
  // Only shown when a global default exists and is not overridden locally.
  if (globalVal && !localVal) return globalVal
  return ''
}

function hasFieldError(templateId, paramKey) {
  return validationErrors.value.some(e => e.templateId === templateId && e.paramKey === paramKey)
}

function getFieldError(templateId, paramKey) {
  return validationErrors.value.find(e => e.templateId === templateId && e.paramKey === paramKey)?.message || ''
}

async function browseFile(templateId, paramKey) {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const path = await open({
      filters: [
        { name: lhText.value.executableFiles || 'Executables', extensions: ['exe', 'bat', 'cmd', 'lnk', 'com', 'scr'] },
        { name: lhText.value.allFiles || 'All Files', extensions: ['*'] },
      ],
    })
    if (path) {
      setParamValue(templateId, paramKey, path)
    }
  } catch (e) {
    console.warn('File dialog not available:', e)
  }
}

const hasAnyEnabled = computed(() => editingHelpers.value.some(h => h.enabled))

const previewCommands = computed(() => {
  return buildPreviewCommands(editingHelpers.value)
})

async function handleSave() {
  // Validate the required fields.
  const errors = validateHelpers(editingHelpers.value)
  validationErrors.value = errors
  if (errors.length > 0) {
    return
  }

  saving.value = true
  try {
    // 1. Save locally.
    setAppHelpers(props.appName, editingHelpers.value)

    // 2. Generate the prep-cmds and save them to the server.
    const { prepCmds, wrapCmd } = generateAppCommands(props.appName)

    // Read every application.
    const resp = await fetch(`${props.proxyUrl}/api/apps`)
    if (!resp.ok) throw new Error('Failed to fetch apps')
    const data = await resp.json()
    const allApps = data.apps || data || []

    const appIndex = allApps.findIndex(a => a.name === props.appName)
    if (appIndex === -1) throw new Error('App not found')

    const app = { ...allApps[appIndex] }
    const MARKER = '& REM launch-helper'

    // Remove the previous helper prep-cmds.
    app['prep-cmd'] = (app['prep-cmd'] || []).filter(
      c => !c.do?.includes('REM launch-helper') && !c.undo?.includes('REM launch-helper')
    )

    // Add the new helper prep-cmds.
    const helperPrepCmds = prepCmds.map(c => ({
      do: `${c.do} ${MARKER}`,
      undo: c.undo ? `${c.undo} ${MARKER}` : '',
      elevated: c.elevated ? 'true' : 'false',
    }))
    app['prep-cmd'] = [...helperPrepCmds, ...app['prep-cmd']]

    // Apply the command wrapper.
    if (wrapCmd) {
      const originalCmd = app._originalCmd || app.cmd
      app.cmd = wrapCmd(originalCmd)
    }

    // Strip the internal fields before sending to the server.
    delete app._originalCmd

    // Save in the Sunshine API shape: { apps: [...], editApp: { ...app, index } }.
    const editApp = { ...app, index: appIndex }
    const saveResp = await fetch(`${props.proxyUrl}/api/apps`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ apps: allApps, editApp }),
    })

    if (!saveResp.ok) throw new Error('Failed to save app')

    emit('saved')
    emit('close')
  } catch (e) {
    console.error('Failed to save launch helpers:', e)
    alert(`${lhText.value.saveFailed || 'Save failed: '}${e.message}`)
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  hasTauri.value = await isTauriRuntime()
})

// Global Escape handler, active while the panel is open.
function onKeydown(e) {
  if (e.key === 'Escape' && props.open) {
    emit('close')
  }
}
watch(() => props.open, (open) => {
  if (open) {
    document.addEventListener('keydown', onKeydown)
  } else {
    document.removeEventListener('keydown', onKeydown)
  }
})
onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})
</script>

<style lang="less" scoped>
.helper-overlay {
  position: fixed;
  inset: 0;
  background: var(--scrim);
  z-index: 9998;
}

.helper-panel {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 460px;
  max-width: 92vw;
  background: var(--bg-panel);
  border-left: 1px solid var(--border);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  font-family: var(--font-sans);
  font-size: var(--font-size);
  color: var(--text);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  height: var(--row-height);
  padding: 0 4px 0 10px;
  border-bottom: 1px solid var(--border);

  h2 {
    margin: 0;
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text);
  }

  .app-name-tag {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .close-btn {
    width: 20px;
    height: 20px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    font-size: var(--font-size);
    line-height: 1;
    cursor: pointer;

    &:hover {
      background: var(--bg-hover);
      color: var(--text);
    }
  }
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 10px;
}

.panel-desc {
  margin: 0 0 8px;
  font-size: var(--font-size-sm);
  color: var(--text-dim);

  small {
    color: var(--text-faint);
    font-size: var(--font-size-xs);
  }
}

.helper-card {
  border: 1px solid var(--border);
  background: transparent;
  margin-bottom: 6px;

  &.active {
    border-color: var(--border-strong);
  }
}

.helper-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  min-height: var(--row-height);
  cursor: pointer;

  &:hover {
    background: var(--bg-hover);
  }

  .order-buttons,
  .order-placeholder {
    width: 14px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .order-btn {
    width: 14px;
    height: 10px;
    padding: 0;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    font-size: 7px;
    line-height: 1;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: var(--bg-hover);
      color: var(--text);
    }

    &:disabled {
      color: var(--text-faint);
      cursor: default;
    }
  }

  .helper-icon {
    color: var(--text-dim);
    flex-shrink: 0;
  }

  .helper-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .helper-name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--font-size);
    color: var(--text);
  }

  .type-badge {
    font-size: var(--font-size-xs);
    line-height: 14px;
    padding: 0 4px;
    border: 1px solid var(--border);
    color: var(--text-dim);

    &.elevated {
      border-color: var(--warn);
      color: var(--warn);
    }
  }

  .helper-desc {
    font-size: var(--font-size-sm);
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

// The same square switch the settings screen uses.
.helper-switch {
  position: relative;
  width: 28px;
  height: 14px;
  flex-shrink: 0;
  background: var(--bg-input);
  border: 1px solid var(--border);
  cursor: pointer;

  .switch-thumb {
    position: absolute;
    top: 1px;
    left: 1px;
    width: 10px;
    height: 10px;
    background: var(--off);
  }

  &.on {
    border-color: var(--border-strong);

    .switch-thumb {
      left: 15px;
      background: var(--on);
    }
  }
}

.helper-config {
  padding: 6px 8px 8px;
  border-top: 1px solid var(--border);
}

.config-field {
  margin-bottom: 6px;
}

.field-label {
  display: block;
  font-size: var(--font-size-sm);
  color: var(--text-dim);
  margin-bottom: 2px;

  .required {
    color: var(--error);
  }
}

.field-input-row {
  display: flex;
  gap: 4px;
}

.field-input {
  flex: 1;
  height: 24px;
  padding: 0 8px;
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  outline: none;

  &::placeholder { color: var(--text-faint); }

  &:focus { border-color: var(--border-strong); }
}

.browse-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;

  :deep(svg) {
    width: 12px;
    height: 12px;
  }

  &:hover {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--border-strong);
  }
}

.field-hint {
  font-size: var(--font-size-xs);
  color: var(--text-faint);
  margin-top: 2px;
  word-break: break-all;
}

.field-error-msg {
  font-size: var(--font-size-xs);
  color: var(--error);
  margin-top: 2px;
}

.field-input.field-error {
  border-color: var(--error);
}

.field-path-warn {
  font-size: var(--font-size-xs);
  color: var(--warn);
  margin-top: 2px;
}

.field-input.field-warn {
  border-color: var(--warn);
}

.preview-section {
  margin-top: 8px;
  border: 1px solid var(--border);
}

.global-save-notice {
  padding: 4px 8px;
  border-bottom: 1px solid var(--border);
  font-size: var(--font-size-xs);
  color: var(--text-dim);
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 24px;
  padding: 0 8px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-dim);

  &:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .preview-toggle {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }
}

.preview-content {
  padding: 6px 8px;
}

.preview-cmd {
  margin-bottom: 6px;

  &:last-child {
    margin-bottom: 0;
  }

  .cmd-label {
    font-size: var(--font-size-xs);
    color: var(--text-dim);
    margin-bottom: 2px;
  }

  .cmd-code {
    display: block;
    padding: 4px 6px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    white-space: pre-wrap;
    word-break: break-all;
  }
}

.panel-footer {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
  padding: 6px 10px;
  border-top: 1px solid var(--border);
}

.btn-secondary,
.btn-primary {
  height: 24px;
  padding: 0 12px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
  font-size: var(--font-size-sm);
  cursor: pointer;

  &:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  &:disabled {
    color: var(--text-faint);
    cursor: not-allowed;
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }
}

.btn-primary {
  border-color: var(--border-strong);
  font-weight: 600;
}

.slide-enter-active,
.slide-leave-active,
.fade-enter-active,
.fade-leave-active,
.expand-enter-active,
.expand-leave-active {
  transition: none;
}
</style>
