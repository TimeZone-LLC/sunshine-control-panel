<template>
  <div class="recent-section fade-in">
    <div class="section-label">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="section-icon">
        <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
      </svg>
      {{ t.apps.recentSection }}
    </div>
    <div class="recent-strip">
      <div
        v-for="app in apps"
        :key="'recent-' + app.name"
        class="recent-tile"
        tabindex="0"
        :data-app-name="app.name"
        :data-focus-key="'recent-' + app.name"
        @click="$emit('launch', app)"
        @keydown.enter="$emit('launch', app)"
        @contextmenu.prevent="$emit('contextmenu', $event, app)"
      >
        <div class="recent-cover">
          <img
            v-if="getAppImageUrl(app)"
            :src="getAppImageUrl(app)"
            :alt="app.name"
            loading="lazy"
            decoding="async"
            @error="handleImageError($event, app)"
          />
          <div v-else class="mini-placeholder">{{ app.name?.[0] || '?' }}</div>
        </div>
        <span class="recent-name">{{ app.name }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

defineProps({
  apps: { type: Array, required: true },
  getAppImageUrl: { type: Function, required: true },
  handleImageError: { type: Function, required: true },
})

defineEmits(['launch', 'contextmenu'])
</script>

<style lang="less" scoped>
.recent-section {
  margin-bottom: 12px;
}

.section-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--text-dim);
  margin-bottom: 6px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 6px;

  .section-icon {
    width: 12px;
    height: 12px;
    color: var(--text-faint);
  }
}

.recent-strip {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding-bottom: 2px;

  &::-webkit-scrollbar { height: 6px; }
  &::-webkit-scrollbar-thumb { background: var(--border); }
}

.recent-tile {
  content-visibility: auto;
  contain-intrinsic-size: auto 80px;
  flex-shrink: 0;
  width: 56px;
  cursor: pointer;

  &:hover .recent-name,
  &:focus-visible .recent-name {
    color: var(--text);
  }

  .recent-cover {
    width: 56px;
    height: 56px;
    overflow: hidden;
    background: var(--bg-input);
    border: 1px solid var(--border);
    margin-bottom: 3px;

    img { width: 100%; height: 100%; object-fit: cover; }
  }

  &:hover .recent-cover {
    border-color: var(--border-strong);
  }

  .recent-name {
    font-size: var(--font-size-xs);
    color: var(--text-dim);
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.mini-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: var(--font-size);
  color: var(--text-faint);
  text-transform: uppercase;
}
</style>
