<template>
  <section class="chub-panel chub-runtime-components">
    <p class="chub-hint">{{ t.deviceHub.components.managementHint }}</p>
    <DualSenseComponentPanel :component="component" :description="t.deviceHub.components.hostHint" />
    <UsbTransportPanel :transport="transport" maintenance />
    <InputDriversPanel />
  </section>
</template>

<script setup>
import InputDriversPanel from './InputDriversPanel.vue'
import DualSenseComponentPanel from './DualSenseComponentPanel.vue'
import { useDualSenseComponent } from '../../composables/useDualSenseComponent.js'
import UsbTransportPanel from './UsbTransportPanel.vue'
import { useUsbTransport } from '../../composables/useUsbTransport.js'
import { useI18n } from '../../i18n/index.js'

const props = defineProps({
  component: { type: Object, default: null },
  transport: { type: Object, default: null },
})
const { t } = useI18n()
const component = props.component || useDualSenseComponent()
const transport = props.transport || useUsbTransport()
</script>
