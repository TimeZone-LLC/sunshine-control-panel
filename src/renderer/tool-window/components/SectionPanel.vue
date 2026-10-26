<template>
  <div class="section-panel">
    <div class="section-header">
      <span v-if="icon || $slots.icon" class="section-icon">
        <slot name="icon">{{ icon }}</slot>
      </span>
      <span class="section-title">{{ title }}</span>
      <div class="section-actions">
        <slot name="actions" />
      </div>
    </div>
    <div class="section-body">
      <slot />
    </div>
  </div>
</template>

<script setup>
defineProps({
  icon: { type: String, default: '' },
  title: { type: String, required: true },
})
</script>

<style lang="less" scoped>
.section-panel {
  background: var(--bg);
  border: 1px solid var(--border);
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: @spacing-sm;
  height: @row-height;
  padding: 0 @pad-x;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
}

.section-icon {
  font-size: @font-size-sm;
  color: var(--text-dim);
  flex-shrink: 0;
}

.section-title {
  font-size: @font-size-base;
  font-weight: @font-weight-medium;
  flex: 1;
  color: var(--text);
  letter-spacing: normal;
}

.section-actions {
  display: flex;
  align-items: center;
  gap: @spacing-sm;
  flex-shrink: 0;
}

.section-body {
  padding: @pad-y @pad-x;
  display: flex;
  flex-direction: column;
  gap: @spacing-sm;
}
</style>
