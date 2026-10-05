<template>
  <div class="apps-list">
    <div
      v-for="(app, index) in apps"
      :key="'list-' + app.name"
      class="app-list-item"
      :class="{ launching: launchingApp === app.name }"
      tabindex="0"
      :data-app-name="app.name"
      :data-focus-key="'app-' + app.name"
      @click="$emit('launch', app)"
      @keydown.enter="$emit('launch', app)"
      @contextmenu.prevent="$emit('contextmenu', $event, app)"
    >
      <div class="list-cover">
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
      <div class="list-info">
        <span class="list-name">{{ app.name }}</span>
        <span class="list-cmd" v-if="app.cmd">{{ app.cmd }}</span>
      </div>
      <div class="list-tags">
        <span v-if="playtimeLabel(app.name)" class="list-tag playtime">{{ playtimeLabel(app.name) }}</span>
        <span v-if="isFavorite(app.name)" class="list-tag fav"><StarFilled /> {{ t.appContext.favorite }}</span>
        <span v-if="app.elevated && app.elevated !== 'false'" class="list-tag admin">{{ t.appContext.admin }}</span>
      </div>
      <button class="list-play" tabindex="-1"><VideoPlay /></button>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from '../i18n/index.js'
import { StarFilled, VideoPlay } from '@element-plus/icons-vue'
import { formatDuration } from '../composables/useGameSession.js'
const { t } = useI18n()

const props = defineProps({
  apps: { type: Array, required: true },
  launchingApp: { type: String, default: null },
  isFavorite: { type: Function, required: true },
  getAppImageUrl: { type: Function, required: true },
  handleImageError: { type: Function, required: true },
  stats: { type: Object, default: () => ({}) },
})

defineEmits(['launch', 'contextmenu'])

function playtimeLabel(appName) {
  const seconds = Number(props.stats?.[appName]?.totalSeconds) || 0
  if (seconds < 60) return ''
  return t.value.gameSession.playedFor.replace('{duration}', formatDuration(seconds))
}
</script>

<style lang="less" scoped>
.apps-list {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
}

.app-list-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  min-height: var(--row-height);
  cursor: pointer;
  border-bottom: 1px solid var(--border);
  content-visibility: auto;
  contain-intrinsic-size: auto 32px;

  &:last-child {
    border-bottom: 0;
  }

  &:hover,
  &:focus-visible {
    background: var(--bg-hover);

    .list-name { color: var(--text); }
  }

  &.launching {
    background: var(--bg-active);
  }

  .list-cover {
    width: 20px;
    height: 20px;
    overflow: hidden;
    background: var(--bg-input);
    border: 1px solid var(--border);
    flex-shrink: 0;

    img { width: 100%; height: 100%; object-fit: cover; }
  }

  .list-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 10px;

    .list-name {
      font-size: var(--font-size);
      color: var(--text);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex-shrink: 0;
      max-width: 40%;
    }

    .list-cmd {
      font-family: var(--font-mono);
      font-size: var(--font-size-xs);
      color: var(--text-faint);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
      min-width: 0;
    }
  }

  .list-tags {
    display: flex;
    gap: 4px;
  }

  .list-tag {
    font-size: var(--font-size-xs);
    line-height: 14px;
    padding: 0 4px;
    border: 1px solid var(--border);
    color: var(--text-dim);
    display: inline-flex;
    align-items: center;
    gap: 3px;

    svg {
      width: 9px;
      height: 9px;
    }

    &.fav,
    &.admin {
      border-color: var(--border-strong);
      color: var(--text);
    }
  }

  .list-play {
    width: 20px;
    height: 20px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;

    svg {
      width: 10px;
      height: 10px;
    }
  }

  &:hover .list-play {
    background: var(--bg-active);
    color: var(--text);
    border-color: var(--border-strong);
  }
}

.mini-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-faint);
}
</style>
