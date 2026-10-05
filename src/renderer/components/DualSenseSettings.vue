<template>
  <section class="ds5-page">
    <header class="page-header">
      <div class="page-title-group">
        <p class="eyebrow">{{ t.controllers.eyebrow }}</p>
        <h1>{{ t.controllers.title }}</h1>
        <p>{{ t.controllers.intro }}</p>
      </div>
      <span class="experimental-tag">{{ t.dualSense.experimental }}</span>
    </header>

    <article class="component-panel" :class="`state-${status.state}`">
      <section class="title-row">
        <div>
          <h2>{{ t.dualSense.title }}</h2>
          <p>{{ t.dualSense.intro }}</p>
        </div>
        <el-checkbox
          v-model="enabled"
          class="enable-control"
          :disabled="!componentOperational || status.in_use || componentControlsBusy"
          @change="saveSettings"
        >{{ t.dualSense.enableShort }}</el-checkbox>
      </section>

      <section class="status-row" aria-live="polite">
        <div class="status-heading">
          <span class="status-dot" aria-hidden="true"></span>
          <strong>{{ stateLabel }}</strong>
          <span v-if="overallVersion" class="status-version">{{ overallVersion }}</span>
        </div>
        <div class="status-actions">
          <el-button
            v-if="statusKnown && componentAction === 'install'"
            class="menu-action"
            :loading="operation === 'install'"
            :disabled="status.in_use || controlsBusy"
            @click="install"
          >{{ t.dualSense.install }}</el-button>
          <el-button
            v-else-if="statusKnown && componentAction === 'repair'"
            class="menu-action menu-action-warning"
            :loading="operation === 'install'"
            :disabled="status.in_use || controlsBusy"
            @click="install"
          >{{ t.dualSense.repair }}</el-button>
          <el-button
            v-else-if="statusKnown && componentAction === 'update'"
            class="menu-action menu-action-warning"
            :loading="operation === 'install'"
            :disabled="status.in_use || controlsBusy"
            @click="install"
          >{{ t.dualSense.update }}</el-button>
          <el-button
            class="menu-action"
            :loading="refreshing"
            :disabled="refreshing || controlsBusy"
            @click="refresh()"
          >{{ t.dualSense.refresh }}</el-button>
          <el-button
            v-if="statusKnown && (!status.installed || !status.verified || status.update_available)"
            class="menu-action"
            :disabled="status.in_use || controlsBusy"
            @click="installFromPackage"
          >{{ t.dualSense.installLocalPackage }}</el-button>
        </div>
      </section>

      <article v-if="operation === 'install'" class="operation-card" aria-live="polite">
        <div><strong>{{ operationStage }}</strong><span>{{ operationProgress }}%</span></div>
        <el-progress :percentage="operationProgress" :show-text="false" />
      </article>

      <el-alert
        v-if="showNotice"
        class="notice"
        :type="status.state === 'in_use' ? 'error' : 'warning'"
        :title="nextAction"
        :closable="false"
        show-icon
      />

      <el-alert
        v-if="operationError"
        class="notice"
        type="error"
        :title="t.dualSense.technicalDetails"
        :description="operationError"
        show-icon
        @close="operationError = ''"
      />

      <section v-if="componentOperational" class="profile-section" :aria-label="t.dualSense.profileTitle">
        <span class="section-label">{{ t.dualSense.profileTitle }}</span>
        <div class="profile-options" role="group">
          <div class="profile-option is-selected is-static">
            <span class="selection-cursor" aria-hidden="true"></span>
            <span class="option-copy">
              <strong>{{ t.dualSense.standardModeShort }}</strong>
              <small>{{ t.dualSense.standardModeTip }}</small>
            </span>
            <kbd>{{ t.dualSense.included }}</kbd>
          </div>
          <button
            type="button"
            class="profile-option"
            :class="{ 'is-selected': audioHaptics }"
            role="checkbox"
            :aria-checked="audioHaptics"
            :disabled="status.in_use || componentControlsBusy || (!status.usbip_available && !audioHaptics)"
            @click="setAudioHaptics(!audioHaptics)"
          >
            <span class="selection-cursor" aria-hidden="true"></span>
            <span class="option-copy">
              <strong>{{ t.dualSense.nativeModeShort }}</strong>
              <small>{{ status.usbip_available ? t.dualSense.nativeModeTip : t.dualSense.nativeUnavailable }}</small>
            </span>
            <kbd>{{ audioHaptics ? t.dualSense.enabledLabel : t.dualSense.disabledLabel }}</kbd>
          </button>
        </div>
      </section>

      <section v-if="componentOperational" class="tuning-section" :aria-label="t.dualSense.tuningTitle">
        <div class="tuning-heading">
          <div>
            <span class="section-label">{{ t.dualSense.tuningTitle }}</span>
            <p class="tuning-hint">{{ t.dualSense.tuningHint }}</p>
          </div>
        </div>
        <div class="tuning-presets" role="group" :aria-label="t.dualSense.tuningPresetTitle">
          <span>{{ t.dualSense.tuningPresetTitle }}</span>
          <div>
            <button type="button" class="tuning-preset" :disabled="componentControlsBusy" @click="applyDefaultPreset">
              <strong>{{ t.dualSense.tuningPresetDefault }}</strong>
              <small>{{ t.dualSense.tuningPresetDefaultTip }}</small>
            </button>
            <button type="button" class="tuning-preset" :disabled="componentControlsBusy" @click="applyErmPreset">
              <strong>{{ t.dualSense.tuningPresetErm }}</strong>
              <small>{{ t.dualSense.tuningPresetErmTip }}</small>
            </button>
          </div>
        </div>
        <div class="tuning-grid">
          <label class="tuning-field">
            <span>{{ t.dualSense.tuningStrength }}</span>
            <el-input-number v-model="legacyStrength" :min="0.1" :max="4" :step="0.05" size="small" :disabled="componentControlsBusy" />
          </label>
          <label class="tuning-field">
            <span>{{ t.dualSense.tuningCurve }}</span>
            <el-input-number v-model="legacyCurve" :min="0.3" :max="2" :step="0.05" size="small" :disabled="componentControlsBusy" />
          </label>
          <label class="tuning-field">
            <span>{{ t.dualSense.tuningGate }}</span>
            <el-input-number
              v-model="legacyNoiseGate" :min="0.002" :max="0.06"
              :step="0.002" :precision="3" size="small" :disabled="componentControlsBusy"
            />
          </label>
        </div>
        <div class="tuning-save-row">
          <el-button
            type="primary" class="tuning-save-button"
            :loading="tuningSaving" :disabled="controlsBusy || !tuningDirty"
            @click="saveTuning"
          >
            <span>{{ t.dualSense.tuningSave }}</span>
          </el-button>
          <el-tag v-if="tuningDirty" class="tuning-unsaved-tag" effect="plain">
            {{ t.dualSense.tuningUnsaved }}
          </el-tag>
        </div>
      </section>

      <section v-if="componentOperational" class="compatibility-section" :aria-label="t.dualSense.gameCompatibility">
        <span class="section-label">{{ t.dualSense.gameCompatibility }}</span>
        <div class="compatibility-option">
          <span class="option-copy">
            <strong>{{ t.dualSense.genshinMode }}</strong>
            <small>{{ status.genshin_compatibility_available ? t.dualSense.genshinModeTip : t.dualSense.genshinModeUnavailable }}</small>
          </span>
          <el-checkbox
            v-model="genshinCompatibility"
            :disabled="!status.genshin_compatibility_available || !status.usbip_available || !enabled || !audioHaptics || status.in_use || componentControlsBusy"
            @change="setGenshinCompatibility"
          >{{ genshinCompatibility ? t.dualSense.enabledLabel : t.dualSense.disabledLabel }}</el-checkbox>
        </div>
        <p v-if="genshinCompatibility" class="compatibility-notice">
          <span aria-hidden="true">!</span>{{ t.dualSense.genshinModeActive }}
        </p>
      </section>

      <section v-if="componentOperational" class="test-section">
        <div>
          <strong>{{ t.dualSense.validateMode }}</strong>
          <p>{{ testCompleted ? t.dualSense.validationTip : t.dualSense.validateModeTip }}</p>
        </div>
        <div class="test-actions">
          <el-button
            :loading="operation === 'standard'"
            :disabled="status.in_use || controlsBusy || !status.standard_profile"
            @click="test('standard')"
          >{{ t.dualSense.testStandard }}</el-button>
          <el-button
            :loading="operation === 'composite'"
            :disabled="status.in_use || controlsBusy || !canTestAudioHaptics"
            @click="test('composite')"
          >{{ t.dualSense.testComposite }}</el-button>
          <div v-if="testCompleted" class="controller-meta-action">
            <el-button @click="emit('open-controller-meta')">{{ t.dualSense.openControllerMeta }}</el-button>
          </div>
        </div>
      </section>

      <el-collapse v-model="expandedSections" class="details-collapse">
        <el-collapse-item name="health" :title="t.dualSense.componentHealth">
          <div class="health-list">
            <div v-for="item in healthRows" :key="item.label" class="health-row">
              <span>{{ item.label }}</span>
              <strong class="health-state"><i :class="item.tone" aria-hidden="true"></i>{{ item.state }}</strong>
              <span class="health-detail">{{ item.detail }}</span>
            </div>
          </div>
          <footer class="panel-footer">
            <details v-if="safeStatusDetail">
              <summary>{{ status.error_code || t.dualSense.technicalDetails }}</summary>
              <pre>{{ safeStatusDetail }}</pre>
            </details>
            <el-button
              v-if="status.installed"
              type="danger"
              :loading="operation === 'uninstall'"
              :disabled="status.in_use || controlsBusy"
              @click="uninstall"
            >{{ t.dualSense.uninstall }}</el-button>
          </footer>
        </el-collapse-item>
      </el-collapse>
    </article>
  </section>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { open } from '@tauri-apps/plugin-dialog'
import { dualsense } from '../tauri-adapter.js'
import {
  dualSenseErrorCode,
  friendlyDualSenseError,
  safeDualSenseTechnicalError,
} from '../composables/dualsenseErrors.js'
import {
  createLatestIntentQueue,
  dualSenseConfigAfterInstall,
  dualSenseConfigAfterUninstall,
  dualSenseConfigMatches,
  dualSenseConfigReadable,
  dualSenseConfigUiState,
  mergeDualSenseStatus,
} from '../composables/dualsenseConfigSync.js'
import { installSelectedDualSensePackages } from '../composables/dualsenseInstallFlow.js'
import { useI18n } from '../desktop/i18n/index.js'

const { t } = useI18n()
const emit = defineEmits(['open-controller-meta'])
const statusKnown = ref(false)
const saving = ref(false)
const refreshing = ref(false)
const operation = ref('')
const operationProgress = ref(0)
const operationStageKey = ref('preparing')
const operationError = ref('')
const enabled = ref(false)
const audioHaptics = ref(false)
const genshinCompatibility = ref(false)
const legacyStrength = ref(1)
const legacyCurve = ref(0.5)
const legacyNoiseGate = ref(0.02)
const tuningSaving = ref(false)
const testCompleted = ref(false)
const expandedSections = ref([])
const rebootRecommended = ref(false)
const status = ref({
  state: 'loading', installed: false, verified: false, enabled: false,
  audio_haptics: false, genshin_compatibility: false,
  genshin_compatibility_available: false,
  component_version: '', available_component_version: '', update_available: false,
  driver_installed: false, usbip_available: false, usbip_version: '',
  usbip_version_valid: false, reboot_recommended: false,
  standard_profile: false, composite_profile: false, in_use: false,
  legacy_strength: 1, legacy_curve: 0.5, legacy_noise_gate: 0.02,
  config_revision: 0, config_readable: false,
  error_code: '', detail: '',
})
let pollTimer
let unlistenProgress
let statusRefreshGeneration = 0
let statusRefreshPromise = null
let tuningSavePromise = null
const STATUS_REFRESH_WAIT_TIMEOUT_MS = 5000
const CONFIG_SAVE_DEBOUNCE_MS = 300

const controlsBusy = computed(() => saving.value || tuningSaving.value || Boolean(operation.value))
const componentControlsBusy = computed(() => Boolean(operation.value))
const componentAction = computed(() => {
  if (!status.value.installed) return 'install'
  if (!status.value.verified) return 'repair'
  if (status.value.update_available) return 'update'
  return ''
})
const componentOperational = computed(() => Boolean(status.value.verified))
const tuningDirty = computed(() =>
  legacyStrength.value !== status.value.legacy_strength
  || legacyCurve.value !== status.value.legacy_curve
  || legacyNoiseGate.value !== status.value.legacy_noise_gate)
const stateLabel = computed(() => t.value.dualSense.states[status.value.state] || status.value.state)
const safeStatusDetail = computed(() => status.value.detail
  ? safeDualSenseTechnicalError(status.value.detail)
  : '')
const operationStage = computed(() => t.value.dualSense.stages[operationStageKey.value] || operationStageKey.value)
const nextAction = computed(() => {
  if (rebootRecommended.value) {
    return status.value.usbip_available
      ? t.value.dualSense.restartSuggestedAvailable
      : t.value.dualSense.restartSuggestedUnavailable
  }
  return {
    not_installed: t.value.dualSense.nextNotInstalled,
    repair_required: t.value.dualSense.nextRepair,
    update_available: t.value.dualSense.nextUpdate,
    transport_missing: t.value.dualSense.nextTransport,
    in_use: t.value.dualSense.nextInUse,
  }[status.value.state] || t.value.dualSense.nextRepair
})

const overallVersion = computed(() => {
  if (status.value.verified && status.value.update_available) {
    const installedVersion = status.value.component_version || t.value.dualSense.unknownVersion
    return `${installedVersion} -> ${status.value.available_component_version}`
  }
  return status.value.component_version || status.value.runtime_version || status.value.error_code
})
const canTestAudioHaptics = computed(() => status.value.composite_profile && status.value.usbip_available)
const usbTransportDetail = computed(() => {
  if (!status.value.usbip_version_valid) return t.value.dualSense.pinnedTransportRequired
  if (status.value.usbip_available) return `USB/IP ${status.value.usbip_version}`
  if (!status.value.installed) return t.value.dualSense.transportCheckPending
  return t.value.dualSense.transportProbeFailed
})
const showNotice = computed(() => rebootRecommended.value || ['not_installed', 'repair_required', 'update_available', 'transport_missing', 'in_use'].includes(status.value.state))
const healthRows = computed(() => [
  {
    label: t.value.dualSense.component,
    state: status.value.verified && status.value.update_available
      ? t.value.dualSense.updateAvailable
      : status.value.verified ? t.value.dualSense.available : t.value.dualSense.unavailable,
    detail: status.value.verified && status.value.update_available
      ? `${status.value.component_version || t.value.dualSense.unknownVersion} -> ${status.value.available_component_version}`
      : status.value.component_version || status.value.error_code,
    tone: status.value.verified && status.value.update_available ? 'warn' : status.value.verified ? 'ok' : status.value.installed ? 'bad' : '',
  },
  {
    label: t.value.dualSense.runtime,
    state: status.value.runtime_version ? t.value.dualSense.available : t.value.dualSense.unavailable,
    detail: status.value.runtime_version,
    tone: status.value.runtime_version ? 'ok' : '',
  },
  {
    label: t.value.dualSense.standard,
    state: status.value.standard_profile ? t.value.dualSense.available : t.value.dualSense.unavailable,
    detail: status.value.in_use ? t.value.dualSense.deviceActive : t.value.dualSense.createdOnDemand,
    tone: status.value.in_use ? 'busy' : status.value.standard_profile ? 'ok' : '',
  },
  {
    label: t.value.dualSense.usbTransport,
    state: status.value.usbip_available ? t.value.dualSense.available : t.value.dualSense.unavailable,
    detail: usbTransportDetail.value,
    tone: status.value.usbip_available ? 'ok' : 'warn',
  },
  {
    label: t.value.dualSense.composite,
    state: status.value.composite_profile && status.value.usbip_available ? t.value.dualSense.available : t.value.dualSense.unavailable,
    detail: status.value.composite_profile ? '4 ch · 48 kHz' : '',
    tone: status.value.composite_profile && status.value.usbip_available ? 'ok' : 'warn',
  },
])

const showError = (message, context = 'generic') => {
  operationError.value = safeDualSenseTechnicalError(message)
  ElMessage.error(friendlyDualSenseError(message, t.value.dualSense.errors, context))
}

const invalidateStatusRefresh = () => {
  statusRefreshGeneration += 1
}

const waitForStatusRefresh = async () => {
  const pending = statusRefreshPromise
  if (!pending) return true

  let timeoutId
  try {
    return await Promise.race([
      pending.then(() => true),
      new Promise((resolve) => {
        timeoutId = window.setTimeout(() => resolve(false), STATUS_REFRESH_WAIT_TIMEOUT_MS)
      }),
    ])
  } finally {
    if (timeoutId !== undefined) window.clearTimeout(timeoutId)
  }
}

const waitForTuningSave = async () => {
  const pending = tuningSavePromise
  if (pending) await pending
}

const refresh = async (quiet = false, allowBusy = false) => {
  if ((!allowBusy && controlsBusy.value) || refreshing.value || (quiet && statusRefreshPromise)) return false
  if (!quiet && statusRefreshPromise) {
    await waitForStatusRefresh()
    if (controlsBusy.value || refreshing.value || statusRefreshPromise) return false
  }
  const refreshGeneration = ++statusRefreshGeneration
  if (!quiet) operationError.value = ''
  const preserveTuning = quiet && tuningDirty.value
  if (!quiet) refreshing.value = true
  let refreshed = false
  const request = dualsense.getStatus()
  statusRefreshPromise = request
  try {
    const result = await request
    if (refreshGeneration !== statusRefreshGeneration) return false
    if (result.success) {
      const configReadable = dualSenseConfigReadable(result.data)
      status.value = mergeDualSenseStatus(status.value, result.data)
      statusKnown.value = true
      if (configReadable) {
        enabled.value = result.data.enabled
        audioHaptics.value = result.data.audio_haptics
        genshinCompatibility.value = result.data.genshin_compatibility ?? false
        if (!preserveTuning) {
          legacyStrength.value = result.data.legacy_strength
          legacyCurve.value = result.data.legacy_curve
          legacyNoiseGate.value = result.data.legacy_noise_gate
        }
        refreshed = true
      } else if (!quiet) {
        showError(result.data.detail || result.data.error_code, 'status')
      }
    } else if (!quiet) {
      showError(result.message, 'status')
    }
  } finally {
    if (statusRefreshPromise === request) statusRefreshPromise = null
    if (!quiet) refreshing.value = false
  }
  return refreshed
}

const install = async (packagePaths = []) => {
  if (controlsBusy.value) return
  packagePaths = Array.isArray(packagePaths)
    ? packagePaths.filter((path) => typeof path === 'string' && path)
    : []
  const componentWasInstalled = status.value.installed
  const upgrading = componentAction.value === 'update'
  const localPackage = packagePaths.length > 0
  operation.value = 'confirm-install'
  try {
    await ElMessageBox.confirm(
      localPackage
        ? t.value.dualSense.localInstallConfirm
        : upgrading ? t.value.dualSense.updateConfirm : t.value.dualSense.installConfirm,
      localPackage
        ? t.value.dualSense.localInstallTitle
        : upgrading ? t.value.dualSense.updateTitle : t.value.dualSense.installTitle,
      {
        type: 'warning',
        confirmButtonText: upgrading ? t.value.dualSense.update : t.value.dualSense.install,
      },
    )
  } catch {
    operation.value = ''
    return
  }
  invalidateStatusRefresh()
  operation.value = 'install'
  operationError.value = ''
  operationProgress.value = 0
  await waitForStatusRefresh()
  let result
  try {
    result = await dualsense.install(packagePaths)
  } catch (error) {
    operation.value = ''
    await refresh(true)
    return showError(error, 'install')
  }
  if (!result.success) {
    operation.value = ''
    await refresh(true)
    return showError(result.message, 'install')
  }
  operation.value = ''
  rebootRecommended.value = result.data.reboot_recommended
  const configReadable = dualSenseConfigReadable(result.data)
  status.value = mergeDualSenseStatus(status.value, result.data)
  statusKnown.value = true
  if (configReadable) {
    enabled.value = result.data.enabled
    audioHaptics.value = result.data.audio_haptics
    genshinCompatibility.value = result.data.genshin_compatibility ?? false
    legacyStrength.value = result.data.legacy_strength
    legacyCurve.value = result.data.legacy_curve
    legacyNoiseGate.value = result.data.legacy_noise_gate
  }
  const installedConfig = dualSenseConfigAfterInstall(result.data, componentWasInstalled)
  if (installedConfig && !(await applyComponentConfig(installedConfig))) {
    if (rebootRecommended.value) {
      ElMessage.warning(result.data.usbip_available
        ? t.value.dualSense.restartSuggestedAvailable
        : t.value.dualSense.restartSuggestedUnavailable)
    }
    return
  }
  operationError.value = ''
  ElMessage.success(upgrading ? t.value.dualSense.updateSuccess : t.value.dualSense.installSuccess)
  if (result.data.reboot_recommended) {
    ElMessage.warning(status.value.usbip_available
      ? t.value.dualSense.restartSuggestedAvailable
      : t.value.dualSense.restartSuggestedUnavailable)
  }
}

const installFromPackage = async () => {
  if (controlsBusy.value) return
  let selected
  try {
    selected = await open({
      multiple: true,
      directory: false,
      title: t.value.dualSense.selectLocalPackage,
      filters: [{ name: t.value.dualSense.componentPackage, extensions: ['zip', 'exe'] }],
    })
  } catch (error) {
    showError(error, 'packagePicker')
    return
  }
  await installSelectedDualSensePackages({ packagePaths: selected, installPackages: install })
}

const applyConfigControls = (requested) => {
  enabled.value = requested.enabled
  audioHaptics.value = requested.audioHaptics
  genshinCompatibility.value = requested.genshinCompatibility
}

const restoreConfirmedConfigControls = () => {
  enabled.value = status.value.enabled
  audioHaptics.value = status.value.audio_haptics
  genshinCompatibility.value = status.value.genshin_compatibility ?? false
}

const synchronizeConfirmedConfig = (preserveTuning) => {
  const uiState = dualSenseConfigUiState(status.value, preserveTuning || tuningDirty.value)
  enabled.value = uiState.enabled
  audioHaptics.value = uiState.audioHaptics
  genshinCompatibility.value = uiState.genshinCompatibility
  if (uiState.tuning) {
    legacyStrength.value = uiState.tuning.strength
    legacyCurve.value = uiState.tuning.curve
    legacyNoiseGate.value = uiState.tuning.noiseGate
  }
}

const persistConfigIntent = async (requestedConfig, queue) => {
  const requestedEnabled = requestedConfig.enabled
  const requestedAudioHaptics = requestedConfig.audioHaptics
  const requestedGenshinCompatibility = requestedConfig.genshinCompatibility
  const preserveTuning = tuningDirty.value
  operationError.value = ''
  await waitForStatusRefresh()
  await waitForTuningSave()
  if (queue.hasPending()) return { success: false, preserveTuning }
  let result = await dualsense.setConfig(
    requestedEnabled,
    requestedAudioHaptics,
    requestedGenshinCompatibility,
  )
  let refreshedAfterFailure = false
  if (!result.success && queue.hasPending()) {
    return { success: false, preserveTuning, message: result.message }
  }
  const firstErrorCode = result.success ? '' : dualSenseErrorCode(result.message)
  if (!result.success && ['DS5-CFG-001', 'DS5-CFG-003'].includes(firstErrorCode)) {
    refreshedAfterFailure = await refresh(true, true)
    applyConfigControls(queue.peekPending() ?? requestedConfig)
    if (refreshedAfterFailure && dualSenseConfigMatches(status.value, requestedConfig)) {
      return { success: true, preserveTuning }
    }
    const canRetry = refreshedAfterFailure
      && !queue.hasPending()
      && !status.value.in_use
      && (!requestedEnabled || status.value.verified)
      && (!requestedEnabled || !requestedAudioHaptics || status.value.usbip_available)
      && (!requestedGenshinCompatibility || status.value.genshin_compatibility_available)
    if (canRetry) {
      result = await dualsense.setConfig(
        requestedEnabled,
        requestedAudioHaptics,
        requestedGenshinCompatibility,
      )
      refreshedAfterFailure = false
    }
  }
  if (!result.success) {
    if (!refreshedAfterFailure && !queue.hasPending()) {
      refreshedAfterFailure = await refresh(true, true)
      applyConfigControls(queue.peekPending() ?? requestedConfig)
    }
    if (refreshedAfterFailure && dualSenseConfigMatches(status.value, requestedConfig)) {
      return { success: true, preserveTuning }
    }
    return { success: false, preserveTuning, message: result.message }
  }
  status.value = result.data
  statusKnown.value = true
  return { success: true, preserveTuning }
}

const applyComponentConfig = async (requestedConfig) => {
  const queue = {
    hasPending: () => false,
    peekPending: () => undefined,
  }
  invalidateStatusRefresh()
  saving.value = true
  applyConfigControls(requestedConfig)
  let outcome
  try {
    outcome = await persistConfigIntent(requestedConfig, queue)
  } catch (error) {
    outcome = { success: false, preserveTuning: tuningDirty.value, message: error }
  }
  saving.value = false
  if (!outcome.success) {
    restoreConfirmedConfigControls()
    showError(outcome.message, 'config')
    return false
  }
  synchronizeConfirmedConfig(outcome.preserveTuning)
  return true
}

const configSaveQueue = createLatestIntentQueue(async (requestedConfig, queue) => {
  saving.value = true
  applyConfigControls(requestedConfig)
  let outcome
  try {
    outcome = await persistConfigIntent(requestedConfig, queue)
  } catch (error) {
    outcome = { success: false, preserveTuning: tuningDirty.value, message: error }
  }
  const pending = queue.peekPending()
  if (pending) {
    applyConfigControls(pending)
    return
  }

  saving.value = false
  if (!outcome.success) {
    restoreConfirmedConfigControls()
    showError(outcome.message, 'config')
    return
  }
  synchronizeConfirmedConfig(outcome.preserveTuning)
  ElMessage.success(t.value.dualSense.configSuccess)
}, { debounceMs: CONFIG_SAVE_DEBOUNCE_MS })

const saveSettings = async () => {
  if (operation.value) {
    enabled.value = status.value.enabled
    audioHaptics.value = status.value.audio_haptics
    genshinCompatibility.value = status.value.genshin_compatibility ?? false
    return
  }
  const requestedEnabled = enabled.value
  const requestedAudioHaptics = audioHaptics.value
  const requestedGenshinCompatibility = requestedEnabled && requestedAudioHaptics && genshinCompatibility.value
  const requestedConfig = {
    enabled: requestedEnabled,
    audioHaptics: requestedAudioHaptics,
    genshinCompatibility: requestedGenshinCompatibility,
  }
  invalidateStatusRefresh()
  operationError.value = ''
  saving.value = true
  applyConfigControls(requestedConfig)
  await configSaveQueue.submit(requestedConfig)
}

const setAudioHaptics = async (enabled) => {
  if (operation.value || (audioHaptics.value === enabled && !saving.value)) return
  audioHaptics.value = enabled
  if (!enabled) genshinCompatibility.value = false
  testCompleted.value = false
  await saveSettings()
}

const applyErmPreset = () => {
  legacyCurve.value = 0.5
  legacyNoiseGate.value = 0.006
}

const applyDefaultPreset = () => {
  legacyStrength.value = 1
  legacyCurve.value = 0.5
  legacyNoiseGate.value = 0.02
}

const saveTuning = async () => {
  if (controlsBusy.value) return
  const requestedTuning = {
    strength: legacyStrength.value,
    curve: legacyCurve.value,
    noiseGate: legacyNoiseGate.value,
  }
  invalidateStatusRefresh()
  operationError.value = ''
  tuningSaving.value = true
  const saveOperation = (async () => {
    await waitForStatusRefresh()
    const result = await dualsense.setHapticsTuning(
      requestedTuning.strength, requestedTuning.curve, requestedTuning.noiseGate)
    if (!result.success) return showError(result.message, 'config')
    const tuningChangedWhileSaving = legacyStrength.value !== requestedTuning.strength
      || legacyCurve.value !== requestedTuning.curve
      || legacyNoiseGate.value !== requestedTuning.noiseGate
    if (!tuningChangedWhileSaving) {
      legacyStrength.value = result.data.legacy_strength
      legacyCurve.value = result.data.legacy_curve
      legacyNoiseGate.value = result.data.legacy_noise_gate
    }
    status.value.legacy_strength = result.data.legacy_strength
    status.value.legacy_curve = result.data.legacy_curve
    status.value.legacy_noise_gate = result.data.legacy_noise_gate
    status.value.config_revision = result.data.revision
    if (!tuningChangedWhileSaving) ElMessage.success(t.value.dualSense.tuningSaved)
  })()
  tuningSavePromise = saveOperation
  try {
    await saveOperation
  } finally {
    if (tuningSavePromise === saveOperation) tuningSavePromise = null
    tuningSaving.value = false
  }
}

const setGenshinCompatibility = async (value) => {
  if (operation.value) {
    genshinCompatibility.value = status.value.genshin_compatibility ?? false
    return
  }
  genshinCompatibility.value = Boolean(value)
  await saveSettings()
}

const test = async (profile) => {
  if (controlsBusy.value) return
  invalidateStatusRefresh()
  operationError.value = ''
  operation.value = profile
  await waitForStatusRefresh()
  const result = await dualsense.selfTest(profile)
  operation.value = ''
  if (!result.success) return showError(result.message, 'test')
  testCompleted.value = true
  ElMessage.success(t.value.dualSense.testSuccess)
  await refresh()
}

const uninstall = async () => {
  if (controlsBusy.value) return
  operation.value = 'confirm-uninstall'
  try {
    await ElMessageBox.confirm(t.value.dualSense.uninstallConfirm, t.value.dualSense.uninstallTitle, {
      type: 'warning', confirmButtonText: t.value.dualSense.uninstall,
    })
  } catch {
    operation.value = ''
    return
  }
  invalidateStatusRefresh()
  operation.value = 'uninstall'
  operationError.value = ''
  await waitForStatusRefresh()
  const result = await dualsense.uninstall()
  operation.value = ''
  if (!result.success) return showError(result.message, 'uninstall')
  rebootRecommended.value = false
  status.value = mergeDualSenseStatus(status.value, result.data)
  statusKnown.value = true
  if (!(await applyComponentConfig(dualSenseConfigAfterUninstall()))) return
  ElMessage.success(t.value.dualSense.uninstallSuccess)
}

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenProgress = await listen('dualsense-operation-progress', ({ payload }) => {
      operationStageKey.value = payload?.stage || 'preparing'
      operationProgress.value = Number(payload?.progress || 0)
    })
  } catch {
    // Browser-only renderer previews do not provide the Tauri event bridge.
  }
  void dualsense.logPanelOpened()
  await refresh()
  pollTimer = window.setInterval(() => {
    refresh(true)
  }, 30000)
})
onUnmounted(() => {
  invalidateStatusRefresh()
  window.clearInterval(pollTimer)
  unlistenProgress?.()
})
</script>

<style scoped lang="less">
// Greyscale, IntelliJ-tight. Every colour, size and spacing value below
// resolves to a token from styles/brutalist.less: no local palette, no
// per-theme block, no rounding, no shadow, no motion beyond the 80ms colour
// budget the design system allows.

.ds5-page {
  max-width: 880px;
  margin: 0;
  padding: 0 0 24px;
  color: var(--text);
  background: var(--bg);
  font-family: var(--font-sans);
  font-size: var(--font-size);
  line-height: var(--line-height);
}

// ========== Page header ==========
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--gap);
  min-height: var(--row-height);
  padding: var(--pad-y) var(--pad-x);
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
}

.page-title-group {
  min-width: 0;

  h1 {
    margin: 0;
    font-size: var(--font-size);
    font-weight: 500;
    color: var(--text);
  }

  > p:last-child {
    margin: 2px 0 0;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
  }
}

.eyebrow {
  margin: 0;
  color: var(--text-faint);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
}

.experimental-tag {
  flex: 0 0 auto;
  padding: 1px 6px;
  border: 1px solid var(--border);
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  white-space: nowrap;
}

.section-label {
  display: block;
  min-height: var(--row-height);
  padding: var(--pad-y) 0 4px;
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  font-weight: 500;
}

// ========== Panel ==========
.component-panel {
  padding: 0 var(--pad-x);
}

.title-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--gap);
  min-height: var(--row-height);
  padding: var(--pad-y) 0;
  border-bottom: 1px solid var(--border);

  h2 {
    margin: 0;
    font-size: var(--font-size);
    font-weight: 500;
  }

  p {
    margin: 2px 0 0;
    max-width: 560px;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
  }
}

.enable-control {
  font-size: var(--font-size);
  font-weight: 400;
}

// ========== Status ==========
.status-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--gap);
  min-height: var(--row-height);
  padding: var(--pad-y) 0;
  border-bottom: 1px solid var(--border);
}

.status-heading,
.status-actions,
.test-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.status-heading strong {
  font-weight: 500;
}

.status-version {
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

// "Running / enabled" is a filled square, never a coloured dot. Only the two
// meaningful hues in the system - warn and error - get to overrule it.
.status-dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  background: transparent;
  border: 1px solid var(--off);
}

.state-ready .status-dot {
  background: var(--on);
  border-color: var(--on);
}

.state-update_available .status-dot,
.state-transport_missing .status-dot,
.state-in_use .status-dot {
  background: var(--warn);
  border-color: var(--warn);
}

.state-repair_required .status-dot {
  background: var(--error);
  border-color: var(--error);
}

.status-actions :deep(.el-button.menu-action-warning) {
  color: var(--warn);
  border-color: var(--warn);
}

// ========== Install progress ==========
.operation-card {
  padding: var(--pad-y) 0;
  border-bottom: 1px solid var(--border);

  div {
    display: flex;
    justify-content: space-between;
    margin-bottom: 4px;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
  }
}

.notice {
  margin: var(--pad-y) 0;
  padding-inline: 0;
  background: transparent;
}

// ========== Profile rows ==========
.profile-options {
  display: grid;
}

.profile-option {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  gap: var(--gap);
  align-items: center;
  width: 100%;
  min-height: var(--row-height);
  padding: var(--pad-y) var(--pad-x);
  border: 0;
  border-bottom: 1px solid var(--border);
  color: var(--text-dim);
  background: transparent;
  font: inherit;
  text-align: left;
  cursor: pointer;

  &.is-static {
    cursor: default;
  }

  &:disabled {
    cursor: not-allowed;
    color: var(--text-faint);
  }

  &:hover:not(.is-static):not(:disabled) {
    background: var(--bg-hover);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  &.is-selected {
    color: var(--text);
    background: var(--bg-active);
  }

  kbd {
    min-width: 56px;
    padding: 1px 6px;
    border: 1px solid var(--border);
    color: var(--text-dim);
    background: transparent;
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    text-align: center;
  }
}

// The selection marker is a filled square, not a glyph.
.selection-cursor {
  width: 8px;
  height: 8px;
  background: transparent;
}

.profile-option.is-selected .selection-cursor {
  background: var(--on);
}

.option-copy {
  min-width: 0;

  strong,
  small {
    display: block;
  }

  strong {
    font-weight: 500;
  }

  small {
    margin-top: 2px;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
    line-height: 1.45;
  }
}

// ========== Haptics tuning ==========
.tuning-section {
  padding-bottom: var(--pad-x);
  border-bottom: 1px solid var(--border);
}

.tuning-heading,
.tuning-field {
  display: grid;
  gap: 4px;
}

.tuning-hint {
  margin: 0 0 var(--pad-y);
  max-width: 680px;
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  line-height: 1.45;
}

.tuning-presets {
  padding: var(--pad-y) 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);

  > span {
    display: block;
    margin-bottom: 4px;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
  }

  > div {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
  }
}

.tuning-preset {
  display: grid;
  gap: 2px;
  min-height: var(--control-height);
  padding: var(--pad-y) var(--pad-x);
  border: 1px solid var(--border);
  border-radius: 0;
  color: var(--text);
  background: var(--bg-input);
  font: inherit;
  text-align: left;
  cursor: pointer;

  strong {
    font-size: var(--font-size);
    font-weight: 500;
  }

  small {
    color: var(--text-dim);
    font-size: var(--font-size-sm);
    line-height: 1.4;
  }

  &:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-strong);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 1px;
  }

  &:disabled {
    color: var(--text-faint);
    cursor: not-allowed;
  }
}

.tuning-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--gap);
  padding: var(--pad-y) 0;
}

.tuning-field {
  color: var(--text-dim);
  font-size: var(--font-size-sm);
}

.tuning-save-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.tuning-unsaved-tag {
  border-radius: 0;
  color: var(--text-dim);
  border-color: var(--border);
  background: transparent;
}

// ========== Compatibility ==========
.compatibility-option {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--gap);
  align-items: center;
  min-height: var(--row-height);
  padding: var(--pad-y) var(--pad-x);
  border-bottom: 1px solid var(--border);
}

.compatibility-notice {
  display: flex;
  gap: 6px;
  margin: var(--pad-y) var(--pad-x) 0;
  color: var(--warn);
  font-size: var(--font-size-sm);
  line-height: 1.45;

  span {
    flex: 0 0 auto;
    font-family: var(--font-mono);
    font-weight: 600;
  }
}

// ========== Validation ==========
.test-section {
  display: grid;
  gap: var(--pad-y);
  padding: var(--pad-x) 0;
  border-bottom: 1px solid var(--border);

  strong {
    font-weight: 500;
  }

  p {
    margin: 2px 0 0;
    max-width: 680px;
    color: var(--text-dim);
    font-size: var(--font-size-sm);
    line-height: 1.45;
  }
}

.controller-meta-action {
  flex: 0 0 100%;
}

.controller-meta-action :deep(.el-button) {
  margin-left: 0;
}

// ========== Component health ==========
.details-collapse {
  margin-top: var(--pad-x);
  border: 0;

  :deep(.el-collapse-item__header) {
    height: var(--row-height);
    line-height: var(--row-height);
    padding: 0;
    border: 0;
    border-bottom: 1px solid var(--border);
    color: var(--text-dim);
    background: transparent;
    font-family: inherit;
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  :deep(.el-collapse-item__wrap) {
    border: 0;
    background: transparent;
  }

  :deep(.el-collapse-item__content) {
    padding-bottom: 0;
    color: inherit;
    font-family: inherit;
    font-size: var(--font-size);
  }
}

.health-list {
  display: grid;
}

.health-row {
  display: grid;
  grid-template-columns: minmax(140px, 1fr) minmax(110px, auto) minmax(0, 1fr);
  gap: var(--gap);
  align-items: center;
  min-height: var(--row-height);
  padding: 0 var(--pad-x);
  border-bottom: 1px solid var(--border);
}

.health-state {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;

  i {
    width: 8px;
    height: 8px;
    flex: 0 0 8px;
    background: transparent;
    border: 1px solid var(--off);
  }

  i.ok,
  i.busy {
    background: var(--on);
    border-color: var(--on);
  }

  i.warn {
    background: var(--warn);
    border-color: var(--warn);
  }

  i.bad {
    background: var(--error);
    border-color: var(--error);
  }
}

.health-detail {
  min-width: 0;
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  text-align: right;
  overflow-wrap: anywhere;
}

.panel-footer {
  display: grid;
  gap: var(--pad-y);
  justify-items: start;
  padding: var(--pad-x);
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  overflow-wrap: anywhere;

  details summary {
    cursor: pointer;

    &:focus-visible {
      outline: 1px solid var(--border-strong);
      outline-offset: 0;
    }
  }

  pre {
    margin: 6px 0 0;
    padding: var(--pad-y) var(--pad-x);
    background: var(--bg-input);
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
}

// ========== Responsive ==========
@media (max-width: 760px) {
  .health-row {
    grid-template-columns: minmax(0, 1fr) minmax(0, auto);
    padding: var(--pad-y) var(--pad-x);
  }

  .health-detail {
    grid-column: 1 / -1;
    text-align: left;
  }
}

@media (max-width: 600px) {
  .page-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .title-row,
  .status-row,
  .test-section {
    grid-template-columns: 1fr;
    gap: var(--pad-y);
  }

  .enable-control {
    justify-self: stretch;
  }

  .status-actions,
  .test-actions {
    justify-content: flex-start;
  }

  .profile-option {
    grid-template-columns: 10px minmax(0, 1fr) auto;
    padding-inline: 6px;
  }

  .tuning-presets > div,
  .tuning-grid {
    grid-template-columns: 1fr;
  }

  .compatibility-option {
    grid-template-columns: 1fr;
    gap: 6px;
    padding-inline: 6px;
  }
}
</style>
