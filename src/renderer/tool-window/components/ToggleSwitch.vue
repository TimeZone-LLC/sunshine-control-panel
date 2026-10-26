<template>
  <button
    type="button"
    class="toggle-switch"
    :class="{ on: modelValue }"
    role="switch"
    :aria-checked="modelValue ? 'true' : 'false'"
    :disabled="disabled"
    @click="onToggle"
  >
    <span class="toggle-thumb"></span>
  </button>
</template>

<script setup>
const props = defineProps({
  modelValue: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})
const emit = defineEmits(['update:modelValue'])

function onToggle() {
  if (props.disabled) return
  emit('update:modelValue', !props.modelValue)
}
</script>

<style lang="less" scoped>
.toggle-switch {
  width: 38px;
  height: 20px;
  padding: 0;
  background: var(--bg-input);
  border: 1px solid var(--border-strong);
  cursor: pointer;
  transition: background-color 120ms ease, border-color 120ms ease;
  position: relative;
  flex-shrink: 0;

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }

  &:disabled {
    cursor: not-allowed;
    border-color: var(--border);
  }

  &.on {
    background: var(--bg-input);
    border-color: var(--border-strong);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    background: var(--text-faint);
    transition: background-color 120ms ease;
  }

  &.on .toggle-thumb {
    left: auto;
    right: 2px;
    background: var(--on);
  }

  &:disabled .toggle-thumb {
    background: var(--text-faint);
  }
}
</style>
