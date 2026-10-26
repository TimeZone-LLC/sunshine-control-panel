<template>
  <div class="vmouse-settings-wrapper">
    <div class="vmouse-header">
      <h2>{{ t.vmouseSettings.title }}</h2>
    </div>

    <div class="vmouse-content">
      <el-form label-width="170px" class="vmouse-form">
        <!-- Driver status -->
        <el-form-item :label="t.vmouseSettings.driverStatus">
          <div class="status-row">
            <span class="status-indicator" :class="status.running ? 'bru-on' : 'bru-off'" aria-hidden="true"></span>
            <span class="status-state">
              {{ status.installed ? (status.running ? t.vmouseSettings.running : t.vmouseSettings.installed) : t.vmouseSettings.notInstalled }}
            </span>
            <span class="status-detail" v-if="status.status_text">{{ status.status_text }}</span>
            <el-button size="small" :icon="Refresh" @click="refreshStatus" :loading="refreshing">Refresh</el-button>
          </div>
        </el-form-item>

        <!-- Enable switch, bound to virtual_mouse in sunshine.conf -->
        <el-form-item :label="t.vmouseSettings.functionSwitch">
          <el-switch
            v-model="configEnabled"
            @change="handleConfigToggle"
            :loading="configSaving"
            :active-text="t.vmouseSettings.switchEnabled"
            :inactive-text="t.vmouseSettings.switchDisabled"
          />
          <span class="form-tip">{{ t.vmouseSettings.switchTip }}</span>
        </el-form-item>

        <!-- Driver path -->
        <el-form-item :label="t.vmouseSettings.driverPath" v-if="status.driver_path">
          <span class="driver-path">{{ status.driver_path }}</span>
        </el-form-item>

        <!-- Description -->
        <el-form-item :label="t.vmouseSettings.description">
          <div class="description">
            <p>{{ t.vmouseSettings.descText1 }}</p>
            <p>{{ t.vmouseSettings.descText2 }}</p>
          </div>
        </el-form-item>

        <!-- Install and uninstall buttons -->
        <el-form-item class="form-actions">
          <el-button
            v-if="!status.installed"
            type="primary"
            @click="installDriver"
            :loading="installing"
          >
            <el-icon><Download /></el-icon>
            {{ t.vmouseSettings.installDriver }}
          </el-button>
          <el-button
            v-else
            type="danger"
            @click="uninstallDriver"
            :loading="uninstalling"
          >
            <el-icon><Delete /></el-icon>
            {{ t.vmouseSettings.uninstallDriver }}
          </el-button>
        </el-form-item>
      </el-form>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from './ui/compact/feedback.js'
import { Refresh, Download, Delete } from './ui/compact/icons.js'
import { vmouse } from '../tauri-adapter.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const status = reactive({
  installed: false,
  running: false,
  status_text: t.value.vmouseSettings.detecting,
  driver_path: '',
  config_enabled: true,
})

const configEnabled = ref(true)
const refreshing = ref(false)
const installing = ref(false)
const uninstalling = ref(false)
const configSaving = ref(false)

const refreshStatus = async () => {
  refreshing.value = true
  try {
    const result = await vmouse.getStatus()
    if (result?.success) {
      Object.assign(status, result.data)
      configEnabled.value = result.data.config_enabled
    }
  } catch (error) {
    console.error('Failed to get vmouse status:', error)
  } finally {
    refreshing.value = false
  }
}

const handleConfigToggle = async (enabled) => {
  configSaving.value = true
  try {
    const result = await vmouse.setConfig(enabled)
    if (result?.success) {
      ElMessage.success(result.data)
    } else {
      throw new Error(result?.message || t.value.vmouseSettings.unknownError)
    }
  } catch (error) {
    ElMessage.error(t.value.vmouseSettings.setFailed.replace('{error}', error.message || error))
    // Roll the switch back.
    configEnabled.value = !enabled
  } finally {
    configSaving.value = false
  }
}

const installDriver = async () => {
  try {
    await ElMessageBox.confirm(
      t.value.vmouseSettings.installConfirm,
      t.value.vmouseSettings.installTitle,
      {
        confirmButtonText: t.value.vmouseSettings.installBtn,
        cancelButtonText: t.value.vmouseSettings.cancelBtn,
        type: 'info',
      }
    )

    installing.value = true
    const result = await vmouse.install()
    if (result?.success) {
      ElMessage.success(result.data)
      // Give the driver a moment before refreshing the status.
      setTimeout(() => refreshStatus(), 2000)
    } else {
      throw new Error(result?.message || t.value.vmouseSettings.installFailed)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(t.value.vmouseSettings.installError.replace('{error}', error.message || error))
    }
  } finally {
    installing.value = false
  }
}

const uninstallDriver = async () => {
  try {
    await ElMessageBox.confirm(
      t.value.vmouseSettings.uninstallConfirm,
      t.value.vmouseSettings.uninstallTitle,
      {
        confirmButtonText: t.value.vmouseSettings.uninstallBtn,
        cancelButtonText: t.value.vmouseSettings.cancelBtn,
        type: 'warning',
      }
    )

    uninstalling.value = true
    const result = await vmouse.uninstall()
    if (result?.success) {
      ElMessage.success(result.data)
      setTimeout(() => refreshStatus(), 2000)
    } else {
      throw new Error(result?.message || t.value.vmouseSettings.uninstallFailed)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(t.value.vmouseSettings.uninstallError.replace('{error}', error.message || error))
    }
  } finally {
    uninstalling.value = false
  }
}

onMounted(() => {
  refreshStatus()
})
</script>


<style lang="less" scoped>
// Greyscale, IntelliJ settings-dialog density. Every colour, size and spacing
// value below resolves to a token from app/app.css; nothing local.

.vmouse-settings-wrapper {
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
.vmouse-header {
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
.vmouse-content {
  flex: 1;
  overflow-y: auto;
  padding: 0;
}

.vmouse-form {
  max-width: 760px;
  border-bottom: 1px solid var(--border);

  // Label left, control right, one hairline between rows.
  :deep(.el-form-item) {
    align-items: flex-start;
    margin: 0;
    padding: var(--pad-y) var(--pad-x);
    border-top: 1px solid var(--border);
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
    line-height: var(--line-height);
  }
}

// ========== Status row ==========
.status-row {
  display: flex;
  align-items: center;
  gap: var(--gap);
  flex-wrap: wrap;
  min-height: var(--row-height);
}

.status-indicator {
  flex: 0 0 auto;
}

.status-state {
  color: var(--text);
  font-weight: 500;
}

.status-detail {
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  overflow-wrap: anywhere;
}

.driver-path {
  display: block;
  padding: var(--pad-y) 0;
  color: var(--text-dim);
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  word-break: break-all;
}

// ========== Copy ==========
.form-tip {
  margin-left: var(--gap);
  color: var(--text-dim);
  font-size: var(--font-size-sm);
}

.description {
  max-width: 620px;
  padding: 2px 0;
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  line-height: 1.5;

  p {
    margin: 0 0 6px;

    &:last-child {
      margin-bottom: 0;
    }
  }
}

// ========== Actions ==========
.form-actions {
  :deep(.el-form-item__content) {
    gap: var(--gap);
  }
}
</style>
