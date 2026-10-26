<template>
  <div class="ui-notice-stack" aria-live="polite" aria-atomic="false">
    <div
      v-for="notice in feedbackState.notices"
      :key="notice.id"
      class="ui-notice"
      :data-tone="notice.tone"
      role="status"
    >
      <span>{{ notice.text }}</span>
      <button type="button" aria-label="Dismiss notification" @click="dismissNotice(notice.id)">×</button>
    </div>
  </div>

  <div v-if="activeDialog" class="ui-dialog-backdrop" role="presentation">
    <section
      class="ui-dialog"
      role="alertdialog"
      aria-modal="true"
      :aria-labelledby="`ui-dialog-title-${activeDialog.id}`"
      :aria-describedby="`ui-dialog-body-${activeDialog.id}`"
    >
      <header :id="`ui-dialog-title-${activeDialog.id}`">{{ activeDialog.title }}</header>
      <p :id="`ui-dialog-body-${activeDialog.id}`">{{ activeDialog.message }}</p>
      <footer>
        <button
          v-if="activeDialog.kind === 'confirm'"
          ref="cancelButton"
          type="button"
          class="ui-button ui-button-outline"
          @click="settleDialog(activeDialog, false)"
        >
          {{ activeDialog.cancelText }}
        </button>
        <button
          ref="confirmButton"
          type="button"
          class="ui-button ui-button-primary"
          @click="settleDialog(activeDialog, true)"
        >
          {{ activeDialog.confirmText }}
        </button>
      </footer>
    </section>
  </div>
</template>

<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { dismissNotice, feedbackState, settleDialog } from './feedback.js'

const activeDialog = computed(() => feedbackState.dialogs[0] || null)
const confirmButton = ref(null)
const cancelButton = ref(null)

watch(activeDialog, async (dialog) => {
  if (!dialog) return
  await nextTick()
  ;(dialog.kind === 'confirm' ? cancelButton.value : confirmButton.value)?.focus()
})
</script>
