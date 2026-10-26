<template>
  <div class="webstream-settings-wrapper">
    <div class="webstream-header">
      <h2>{{ t.webStream.title }}</h2>
    </div>

    <div class="webstream-content">
      <div class="webstream-form">
        <!-- Description -->
        <p class="section-desc">{{ t.webStream.desc }}</p>

        <!-- Status -->
        <div class="status-section">
          <div class="status-row">
            <div class="status-info">
              <span class="status-dot" :class="status.running ? 'bru-on' : 'bru-off'" aria-hidden="true" />
              <span class="status-label">{{ status.running ? t.webStream.statusRunning : (status.installed ? t.webStream.statusStopped : t.webStream.statusNotInstalled) }}</span>
              <span v-if="status.version" class="status-version">v{{ status.version }}</span>
            </div>
            <div v-if="status.installed" class="status-actions">
              <el-button
                v-if="!status.running"
                type="primary"
                :loading="starting"
                @click="handleStart"
              >
                {{ t.webStream.start }}
              </el-button>
              <el-button
                v-else
                type="danger"
                :loading="stopping"
                @click="handleStop"
              >
                {{ t.webStream.stop }}
              </el-button>
            </div>
          </div>
        </div>

        <!-- Access links -->
        <div v-if="status.running" class="access-section">
          <div class="section-title">{{ t.webStream.accessLink }}</div>
          <div class="access-url-row">
            <el-input :model-value="status.access_url" readonly class="url-input">
              <template #prepend>URL</template>
            </el-input>
            <el-button type="primary" @click="copyUrl">{{ t.webStream.copy }}</el-button>
            <el-button @click="openInBrowser">{{ t.webStream.open }}</el-button>
          </div>
          <p class="form-tip">
            {{ t.webStream.accessTip.replace('{port}', status.port) }}
          </p>
        </div>

        <!-- Configuration -->
        <template v-if="status.installed">
          <div class="section-divider">
            <span class="section-divider-text">{{ t.webStream.serviceConfig }}</span>
          </div>

          <el-form :model="config" label-width="170px" class="config-form">
            <el-form-item :label="t.webStream.bindAddress">
              <el-input v-model="config.web_server.bind_address" placeholder="0.0.0.0:8080" />
              <span class="form-tip" v-html="t.webStream.bindAddressTip.replace('{allIp}', '<code>0.0.0.0:port</code>').replace('{localhost}', '<code>127.0.0.1:port</code>')"></span>
            </el-form-item>

            <el-form-item :label="t.webStream.httpsCert">
              <div class="setting-content">
                <el-switch v-model="httpsEnabled" />
                <span class="form-tip">{{ httpsEnabled ? t.webStream.httpsEnabled : t.webStream.httpsDisabled }}</span>
              </div>
            </el-form-item>

            <template v-if="httpsEnabled">
              <el-form-item :label="t.webStream.privateKey">
                <el-input v-model="certKeyPath" placeholder="./server/key.pem" />
              </el-form-item>
              <el-form-item :label="t.webStream.certFile">
                <el-input v-model="certPemPath" placeholder="./server/cert.pem" />
              </el-form-item>
              <el-form-item>
                <el-button @click="handleGenerateCert" :loading="generatingCert">
                  {{ t.webStream.generateCert }}
                </el-button>
                <span class="form-tip">{{ t.webStream.generateCertTip }}</span>
              </el-form-item>
            </template>

            <div class="section-divider">
              <span class="section-divider-text">{{ t.webStream.defaultStreamSettings }}</span>
            </div>

            <div class="two-column-layout">
              <el-form-item :label="t.webStream.videoCodec">
                <el-select v-model="defaultVideoCodec" style="width: 100%">
                  <el-option :label="t.webStream.h264Label" value="h264" />
                  <el-option label="H.265 / HEVC" value="h265" />
                  <el-option label="AV1" value="av1" />
                  <el-option :label="t.webStream.autoLabel" value="auto" />
                </el-select>
              </el-form-item>

              <el-form-item :label="t.webStream.defaultFps">
                <el-select v-model="defaultFps" style="width: 100%">
                  <el-option :label="30" :value="30" />
                  <el-option :label="60" :value="60" />
                  <el-option :label="120" :value="120" />
                </el-select>
              </el-form-item>

              <el-form-item :label="t.webStream.bitrate">
                <el-input-number v-model="defaultBitrate" :min="1000" :max="150000" :step="1000" style="width: 100%" />
              </el-form-item>

              <el-form-item :label="t.webStream.transportMode">
                <el-select v-model="defaultTransport" style="width: 100%">
                  <el-option :label="t.webStream.transportAuto" value="auto" />
                  <el-option label="WebRTC" value="webrtc" />
                  <el-option label="WebSocket" value="websocket" />
                </el-select>
              </el-form-item>
            </div>

            <div class="form-actions">
              <el-form-item>
                <el-button type="primary" @click="handleSaveConfig" :loading="saving">
                  {{ t.webStream.saveConfig }}
                </el-button>
                <el-button @click="loadConfig">
                  {{ t.webStream.reload }}
                </el-button>
                <span v-if="configDirty" class="unsaved-hint">{{ t.webStream.unsavedChanges }}</span>
              </el-form-item>
            </div>
          </el-form>
        </template>

        <!-- Installation path -->
        <div v-if="status.installed" class="install-path">
          {{ t.webStream.installPath.replace('{path}', status.install_path) }}
        </div>
      </div>
    </div>
  </div>
</template>


<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from './ui/compact/feedback.js'
import { moonlightWeb } from '../tauri-adapter.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const emit = defineEmits(['close'])

// State
const status = reactive({
  installed: false,
  running: false,
  install_path: '',
  version: '',
  access_url: '',
  port: 8080,
})

const config = reactive({
  web_server: { bind_address: '0.0.0.0:8080' },
  webrtc: null,
  default_settings: null,
})

// UI state
const starting = ref(false)
const stopping = ref(false)
const saving = ref(false)
const configDirty = ref(false)
const generatingCert = ref(false)
const httpsEnabled = ref(false)
const certKeyPath = ref('./server/key.pem')
const certPemPath = ref('./server/cert.pem')
const defaultVideoCodec = ref('h264')
const defaultBitrate = ref(20000)
const defaultFps = ref(60)
const defaultTransport = ref('auto')

// Status polling
let pollTimer = null

// ========== Lifecycle ==========

onMounted(async () => {
  await refreshStatus()
  if (status.installed) {
    await loadConfig()
  }
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})

// Mark the form dirty when the configuration changes.
watch([() => config.web_server.bind_address, httpsEnabled, certKeyPath, certPemPath, defaultVideoCodec, defaultBitrate, defaultFps, defaultTransport], () => {
  configDirty.value = true
})

// ========== Methods ==========

async function refreshStatus() {
  try {
    const s = await moonlightWeb.getStatus()
    Object.assign(status, s)
  } catch (e) {
    console.error('Failed to get status:', e)
  }
}

async function loadConfig() {
  try {
    const c = await moonlightWeb.getConfig()
    Object.assign(config, c)

    // Make sure bind_address is not empty.
    if (!config.web_server?.bind_address) {
      config.web_server.bind_address = '0.0.0.0:8080'
    }

    // Resolve the HTTPS state.
    httpsEnabled.value = !!config.web_server?.certificate
    if (config.web_server?.certificate) {
      certKeyPath.value = config.web_server.certificate.private_key_pem || './server/key.pem'
      certPemPath.value = config.web_server.certificate.certificate_pem || './server/cert.pem'
    }

    // Resolve the default settings.
    const ds = config.default_settings || {}
    defaultVideoCodec.value = ds.videoCodec || 'h264'
    defaultBitrate.value = ds.bitrate || 20000
    defaultFps.value = ds.fps || 60
    defaultTransport.value = ds.dataTransport || 'auto'

    configDirty.value = false
  } catch (e) {
    console.error('Failed to load config:', e)
  }
}

function startPolling() {
  pollTimer = setInterval(refreshStatus, 5000)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

async function handleStart() {
  starting.value = true
  try {
    const msg = await moonlightWeb.start()
    ElMessage.success(msg)
    await refreshStatus()
  } catch (e) {
    ElMessage.error(t.value.webStream.startFailed.replace('{error}', e))
  } finally {
    starting.value = false
  }
}

async function handleStop() {
  stopping.value = true
  try {
    const msg = await moonlightWeb.stop()
    ElMessage.success(msg)
    await refreshStatus()
  } catch (e) {
    ElMessage.error(t.value.webStream.stopFailed.replace('{error}', e))
  } finally {
    stopping.value = false
  }
}

async function handleGenerateCert() {
  generatingCert.value = true
  try {
    const result = await moonlightWeb.generateCert()
    certKeyPath.value = result.private_key_pem
    certPemPath.value = result.certificate_pem
    // Save the configuration after generating.
    await handleSaveConfig()
    // Prompt for a restart when the service is running.
    if (status.running) {
      try {
        await ElMessageBox.confirm(
          t.value.webStream.certGenRestart,
          t.value.webStream.certTitle,
          { confirmButtonText: t.value.webStream.restartService, cancelButtonText: t.value.webStream.laterManualRestart }
        )
        await moonlightWeb.stop()
        await new Promise(resolve => setTimeout(resolve, 500))
        await moonlightWeb.start()
        await refreshStatus()
        ElMessage.success(t.value.webStream.serviceRestarted)
      } catch {
        ElMessage.info(t.value.webStream.manualRestartHint)
      }
    } else {
      ElMessage.success(t.value.webStream.certGenSuccess)
    }
  } catch (e) {
    ElMessage.error(t.value.webStream.certGenFailed.replace('{error}', e))
  } finally {
    generatingCert.value = false
  }
}

async function handleSaveConfig() {
  saving.value = true
  try {
    // Update on top of the full configuration so upstream fields survive.
    const saveConfig = JSON.parse(JSON.stringify(config))

    // Update the user-editable fields.
    saveConfig.web_server.bind_address = config.web_server.bind_address

    if (httpsEnabled.value) {
      saveConfig.web_server.certificate = {
        private_key_pem: certKeyPath.value,
        certificate_pem: certPemPath.value,
      }
      saveConfig.web_server.session_cookie_secure = true
    } else {
      delete saveConfig.web_server.certificate
      saveConfig.web_server.session_cookie_secure = false
    }

    saveConfig.default_settings = {
      ...(saveConfig.default_settings || {}),
      videoCodec: defaultVideoCodec.value,
      bitrate: defaultBitrate.value,
      fps: defaultFps.value,
      dataTransport: defaultTransport.value,
    }

    await moonlightWeb.saveConfig(saveConfig)
    ElMessage.success(t.value.webStream.configSaved)
    configDirty.value = false
  } catch (e) {
    ElMessage.error(t.value.webStream.configSaveFailed.replace('{error}', e))
  } finally {
    saving.value = false
  }
}

function copyUrl() {
  navigator.clipboard.writeText(status.access_url).then(() => {
    ElMessage.success(t.value.webStream.copied)
  }).catch(() => {
    ElMessage.warning(t.value.webStream.copyFailed)
  })
}

async function openInBrowser() {
  try {
    const { openExternalUrl } = await import('../tauri-adapter.js')
    await openExternalUrl(status.access_url)
  } catch (e) {
    window.open(status.access_url, '_blank')
  }
}
</script>


<style scoped lang="less">
// Greyscale, IntelliJ settings-dialog density. Every value below resolves to a
// token from app/app.css - no local palette, no per-theme blocks.

.webstream-settings-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-sans);
  font-size: var(--font-size);
  line-height: var(--line-height);
}

// ========== Header ==========
.webstream-header {
  display: flex;
  align-items: center;
  flex: 0 0 auto;
  min-height: var(--row-height);
  padding: var(--pad-y) var(--pad-x);
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);

  h2 {
    margin: 0;
    font-size: var(--font-size);
    font-weight: 500;
    color: var(--text);
  }
}

// ========== Content ==========
.webstream-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--pad-x);
}

.webstream-form {
  max-width: 820px;
}

// ========== Copy ==========
.section-desc {
  margin: 0 0 var(--pad-x);
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  line-height: 1.5;
}

.section-divider {
  display: flex;
  align-items: center;
  min-height: var(--row-height);
  margin: 16px 0 0;
  border-bottom: 1px solid var(--border);

  .section-divider-text {
    color: var(--text-dim);
    font-size: var(--font-size-sm);
    font-weight: 500;
  }
}

// ========== Status ==========
.status-section {
  padding: var(--pad-y) var(--pad-x);
  margin-bottom: var(--pad-x);
  background: var(--bg-panel);
  border: 1px solid var(--border);
}

.status-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--gap);
  min-height: var(--row-height);
}

.status-info {
  display: flex;
  align-items: center;
  gap: var(--gap);
}

.status-dot {
  flex-shrink: 0;
}

.status-label {
  color: var(--text);
  font-weight: 500;
}

.status-version {
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
}

.status-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

// ========== Access links ==========
.access-section {
  padding: var(--pad-y) var(--pad-x);
  margin-bottom: var(--pad-x);
  background: var(--bg-panel);
  border: 1px solid var(--border);

  .section-title {
    margin-bottom: var(--pad-y);
    color: var(--text-dim);
    font-size: var(--font-size-sm);
    font-weight: 500;
  }
}

.access-url-row {
  display: flex;
  gap: 6px;
  align-items: center;

  .url-input {
    flex: 1;

    :deep(.el-input__inner) {
      font-family: var(--font-mono);
    }
  }
}

// ========== Form ==========
.config-form {
  // Label left, control right, one hairline between rows.
  :deep(.el-form-item) {
    align-items: flex-start;
    margin: 0;
    padding: var(--pad-y) 0;
    border-bottom: 1px solid var(--border);
  }

  :deep(.el-form-item__label) {
    padding-right: var(--gap);
    color: var(--text-dim);
    font-size: var(--font-size);
    font-weight: 400;
    line-height: var(--row-height);
  }

  :deep(.el-form-item__content) {
    min-height: var(--row-height);
  }
}

.form-tip {
  margin-top: 2px;
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  line-height: 1.5;

  code {
    padding: 0 3px;
    color: var(--text);
    background: var(--bg-input);
    border: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
  }
}

.setting-content {
  display: flex;
  align-items: center;
  gap: var(--gap);
}

.two-column-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0 24px;

  :deep(.el-form-item) {
    margin-bottom: 0;
  }
}

.form-actions {
  margin-top: var(--pad-x);

  :deep(.el-form-item) {
    border-bottom: 0;
  }

  :deep(.el-form-item__content) {
    gap: var(--gap);
  }
}

.unsaved-hint {
  color: var(--warn);
  font-size: var(--font-size-sm);
}

.install-path {
  padding: var(--pad-x) 0 0;
  color: var(--text-faint);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  overflow-wrap: anywhere;
}

@media (max-width: 768px) {
  .two-column-layout {
    grid-template-columns: 1fr;
  }
}
</style>
