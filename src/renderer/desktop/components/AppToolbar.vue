<template>
  <div class="apps-toolbar fade-in">
    <h1 class="apps-title">{{ t.apps.libraryTitle }}</h1>

    <!-- Filter tags -->
    <div class="filter-tabs">
      <button
        v-for="tab in filterTabs"
        :key="tab.id"
        class="filter-tab"
        :class="{ active: activeFilter === tab.id }"
        tabindex="0"
        :data-focus-key="'filter-' + tab.id"
        @click="$emit('update:activeFilter', tab.id)"
      >
        {{ tab.label }}
        <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
      </button>
    </div>

    <div class="toolbar-spacer"></div>

    <!-- Search -->
    <div class="apps-search" :class="{ focused: searchFocused }">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        :value="searchQuery"
        type="text"
        class="search-input"
        data-focusable
        data-focus-key="apps-search"
        :aria-label="t.apps.searchPlaceholder"
        :placeholder="t.apps.searchPlaceholder"
        @input="$emit('update:searchQuery', $event.target.value)"
        @focus="searchFocused = true"
        @blur="searchFocused = false"
      />
    </div>

    <!-- Sort -->
    <button class="toolbar-btn" tabindex="0" @click="$emit('cycleSortMode')" :title="`${t.apps.sortTitle}: ${sortLabel}`">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 6h18M3 12h12M3 18h6"/>
      </svg>
    </button>

    <!-- Grid size -->
    <button class="toolbar-btn" tabindex="0" @click="$emit('cycleGridSize')" :title="`${t.apps.cardSizeTitle}: ${t.apps.gridSizes?.[gridSize] || gridSize}`">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="7" height="7"/>
        <rect x="14" y="3" width="7" height="7"/>
        <rect x="3" y="14" width="7" height="7"/>
        <rect x="14" y="14" width="7" height="7"/>
      </svg>
    </button>

    <!-- View switch -->
    <button class="toolbar-btn" tabindex="0" @click="$emit('toggleViewMode')" :title="viewMode === 'grid' ? t.apps.listView : t.apps.gridView">
      <svg v-if="viewMode === 'grid'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
      </svg>
    </button>

    <div class="apps-count">{{ t.apps.count.replace('{count}', totalCount) }}</div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from '../i18n/index.js'

const { t } = useI18n()

defineProps({
  filterTabs: { type: Array, required: true },
  activeFilter: { type: String, required: true },
  searchQuery: { type: String, required: true },
  sortLabel: { type: String, required: true },
  gridSize: { type: String, required: true },
  viewMode: { type: String, required: true },
  totalCount: { type: Number, required: true },
})

defineEmits([
  'update:activeFilter',
  'update:searchQuery',
  'cycleSortMode',
  'cycleGridSize',
  'toggleViewMode',
])

const searchFocused = ref(false)
</script>

<style lang="less" scoped>
.apps-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}

.apps-title {
  font-size: var(--font-size);
  font-weight: 600;
  color: var(--text);
  margin: 0;
  white-space: nowrap;
}

.filter-tabs {
  display: flex;
  border: 1px solid var(--border);
}

.filter-tab {
  height: 22px;
  padding: 0 8px;
  border: none;
  background: transparent;
  color: var(--text-dim);
  font-size: var(--font-size-sm);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 5px;

  & + .filter-tab {
    border-left: 1px solid var(--border);
  }

  .tab-count {
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    color: var(--text-faint);
  }

  &:hover:not(.active) {
    background: var(--bg-hover);
    color: var(--text);
  }

  &.active {
    background: var(--bg-active);
    color: var(--text);

    .tab-count {
      color: var(--text-dim);
    }
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: -1px;
  }
}

.toolbar-spacer {
  flex: 1;
}

.apps-search {
  position: relative;
  width: 220px;
  flex-shrink: 0;

  .search-icon {
    position: absolute;
    left: 6px;
    top: 50%;
    margin-top: -6px;
    width: 12px;
    height: 12px;
    color: var(--text-faint);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    height: 22px;
    padding: 0 8px 0 22px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    color: var(--text);
    font-size: var(--font-size-sm);
    outline: none;

    &::placeholder { color: var(--text-faint); }

    &:focus {
      border-color: var(--border-strong);
    }
  }
}

.toolbar-btn {
  width: 22px;
  height: 22px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;

  svg { width: 12px; height: 12px; }

  &:hover {
    background: var(--bg-hover);
    color: var(--text);
    border-color: var(--border-strong);
  }

  &:focus-visible {
    outline: 1px solid var(--border-strong);
    outline-offset: 0;
  }
}

.apps-count {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  color: var(--text-dim);
  white-space: nowrap;
}
</style>
