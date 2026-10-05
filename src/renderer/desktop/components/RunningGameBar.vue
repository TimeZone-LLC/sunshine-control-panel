<template>
  <Transition name="running-bar">
    <div v-if="game" class="running-bar">
      <span class="running-pulse"></span>

      <div class="running-info">
        <span class="running-label">{{ t.gameSession.nowPlaying }}</span>
        <span class="running-name" :title="game.appName">{{ game.appName }}</span>
      </div>

      <div class="running-meta">
        <span class="running-elapsed">{{ formatDuration(elapsed) }}</span>
        <span v-if="totalLabel" class="running-total">{{ totalLabel }}</span>
        <span v-if="game.adopted" class="running-adopted" :title="t.gameSession.adoptedHint">
          {{ t.gameSession.adopted }}
        </span>
      </div>

      <div class="running-actions">
        <button
          type="button"
          class="running-btn"
          data-focusable
          data-focus-key="running-resume"
          @click="emit('resume')"
        >
          {{ t.gameSession.backToGame }}
        </button>
        <button
          type="button"
          class="running-btn danger"
          data-focusable
          data-focus-key="running-stop"
          @click="emit('stop')"
        >
          {{ t.gameSession.stopRunning }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed } from 'vue'
import { formatDuration } from '../composables/useGameSession.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
  game: { type: Object, default: null },
  elapsed: { type: Number, default: 0 },
  stat: { type: Object, default: null },
})

const emit = defineEmits(['resume', 'stop'])

const totalLabel = computed(() => {
  const total = Number(props.stat?.totalSeconds) || 0
  // 本局时长还没结算进统计，所以这里加上去，显示的才是真实累计
  const combined = total + (Number(props.elapsed) || 0)
  if (combined < 60) return ''
  return t.value.gameSession.totalPlaytime.replace('{duration}', formatDuration(combined))
})
</script>

<style lang="less" scoped>
.running-bar {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  max-width: 100%;
  padding: var(--space-2) var(--space-3);
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-xs);
}

.running-pulse {
  width: 9px;
  height: 9px;
  flex-shrink: 0;
  background: var(--on);
}

.running-info {
  display: flex;
  flex-direction: column;
  min-width: 0;

  .running-label {
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }

  .running-name {
    font-size: var(--font-size);
    font-weight: 600;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 320px;
  }
}

.running-meta {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;

  .running-elapsed {
    font-family: var(--font-mono);
    font-size: 16px;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .running-total,
  .running-adopted {
    font-size: var(--font-size-xs);
    color: var(--text-dim);
  }

  .running-adopted {
    padding: 1px var(--space-1);
    border: 1px solid var(--border);
    cursor: help;
  }
}

.running-actions {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
}

.running-btn {
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

  &.danger {
    border-color: var(--error);
    color: var(--error);
  }
}
</style>
