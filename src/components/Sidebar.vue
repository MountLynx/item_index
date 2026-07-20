<template>
  <aside class="sidebar">
    <div class="search-bar">
      <TablerIcon name="search" :size="14" class="search-icon" />
      <input
        ref="searchInput"
        v-model="searchQuery"
        :placeholder="$t('sidebar.search')"
        class="search-input"
        @input="onSearchInput"
      />
    </div>

    <!-- Search results (when query is active) -->
    <div v-if="searchQuery.trim()" class="search-results">
      <div class="sec-hd">
        <TablerIcon name="search" :size="14" />
        <span>{{ $t('sidebar.searchResults') }}</span>
        <span class="result-count">{{ searchResults.length }}</span>
      </div>
      <div v-if="searchLoading" class="search-loading">...</div>
      <div v-else-if="searchResults.length === 0" class="search-empty">
        {{ $t('sidebar.noResults') }}
      </div>
      <div
        v-for="item in searchResults"
        :key="item.id"
        class="search-item"
        :class="{ active: item.id === itemStore.selectedId }"
        @click="onSelectResult(item.id)"
      >
        <span class="search-item-type">{{ getTypeIcon(item.type_id) }}</span>
        <span class="search-item-name">{{ item.name }}</span>
      </div>
    </div>

    <!-- Normal sidebar (when no search) -->
    <template v-else>
      <div class="sec">
        <div class="sec-hd" @click="clearGroup">
          <TablerIcon name="folder" :size="14" />
          <span>{{ $t('sidebar.groups') }}</span>
          <span v-if="gid !== null" class="clear-hint">{{ $t('sidebar.showAll') }}</span>
        </div>
        <GroupTree :selected-id="gid" @select="onGroupSelect" />
      </div>
      <div class="sep" />
      <div class="sec">
        <div class="sec-hd" @click="clearTag">
          <TablerIcon name="tag" :size="14" />
          <span>{{ $t('sidebar.tags') }}</span>
          <span v-if="tid !== null" class="clear-hint">{{ $t('sidebar.showAll') }}</span>
        </div>
        <TagList :selected-id="tid" @select="onTagSelect" />
      </div>
    </template>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import GroupTree from './GroupTree.vue'
import TagList from './TagList.vue'
import TablerIcon from './TablerIcon.vue'
import type { Item, FilterNode } from '@/types/bindings'

const itemStore = useItemStore()
const typeStore = useTypeStore()
const gid = ref<number | null>(null)
const tid = ref<number | null>(null)

// Search state
const searchQuery = ref('')
const searchResults = ref<Item[]>([])
const searchLoading = ref(false)
const searchInput = ref<HTMLInputElement | null>(null)
let searchTimer: ReturnType<typeof setTimeout> | null = null

async function onGroupSelect(id: number | null) { gid.value = id; await itemStore.fetchList(gid.value, tid.value) }
async function onTagSelect(id: number | null) { tid.value = id; await itemStore.fetchList(gid.value, tid.value) }
async function clearGroup() { gid.value = null; await itemStore.fetchList(null, tid.value) }
async function clearTag() { tid.value = null; await itemStore.fetchList(gid.value, null) }

function getTypeIcon(typeId: number): string {
  const t = typeStore.types.find(tp => tp.id === typeId)
  return t?.icon ?? '📄'
}

function onSelectResult(id: string) {
  itemStore.select(id)
}

async function onSearchInput() {
  const q = searchQuery.value.trim()
  if (!q) {
    searchResults.value = []
    return
  }
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(async () => {
    searchLoading.value = true
    try {
      const filter: FilterNode = { field: 'name', op: 'contains', value: q }
      const result = await itemStore.query({ filter, limit: 50 })
      searchResults.value = result.items
    } catch {
      searchResults.value = []
    } finally {
      searchLoading.value = false
    }
  }, 200)
}

// Clear search when repo changes
watch(() => itemStore.items, () => {
  if (searchQuery.value) {
    onSearchInput()
  }
})
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-w); flex-shrink: 0; background: var(--surface);
  border-right: 1px solid var(--border); display: flex; flex-direction: column;
  overflow-y: auto;
}

/* Search bar */
.search-bar {
  display: flex; align-items: center; gap: 6px;
  padding: 8px 12px; border-bottom: 1px solid var(--border);
  position: sticky; top: 0; background: var(--surface); z-index: 1;
}
.search-icon { color: var(--text-muted); flex-shrink: 0; }
.search-input {
  flex: 1; border: none; background: transparent; font-size: var(--fs-sm);
  color: var(--text); outline: none; padding: 2px 0;
}
.search-input::placeholder { color: var(--text-muted); }

/* Search results */
.search-results { padding: 8px; }
.search-loading, .search-empty {
  font-size: var(--fs-xs); color: var(--text-muted); padding: 12px 8px;
  text-align: center;
}
.result-count { font-weight: var(--fw-normal); font-size: 10px; color: var(--text-muted); margin-left: auto; }
.search-item {
  display: flex; align-items: center; gap: 6px; padding: 4px 8px;
  border-radius: var(--r-sm); cursor: pointer; font-size: var(--fs-sm);
  transition: background var(--fast) var(--ease);
}
.search-item:hover { background: var(--surface-hover); }
.search-item.active { background: var(--accent); color: var(--accent-fg); }
.search-item-type { font-size: 14px; flex-shrink: 0; }
.search-item-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* Existing sidebar styles (unchanged) */
.sec { padding: 12px 8px; }
.sec-hd {
  display: flex; align-items: center; gap: 6px; padding: 4px 8px; margin-bottom: 4px;
  font-size: var(--fs-xs); font-weight: var(--fw-semibold); color: var(--text-muted);
  text-transform: uppercase; letter-spacing: 0.05em; cursor: pointer;
  border-radius: var(--r-sm); transition: background var(--fast) var(--ease);
}
.sec-hd:hover { background: var(--surface-hover); }
.clear-hint { font-weight: var(--fw-normal); text-transform: none; color: var(--accent); font-size: 10px; margin-left: auto; }
.sep { height: 1px; background: var(--border); margin: 0 12px; }
</style>
