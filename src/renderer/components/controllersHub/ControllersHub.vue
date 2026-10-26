<template>
  <section class="chub-page">
    <header class="chub-header">
      <span class="chub-header-icon" aria-hidden="true"><IconGamepad /></span>
      <div class="chub-header-copy">
        <h1>{{ t.deviceHub.title }}</h1>
        <p class="chub-intro">{{ t.deviceHub.intro }}</p>
      </div>
    </header>

    <div class="chub-workflow">
      <section class="chub-workflow-section" aria-labelledby="device-hub-controllers-title">
        <h2 id="device-hub-controllers-title" class="chub-workflow-title">
          {{ t.deviceHub.tabs.controllers }}
        </h2>
        <div class="chub-workflow-content">
          <GamepadTypePicker @ds5-selection-change="ds5Selected = $event" />
          <DualSenseSettings
            v-if="ds5Selected"
            :settings="dualSenseSettings"
            embedded
            @open-controller-meta="emit('open-controller-meta')"
            @manage-components="focusRuntimeComponents"
          />
          <div class="chub-advanced-block">
            <el-collapse v-model="controllerAdvancedOpen" class="chub-advanced">
              <el-collapse-item name="advanced" :title="t.deviceHub.controllerAdvanced">
                <AdvancedControllerOptions />
              </el-collapse-item>
            </el-collapse>
          </div>
        </div>
      </section>

      <section class="chub-workflow-section" aria-labelledby="device-hub-microphone-title">
        <h2 id="device-hub-microphone-title" class="chub-workflow-title">
          {{ t.deviceHub.tabs.microphone }}
        </h2>
        <div class="chub-workflow-content">
          <MicrophonePanel @manage-components="focusRuntimeComponents" />
        </div>
      </section>

      <section class="chub-workflow-section" aria-labelledby="device-hub-usb-title">
        <h2 id="device-hub-usb-title" class="chub-workflow-title">
          {{ t.deviceHub.tabs.usb }}
        </h2>
        <div class="chub-workflow-content">
          <UsbPassthroughPanel :transport="usbTransport" />
        </div>
      </section>

      <section
        ref="runtimeComponentsSection"
        class="chub-workflow-section"
        tabindex="-1"
        aria-labelledby="device-hub-components-title"
      >
        <h2 id="device-hub-components-title" class="chub-workflow-title">
          {{ t.deviceHub.tabs.components }}
        </h2>
        <div class="chub-workflow-content">
          <RuntimeComponentsPanel
            :component="dualSenseSettings.component"
            :transport="usbTransport"
          />
        </div>
      </section>

      <section class="chub-workflow-section" aria-labelledby="device-hub-tools-title">
        <h2 id="device-hub-tools-title" class="chub-workflow-title">
          {{ t.deviceHub.tabs.tools }}
        </h2>
        <div class="chub-workflow-content">
          <DeviceDiagnosticsPanel
            @open-controller-meta="emit('open-controller-meta')"
            @open-stylus-input-probe="emit('open-stylus-input-probe')"
            @open-hdr-enhanced="emit('open-hdr-enhanced')"
          />
        </div>
      </section>
    </div>
  </section>
</template>

<script setup>
import { ref } from 'vue'
import { Gamepad2 as IconGamepad } from '@lucide/vue'
import DualSenseSettings from '../DualSenseSettings.vue'
import AdvancedControllerOptions from './AdvancedControllerOptions.vue'
import GamepadTypePicker from './GamepadTypePicker.vue'
import MicrophonePanel from './MicrophonePanel.vue'
import DeviceDiagnosticsPanel from './DeviceDiagnosticsPanel.vue'
import RuntimeComponentsPanel from './RuntimeComponentsPanel.vue'
import UsbPassthroughPanel from './UsbPassthroughPanel.vue'
import { useDualSenseSettings } from '../../composables/useDualSenseSettings.js'
import { useUsbTransport } from '../../composables/useUsbTransport.js'
import { useI18n } from '../../i18n/index.js'

const emit = defineEmits(['open-controller-meta', 'open-stylus-input-probe', 'open-hdr-enhanced'])
const { t } = useI18n()
const ds5Selected = ref(false)
const controllerAdvancedOpen = ref([])
const runtimeComponentsSection = ref(null)
const dualSenseSettings = useDualSenseSettings()
const usbTransport = useUsbTransport()

function focusRuntimeComponents() {
  const target = runtimeComponentsSection.value
  if (!target) return
  target.scrollIntoView({ block: 'start', behavior: 'auto' })
  target.focus({ preventScroll: true })
}
</script>

<style lang="less">
@import '../../styles/ControllersHub.less';
</style>
