<template>
  <Teleport to="body">
    <Transition name="launch-overlay">
      <div v-if="state" ref="overlayRef" class="launch-overlay" :class="'status-' + state.status">
        <div class="launch-card">
          <div class="launch-cover">
            <img v-if="state.coverUrl" :src="state.coverUrl" :alt="state.appName" />
            <span v-else class="launch-cover-letter">{{ state.appName?.[0] || '?' }}</span>
            <div v-if="state.status === 'launching'" class="launch-cover-scan"></div>
          </div>

          <div class="launch-body">
            <div class="launch-status">
              <span v-if="state.status === 'launching'" class="launch-spinner"></span>
              <span v-else class="launch-glyph" :class="'glyph-' + state.status">{{ glyph }}</span>
              <span class="launch-headline">{{ headline }}</span>
            </div>

            <p class="launch-detail">{{ detail }}</p>

            <div class="launch-actions">
              <button
                v-if="state.status === 'conflict'"
                type="button"
                class="launch-btn danger"
                data-focusable
                data-focus-key="launch-stop"
                @click="emit('stop')"
              >
                {{ t.gameSession.stopRunning }}
              </button>
              <button
                v-if="state.status !== 'launching'"
                type="button"
                class="launch-btn"
                data-focusable
                data-focus-key="launch-dismiss"
                @click="emit('dismiss')"
              >
                {{ t.gameSession.dismiss }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { formatDuration } from '../composables/useGameSession.js'
import { focusElement, useModalFocusScope } from '../composables/useFocusNav.js'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

const props = defineProps({
  state: { type: Object, default: null },
})

const emit = defineEmits(['dismiss', 'stop'])

// 这是一层吞点击的全屏遮罩，必须和其他弹层一样压入焦点栈：否则手柄导航会继续在
// 遮罩背后的应用网格里走，而 conflict 态自己的「结束游戏 / 关闭」按钮反而够不到
// —— teleport 出去的节点不在 .desktop-window 里，收集不到。
const overlayRef = ref(null)
useModalFocusScope(overlayRef, () => !!props.state)

// 状态在遮罩存续期间会变（launching -> error / conflict / untracked），而
// useModalFocusScope 只在开合时压栈，不会为中途才出现的按钮重新聚焦。所以这里
// 按状态自己收尾：
//   有按钮而焦点在遮罩外 -> 聚焦第一个。否则焦点还停在 launching 期间被 blur 掉的
//   body 上，第一次按 A 是空的（confirmFocused 对 body 直接返回 false）。
//   没有按钮（launching）-> 把焦点从遮罩背后的卡片上移开，否则 A 会点到被完全挡住
//   的应用卡片。关闭时 popFocusScope 会把焦点还回去。
watch(
  () => props.state?.status,
  async () => {
    await nextTick()
    const root = overlayRef.value
    if (!root) return
    const focusable = root.querySelector('[data-focusable]')
    if (focusable) {
      if (!root.contains(document.activeElement)) focusElement(focusable)
      return
    }
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur()
  },
  { immediate: true }
)

const glyph = computed(() => {
  switch (props.state?.status) {
    case 'started':
      return '✓'
    case 'exited':
      return '⏻'
    case 'untracked':
    case 'conflict':
      return '!'
    default:
      return '×'
  }
})

function text(key, replacements = {}) {
  let value = t.value.gameSession?.[key] || ''
  for (const [name, replacement] of Object.entries(replacements)) {
    value = value.replace(`{${name}}`, replacement)
  }
  return value
}

const headline = computed(() => {
  const name = props.state?.appName || ''
  switch (props.state?.status) {
    case 'launching':
      return text('launching', { name })
    case 'started':
      return text('started', { name })
    case 'untracked':
      return text('untracked', { name })
    case 'conflict':
      return text('conflict', { name: props.state.message || '' })
    case 'exited':
      return text('exited', { name })
    default:
      return text('launchFailed', { name })
  }
})

const detail = computed(() => {
  switch (props.state?.status) {
    case 'launching':
      return text('launchingHint')
    case 'started':
      return text('startedHint')
    case 'untracked':
      return props.state.message === 'detached-only'
        ? text('untrackedDetached')
        : text('untrackedNoHandle')
    case 'conflict':
      return text('conflictHint')
    case 'exited':
      return text('exitedHint', { duration: formatDuration(props.state.seconds) })
    default:
      return props.state?.message || ''
  }
})
</script>

<style lang="less" scoped>
.launch-overlay {
  position: fixed;
  inset: 0;
  z-index: 19000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--scrim);
}

.launch-card {
  display: flex;
  align-items: center;
  gap: var(--space-5);
  padding: var(--space-5);
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-md);
  max-width: min(760px, 88vw);
}

.launch-cover {
  position: relative;
  width: 132px;
  height: 176px;
  flex-shrink: 0;
  overflow: hidden;
  background: var(--bg-input);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .launch-cover-letter {
    font-family: var(--font-mono);
    font-size: 48px;
    font-weight: 600;
    color: var(--text-faint);
  }

  .launch-cover-scan {
    display: none;
  }
}

.launch-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.launch-status {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.launch-headline {
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
}

.launch-spinner {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  border: 3px solid var(--border);
  border-top-color: var(--text);
  border-radius: 50%;
  animation: launch-spin 0.8s linear infinite;
}

@keyframes launch-spin {
  to {
    transform: rotate(360deg);
  }
}

.launch-glyph {
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  border: 1px solid var(--border-strong);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-mono);
  font-weight: 700;
  color: var(--text);

  &.glyph-untracked,
  &.glyph-conflict {
    border-color: var(--warn);
    color: var(--warn);
  }

  &.glyph-error {
    border-color: var(--error);
    color: var(--error);
  }
}

.launch-detail {
  margin: 0;
  font-size: var(--font-size);
  line-height: var(--line-height);
  color: var(--text-dim);
  white-space: pre-line;
  word-break: break-word;
}

.launch-actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-1);
}

.launch-btn {
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
