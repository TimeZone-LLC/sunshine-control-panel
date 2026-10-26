<template>
  <div class="chub-type-picker">
    <p class="chub-question">{{ t.controllersHub.emulation.typeQuestion }}</p>

    <el-alert
      v-if="unreachable"
      class="chub-notice"
      type="error"
      :title="t.controllersHub.emulation.sunshineUnreachable"
      show-icon
      :closable="false"
    >
      <el-button size="small" type="primary" :loading="loading" @click="loadConfig">
        {{ t.controllersHub.emulation.retry }}
      </el-button>
    </el-alert>

    <div v-if="!unreachable" v-loading="loading" class="chub-backend-settings">
      <label class="chub-field-label" for="controller-backend">
        {{ t.controllersHub.emulation.backendTitle }}
      </label>
      <el-select
        id="controller-backend"
        :model-value="config.gamepad_driver"
        :disabled="busyKey !== ''"
        @change="saveDriver"
      >
        <el-option
          v-for="option in driverOptions"
          :key="option.value"
          :value="option.value"
          :label="option.label"
        />
      </el-select>
      <p class="chub-hint">{{ t.controllersHub.emulation.backendHint }}</p>
      <div v-if="config.gamepad_driver !== 'vigembus'" class="chub-identity-setting">
        <el-checkbox
          :model-value="config.ds5_inputtino_randomize_mac"
          :disabled="busyKey !== ''"
          @change="(value) => saveKey(
            'ds5_inputtino_randomize_mac',
            value,
            t.controllersHub.emulation.identitySaveSuccess,
          )"
        >
          {{ t.controllersHub.emulation.randomizeIdentity }}
        </el-checkbox>
        <p class="chub-hint">{{ t.controllersHub.emulation.randomizeIdentityHint }}</p>
      </div>
    </div>

    <div v-if="!unreachable" v-loading="loading" class="chub-picker-row">
      <div class="chub-mode-list" role="radiogroup" :aria-label="t.controllersHub.emulation.typeQuestion">
        <button
          v-for="option in modeOptions"
          :key="option.value"
          type="button"
          class="chub-mode-option"
          :class="{ 'is-selected': selectedMode === option.value }"
          role="radio"
          :aria-checked="selectedMode === option.value"
          :disabled="busyKey !== ''"
          @click="selectedMode = option.value"
        >
          <span class="chub-mode-cursor" aria-hidden="true"></span>
          <span class="chub-mode-body">
            <strong>{{ option.label }}</strong>
            <small>{{ option.hint }}</small>
          </span>
        </button>
      </div>
    </div>

    <el-collapse v-if="!unreachable" class="chub-advanced">
      <el-collapse-item
        name="perGame"
        :title="t.controllersHub.emulation.perAppTitle"
      >
        <p class="chub-hint">{{ t.controllersHub.emulation.perAppHint }}</p>
        <el-alert
          v-if="appsError"
          class="chub-notice"
          type="error"
          :title="t.controllersHub.emulation.appLoadFailed"
          show-icon
          :closable="false"
        >
          <el-button size="small" :loading="appsLoading" @click="loadApps">
            {{ t.controllersHub.emulation.retry }}
          </el-button>
        </el-alert>
        <div v-else v-loading="appsLoading" class="chub-app-list">
          <div v-for="(app, index) in apps" :key="app.name + index" class="chub-app-row">
            <span class="chub-app-name">{{ app.name }}</span>
            <el-select
              :model-value="app.gamepad || ''"
              size="small"
              class="chub-app-select"
              :disabled="appSavingIndex === index"
              @change="(value) => saveAppGamepad(index, value)"
            >
              <el-option
                v-for="opt in appModeOptions"
                :key="opt.value"
                :value="opt.value"
                :label="opt.label"
              />
            </el-select>
          </div>
          <p v-if="!appsLoading && apps.length === 0" class="chub-hint">
            {{ t.controllersHub.emulation.noApps }}
          </p>
        </div>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { ElMessage } from '../ui/compact/feedback.js'
import { controllerHub, sunshine } from '../../tauri-adapter.js'
import {
  availableGamepadProfiles,
  reconcileGamepadProfile,
} from '../../composables/controllerProfiles.js'
import { useI18n } from '../../i18n/index.js'

const { t } = useI18n()
const emit = defineEmits(['ds5-selection-change'])

const APPS_TIMEOUT_MS = 8000

const config = ref({
  gamepad: 'auto',
  gamepad_driver: 'vigembus',
  ds5_inputtino_randomize_mac: true,
})
const loading = ref(false)
const unreachable = ref(false)

const apps = ref([])
const appsLoading = ref(false)
const appsError = ref(false)
const appSavingIndex = ref(-1)
let proxyUrl = ''
let appsRequestId = 0

const busyKey = ref('')

const driverOptions = computed(() => {
  const e = t.value.controllersHub.emulation
  return [
    { value: 'vigembus', label: e.backendVigem },
    { value: 'virtualhid', label: e.backendVirtualHid },
    { value: 'all', label: e.backendAll },
  ]
})

const profileCopy = computed(() => {
  const e = t.value.controllersHub.emulation
  return {
    auto: { label: e.familyAuto, hint: e.familyAutoHint },
    generic: { label: e.genericController, hint: e.genericControllerHint },
    x360: { label: e.xbox360, hint: e.xbox360Hint },
    xone: { label: e.xboxOne, hint: e.xboxOneHint },
    xseries: { label: e.xboxSeries, hint: e.xboxSeriesHint },
    ds4: { label: 'DualShock 4', hint: e.ds4ModeHint },
    ds5: { label: e.ds5Variant, hint: e.ds5ModeHint },
    switch: { label: e.switchPro, hint: e.switchProHint },
  }
})

const modeOptions = computed(() => {
  return availableGamepadProfiles(config.value.gamepad_driver).map((value) => ({
    value,
    ...profileCopy.value[value],
  }))
})

const selectedMode = computed({
  get: () => config.value.gamepad,
  set: (mode) => saveKey('gamepad', mode),
})

const appModeOptions = computed(() => {
  const e = t.value.controllersHub.emulation
  return [
    { value: '', label: e.followGlobal },
    ...availableGamepadProfiles(config.value.gamepad_driver).map((value) => ({
      value,
      label: profileCopy.value[value].label,
    })),
  ]
})

const hasDs5Selection = computed(() =>
  config.value.gamepad === 'ds5' || apps.value.some((app) => app.gamepad === 'ds5'))

watch(hasDs5Selection, (selected) => emit('ds5-selection-change', selected), { immediate: true })

async function loadConfig() {
  loading.value = true
  unreachable.value = false
  try {
    const result = await controllerHub.getConfig()
    if (result?.success) {
      Object.assign(config.value, result.data)
    } else {
      unreachable.value = true
    }
  } catch {
    unreachable.value = true
  } finally {
    loading.value = false
  }
}

async function savePatch(patch, successMessage) {
  const keys = Object.keys(patch)
  if (busyKey.value !== '' || keys.every((key) => config.value[key] === patch[key])) return
  const previous = Object.fromEntries(keys.map((key) => [key, config.value[key]]))
  Object.assign(config.value, patch)
  busyKey.value = keys.join(',')
  try {
    const result = await controllerHub.saveConfig(patch)
    if (result?.success) {
      ElMessage.success(successMessage)
    } else {
      Object.assign(config.value, previous)
      ElMessage.error(result?.message || t.value.controllersHub.emulation.saveFailed)
    }
  } catch (error) {
    Object.assign(config.value, previous)
    ElMessage.error(String(error))
  } finally {
    busyKey.value = ''
  }
}

function saveKey(key, value, successMessage = t.value.controllersHub.emulation.saveSuccess) {
  return savePatch({ [key]: value }, successMessage)
}

function saveDriver(driver) {
  const nextMode = reconcileGamepadProfile(config.value.gamepad, driver)
  const patch = { gamepad_driver: driver }
  if (nextMode !== config.value.gamepad) patch.gamepad = nextMode
  return savePatch(patch, t.value.controllersHub.emulation.backendSaveSuccess)
}

async function loadApps() {
  const requestId = ++appsRequestId
  appsLoading.value = true
  appsError.value = false
  let timeoutTimer = null
  try {
    const proxyResult = await sunshine.waitForProxyReady()
    if (!proxyResult?.success) throw new Error(proxyResult?.message || 'proxy not ready')
    if (requestId !== appsRequestId) return
    proxyUrl = proxyResult.data

    const controller = new AbortController()
    timeoutTimer = setTimeout(() => controller.abort('timeout'), APPS_TIMEOUT_MS)
    const resp = await fetch(`${proxyUrl}/api/apps`, { signal: controller.signal })
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)

    const data = await resp.json()
    if (requestId !== appsRequestId) return
    apps.value = Array.isArray(data) ? data : data?.apps || []
  } catch (error) {
    if (requestId !== appsRequestId) return
    console.error('Unable to load the application list:', error)
    appsError.value = true
  } finally {
    if (timeoutTimer !== null) clearTimeout(timeoutTimer)
    if (requestId === appsRequestId) appsLoading.value = false
  }
}

async function saveAppGamepad(index, value) {
  const app = apps.value[index]
  if (!app || appSavingIndex.value !== -1) return
  const prev = app.gamepad || ''
  app.gamepad = value
  appSavingIndex.value = index
  let timeoutTimer = null
  try {
    const controller = new AbortController()
    timeoutTimer = setTimeout(() => controller.abort('timeout'), APPS_TIMEOUT_MS)
    const resp = await fetch(`${proxyUrl}/api/apps`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ apps: apps.value, editApp: { index, ...app } }),
      signal: controller.signal,
    })
    if (resp.status === 409) {
      app.gamepad = prev
      ElMessage.error(t.value.controllersHub.emulation.appSaveConflict)
      await loadApps()
      return
    }
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    ElMessage.success(t.value.controllersHub.emulation.appSaved)
  } catch {
    app.gamepad = prev
    ElMessage.error(t.value.controllersHub.emulation.appSaveFailed)
  } finally {
    if (timeoutTimer !== null) clearTimeout(timeoutTimer)
    appSavingIndex.value = -1
  }
}

defineExpose({ refresh: loadConfig })

onMounted(() => {
  loadConfig()
  loadApps()
})
</script>
