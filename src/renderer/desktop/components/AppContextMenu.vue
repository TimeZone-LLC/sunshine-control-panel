<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="menuRef"
      class="context-menu"
      :style="{ left: position.left + 'px', top: position.top + 'px' }"
      @click.stop
      @keydown.esc.prevent="$emit('close')"
    >
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-launch" @click="$emit('launch')">
        <span class="menu-icon"><VideoPlay /></span> {{ t.appContext.launch }}
      </button>
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-favorite" @click="$emit('toggleFavorite')">
        <span class="menu-icon">
          <component :is="isFavorited ? StarFilled : Star" />
        </span>
        {{ isFavorited ? t.appContext.unfavorite : t.appContext.favorite }}
      </button>
      <div class="menu-divider"></div>
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-cover" @click="$emit('updateCover')">
        <span class="menu-icon"><Picture /></span> {{ t.appContext.updateCover }}
      </button>
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-helpers" @click="$emit('configHelpers')">
        <span class="menu-icon"><Lightning /></span> {{ t.appContext.launchHelper }}
      </button>
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-copy" v-if="hasCmd" @click="$emit('copyCmd')">
        <span class="menu-icon"><DocumentCopy /></span> {{ t.appContext.copyCommand }}
      </button>
      <button type="button" class="menu-item" data-focusable data-focus-key="ctx-dir" v-if="hasWorkingDir" @click="$emit('openDir')">
        <span class="menu-icon"><Folder /></span> {{ t.appContext.openDirectory }}
      </button>
    </div>
  </Teleport>
</template>

<script setup>
import { nextTick, onUnmounted, ref, watch } from 'vue'
import { useI18n } from '../i18n/index.js'
import { popFocusScope, pushFocusScope } from '../composables/useFocusNav.js'
import { DocumentCopy, Folder, Lightning, Picture, Star, StarFilled, VideoPlay } from '@element-plus/icons-vue'
const { t } = useI18n()

const props = defineProps({
  visible: { type: Boolean, required: true },
  x: { type: Number, default: 0 },
  y: { type: Number, default: 0 },
  isFavorited: { type: Boolean, default: false },
  hasCmd: { type: Boolean, default: false },
  hasWorkingDir: { type: Boolean, default: false },
})

defineEmits(['launch', 'toggleFavorite', 'copyCmd', 'openDir', 'configHelpers', 'updateCover', 'close'])

const menuRef = ref(null)
const position = ref({ left: 0, top: 0 })
const VIEWPORT_MARGIN = 8

let disposeScope = null

/**
 * 用菜单自己的实测尺寸夹到视口内。之前调用方按固定 200px 估算，
 * 菜单项数量变化或大屏缩放后就会溢出屏幕。
 */
function place() {
  const menu = menuRef.value
  const width = menu?.offsetWidth || 180
  const height = menu?.offsetHeight || 200
  position.value = {
    left: Math.max(VIEWPORT_MARGIN, Math.min(props.x, window.innerWidth - width - VIEWPORT_MARGIN)),
    top: Math.max(VIEWPORT_MARGIN, Math.min(props.y, window.innerHeight - height - VIEWPORT_MARGIN)),
  }
}

watch(
  () => [props.visible, props.x, props.y],
  async ([visible]) => {
    if (!visible) {
      disposeScope?.()
      disposeScope = null
      return
    }
    // 先按请求坐标渲染，再测量并纠正，避免出现一帧的错位
    position.value = { left: props.x, top: props.y }
    await nextTick()
    place()
    if (menuRef.value && !disposeScope) disposeScope = pushFocusScope(menuRef.value)
  },
  { immediate: true }
)

onUnmounted(() => {
  disposeScope?.()
})
</script>

<style lang="less" scoped>
.context-menu {
  position: fixed;
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  padding: 2px 0;
  min-width: 180px;
  z-index: 10000;

  .menu-item {
    width: 100%;
    height: 24px;
    padding: 0 10px;
    font-size: var(--font-size);
    font-family: inherit;
    text-align: left;
    border: 0;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;

    .menu-icon {
      width: 14px;
      height: 14px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      color: var(--text-dim);

      :deep(svg) {
        width: 12px;
        height: 12px;
      }
    }

    &:hover,
    &:focus-visible {
      background: var(--bg-hover);
      color: var(--text);
    }
  }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: 2px 0;
  }
}
</style>
