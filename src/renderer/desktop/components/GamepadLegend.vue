<template>
  <span class="gamepad-legend" aria-hidden="true">
    <span v-for="hint in hints" :key="hint.id" class="legend-item">
      <span class="legend-button" :class="'btn-' + hint.tone">{{ hint.glyph }}</span>
      <span class="legend-label">{{ hint.label }}</span>
    </span>
  </span>
</template>

<script setup>
import { computed } from 'vue'
import { viewActions } from '../composables/useGamepadActions.js'
import { bigScreenSettings } from '../composables/useBigScreenSettings.js'
import { chipFor } from '../composables/useGamepadLayout.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
  cursorMode: { type: Boolean, default: false },
})

/** 动作 id → 文案。按键符号（A/B/X/Y 还是 ✕○□△）由 useGamepadLayout 按手柄布局给出。 */
const LABELS = () => {
  const legend = t.value.gamepadLegend
  return {
    click: legend.click,
    exitCursor: legend.exitCursor,
    confirm: legend.confirm,
    back: legend.back,
    search: legend.search,
    favorite: legend.favorite,
    menu: legend.menu,
    scroll: legend.scroll,
    pages: legend.switchPage,
    cursor: legend.cursorOn,
  }
}

/**
 * 只显示当前上下文真的会响应的按键：
 * 全局动作（确认/返回/换页/滚动/光标）始终在，视图级动作（搜索/收藏/菜单）
 * 由各视图通过 useGamepadActions 声明——没声明的页面就不显示，不撒谎。
 * 按键符号跟随当前活跃手柄的布局，换手柄即时切换。
 */
const hints = computed(() => {
  const labels = LABELS()
  if (props.cursorMode) {
    return ['confirm', 'menu', 'back', 'scroll'].map((id) => {
      const { glyph, tone } = chipFor(id)
      const labelId = id === 'confirm' ? 'click' : id === 'back' ? 'exitCursor' : id
      return { id, glyph, tone, label: labels[labelId] }
    })
  }

  const actions = ['confirm', 'back']
  if (viewActions.value.has('search')) actions.push('search')
  if (viewActions.value.has('favorite')) actions.push('favorite')
  if (viewActions.value.has('menu')) actions.push('menu')
  actions.push('scroll', 'pages')
  // 设置里关掉光标模式时 L3 不会响应，提示条不能显示无效按键
  if (bigScreenSettings.value.gamepadCursorEnabled) actions.push('cursor')

  return actions.map((id) => {
    const { glyph, tone } = chipFor(id)
    return { id, glyph, tone, label: labels[id] }
  })
})
</script>

<style lang="less" scoped>
.gamepad-legend {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
  justify-content: center;
  padding: var(--space-1) var(--space-3);
  background: var(--bg-panel);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-xs);
  pointer-events: none;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.legend-button {
  min-width: 20px;
  height: 20px;
  padding: 0 var(--space-1);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 700;
  background: var(--bg-active);
  border: 1px solid var(--border-strong);
  color: var(--text);
}

.legend-label {
  font-size: var(--font-size-xs);
  color: var(--text-dim);
  white-space: nowrap;
}
</style>
