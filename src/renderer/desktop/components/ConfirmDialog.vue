<template>
  <Teleport to="body">
    <Transition name="confirm">
      <div v-if="state.open" class="confirm-mask" @keydown.esc.prevent="reject">
        <div ref="dialogRef" class="confirm-dialog" :class="{ danger: state.danger }">
          <h3 v-if="state.title" class="confirm-title">{{ state.title }}</h3>
          <p class="confirm-message">{{ state.message }}</p>
          <div class="confirm-actions">
            <button
              type="button"
              class="confirm-btn"
              data-focusable
              data-focus-key="confirm-cancel"
              @click="reject"
            >
              {{ state.cancelLabel || t.common.cancel }}
            </button>
            <button
              type="button"
              class="confirm-btn primary"
              data-focusable
              data-focus-key="confirm-accept"
              @click="accept"
            >
              {{ state.confirmLabel || t.common.confirm }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref } from 'vue'
import { acceptConfirm, confirmState, rejectConfirm } from '../composables/useConfirm.js'
import { useModalFocusScope } from '../composables/useFocusNav.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const state = confirmState
const dialogRef = ref(null)
useModalFocusScope(dialogRef, () => state.value.open)

function accept() {
  acceptConfirm()
}

function reject() {
  rejectConfirm()
}
</script>

<style lang="less" scoped>
.confirm-mask {
  position: fixed;
  inset: 0;
  z-index: 21000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--scrim);
}

.confirm-dialog {
  width: min(460px, 88vw);
  padding: var(--space-5);
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-md);

  &.danger {
    border-color: var(--error);
  }
}

.confirm-title {
  margin: 0 0 var(--space-2);
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.confirm-message {
  margin: 0 0 var(--space-4);
  font-size: var(--font-size);
  line-height: var(--line-height);
  color: var(--text-dim);
  white-space: pre-line;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
}

.confirm-btn {
  min-height: var(--control-height);
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

  &.primary {
    background: var(--bg-active);
    border-color: var(--border-strong);
    font-weight: 600;
  }
}

.danger .confirm-btn.primary {
  border-color: var(--error);
  color: var(--error);
}
</style>
