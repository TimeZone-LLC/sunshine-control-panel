<template>
  <div class="apps-grid" :class="'grid-' + gridSize">
    <div
      v-for="app in apps"
      :key="app.name"
      class="app-tile fade-in"
      :class="{ launching: launchingApp === app.name, favorited: isFavorite(app.name) }"
      tabindex="0"
      :data-app-name="app.name"
      :data-focus-key="'app-' + app.name"
      @click="$emit('launch', app)"
      @keydown.enter="$emit('launch', app)"
      @contextmenu.prevent="$emit('contextmenu', $event, app)"
    >
      <div class="tile-cover">
        <img
          v-if="getAppImageUrl(app)"
          :src="getAppImageUrl(app)"
          :alt="app.name"
          class="cover-image"
          loading="lazy"
          decoding="async"
          @error="handleImageError($event, app)"
        />
        <div v-else class="cover-placeholder">
          <span class="placeholder-letter">{{ app.name?.[0] || '?' }}</span>
        </div>
        <!-- Favourite badge -->
        <div v-if="isFavorite(app.name)" class="favorite-badge" @click.stop="$emit('toggleFavorite', app.name)">
          <StarFilled />
        </div>
        <!-- Launch helper badge -->
        <div v-if="helperIds(app.name).length" class="helper-badges">
          <LaunchHelperIcon
            v-for="helperId in helperIds(app.name)"
            :key="helperId"
            :template-id="helperId"
            :size="12"
            class="helper-badge-icon"
          />
        </div>
        <!-- Launch indicator -->
        <div v-if="launchingApp === app.name" class="launch-overlay">
          <div class="launch-spinner"></div>
          <span>{{ t.appContext.launching }}</span>
        </div>
      </div>
      <div class="tile-info">
        <span class="tile-name" :title="app.name">{{ app.name }}</span>
        <span v-if="playtimeLabel(app.name)" class="tile-playtime">{{ playtimeLabel(app.name) }}</span>
        <span v-if="app.elevated && app.elevated !== 'false'" class="tile-badge admin">{{ t.appContext.admin }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from '../i18n/index.js'
import { StarFilled } from '@element-plus/icons-vue'
import LaunchHelperIcon from './LaunchHelperIcon.vue'
import { formatDuration } from '../composables/useGameSession.js'
const { t } = useI18n()

const props = defineProps({
  apps: { type: Array, required: true },
  gridSize: { type: String, required: true },
  launchingApp: { type: String, default: null },
  isFavorite: { type: Function, required: true },
  getAppImageUrl: { type: Function, required: true },
  handleImageError: { type: Function, required: true },
  helperIds: { type: Function, default: () => [] },
  stats: { type: Object, default: () => ({}) },
})

defineEmits(['launch', 'contextmenu', 'toggleFavorite'])

/** 累计时长不足一分钟就不显示，避免失败的启动留下 "0m" 噪声。 */
function playtimeLabel(appName) {
  const seconds = Number(props.stats?.[appName]?.totalSeconds) || 0
  if (seconds < 60) return ''
  return t.value.gameSession.playedFor.replace('{duration}', formatDuration(seconds))
}
</script>

<style lang="less" scoped>
@import "@/components/spinner.less";

.apps-grid {
  display: grid;
  gap: 8px;

  &.grid-small  { grid-template-columns: repeat(auto-fill, minmax(110px, 1fr)); }
  &.grid-medium { grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); }
  &.grid-large  { grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); }
}

.app-tile {
  cursor: pointer;
  position: relative;
  content-visibility: auto;
  contain-intrinsic-size: auto 240px;

  &:hover,
  &:focus-visible {
    .tile-cover { border-color: var(--border-strong); }
    .tile-name { color: var(--text); }
  }

  &.launching {
    pointer-events: none;

    .tile-cover { border-color: var(--border-strong); }
  }
}

.tile-cover {
  position: relative;
  aspect-ratio: 3 / 4;
  overflow: hidden;
  background: var(--bg-input);
  border: 1px solid var(--border);

  .cover-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-input);

    .placeholder-letter {
      font-family: var(--font-mono);
      font-size: 28px;
      font-weight: 600;
      color: var(--text-faint);
    }
  }
}

// A favourite is a filled marker, not a gold star colour.
.favorite-badge {
  position: absolute;
  top: 4px;
  right: 4px;
  color: var(--text);
  background: var(--bg);
  border: 1px solid var(--border-strong);
  z-index: 3;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1px;

  svg {
    width: 10px;
    height: 10px;
  }
}

.helper-badges {
  position: absolute;
  bottom: 4px;
  left: 4px;
  display: flex;
  gap: 2px;
  z-index: 3;

  .helper-badge-icon {
    width: 14px;
    height: 14px;
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--text-dim);
    padding: 1px;
  }
}

.launch-overlay {
  position: absolute;
  inset: 0;
  background: var(--scrim);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  z-index: 4;
  color: var(--text);
  font-size: var(--font-size-sm);

  .launch-spinner {
    .gloss-spinner();
  }
}

.tile-info {
  padding: 4px 0 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.tile-name {
  font-size: var(--font-size-sm);
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.tile-playtime {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-faint);
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.tile-badge {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  line-height: 14px;
  padding: 0 4px;
  border: 1px solid var(--border);
  color: var(--text-dim);
  flex-shrink: 0;

  &.admin {
    border-color: var(--border-strong);
    color: var(--text);
  }
}
</style>
