<template>
  <div class="icon-picker" ref="rootEl">
    <button class="picker-btn" @click="toggle" type="button">
      <TablerIcon :name="modelValue || 'circle'" :size="18" />
      <TablerIcon name="chevron-right" :size="12" class="arr" :class="{ open }" />
    </button>

    <Teleport to="body">
      <div v-if="open" class="pick-drop" :style="dropStyle" @click.stop>
        <input
          ref="searchEl"
          id="icon-picker-search"
          name="icon-search"
          v-model="search"
          class="pick-search"
          placeholder="搜索图标…"
          @keydown.escape="open = false"
        />

        <div v-if="!search && groups.length" class="pick-tabs">
          <button
            v-for="g in groups"
            :key="g.prefix"
            class="tab"
            :class="{ sel: activeGroup === g.prefix }"
            @click="activeGroup = g.prefix"
          >{{ g.label }}</button>
        </div>

        <div class="pick-grid" ref="gridEl" @scroll="onScroll">
          <div :style="{ height: totalHeight + 'px', position: 'relative' }">
            <div :style="{ transform: `translateY(${offsetY}px)` }">
              <div v-for="(row, ri) in visibleRows" :key="ri" class="pick-row">
                <button
                  v-for="name in row"
                  :key="name"
                  class="pick-item"
                  :class="{ sel: modelValue === name }"
                  :title="name"
                  @click="select(name)"
                >
                  <TablerIcon :name="name" :size="18" />
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="search && !filtered.length" class="pick-empty">
          无匹配，可输入 emoji 直接使用
        </div>
        <div v-if="!search && !groups.length" class="pick-empty">
          加载中…
        </div>
      </div>
    </Teleport>
  </div>

  <div v-if="open" class="pick-backdrop" @click="open = false" />
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import type { IconGroup } from '@/assets/icon-names'
import TablerIcon from './TablerIcon.vue'

defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()

const open = ref(false)
const search = ref('')
const rootEl = ref<HTMLElement | null>(null)
const searchEl = ref<HTMLInputElement | null>(null)
const gridEl = ref<HTMLElement | null>(null)
const dropStyle = ref<Record<string, string>>({})

const groups = ref<IconGroup[]>([])
const activeGroup = ref('_base')
const scrollTop = ref(0)
const loadErr = ref('')

const COLS = 6
const ROW_H = 32
const VISIBLE_ROWS = 9
const BUFFER = 3

async function loadIcons() {
  if (groups.value.length) return
  try {
    const mod = await import('@/assets/icon-names')
    groups.value = mod.ICON_GROUPS
  } catch (e) {
    loadErr.value = String(e)
    console.error('Failed to load icon names:', e)
  }
}

const currentIcons = computed(() => {
  if (!groups.value.length) return []
  const g = groups.value.find(g => g.prefix === activeGroup.value)
  return g ? g.icons : groups.value[0]?.icons ?? []
})

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return [] as string[]
  const all: string[] = []
  for (const g of groups.value) {
    for (const n of g.icons) {
      if (n.includes(q)) all.push(n)
      if (all.length >= 200) break
    }
    if (all.length >= 200) break
  }
  return all
})

const visibleRows = computed(() => {
  const list: string[] = search.value.trim() ? filtered.value : currentIcons.value
  if (!list.length) return []
  const total = Math.min(list.length, COLS * (VISIBLE_ROWS + BUFFER * 2))
  if (list.length <= total) {
    const rows: string[][] = []
    for (let i = 0; i < list.length; i += COLS) rows.push(list.slice(i, i + COLS))
    return rows
  }
  const startRow = Math.max(0, Math.floor(scrollTop.value / ROW_H) - BUFFER)
  const endRow = startRow + VISIBLE_ROWS + BUFFER * 2
  const rows: string[][] = []
  for (let r = startRow; r <= endRow; r++) {
    const start = r * COLS
    if (start >= list.length) break
    rows.push(list.slice(start, start + COLS))
  }
  return rows
})

const totalHeight = computed(() => {
  const list: string[] = search.value.trim() ? filtered.value : currentIcons.value
  return Math.max(1, Math.ceil(list.length / COLS)) * ROW_H
})

const offsetY = computed(() => {
  const list: string[] = search.value.trim() ? filtered.value : currentIcons.value
  if (list.length <= COLS * (VISIBLE_ROWS + BUFFER * 2)) return 0
  const startRow = Math.max(0, Math.floor(scrollTop.value / ROW_H) - BUFFER)
  return startRow * ROW_H
})

function onScroll() {
  if (gridEl.value) scrollTop.value = gridEl.value.scrollTop
}

async function toggle() {
  open.value = !open.value
  if (open.value) {
    await loadIcons()
    await nextTick()
    position()
    searchEl.value?.focus()
  }
}

function select(name: string) {
  emit('update:modelValue', name)
  open.value = false
  search.value = ''
}

function position() {
  if (!rootEl.value) return
  const r = rootEl.value.getBoundingClientRect()
  dropStyle.value = {
    top: Math.min(r.bottom + 4, window.innerHeight - 320) + 'px',
    left: Math.min(r.left, window.innerWidth - 260) + 'px',
  }
}

watch(open, (v) => {
  if (!v) { search.value = ''; scrollTop.value = 0 }
})

watch([activeGroup, search], () => { scrollTop.value = 0 })
</script>

<style scoped>
.icon-picker { display: inline-flex; flex-shrink: 0; }
.picker-btn {
  display: flex; align-items: center; gap: 2px;
  padding: 0 6px; height: 32px;
  border: 1px solid var(--border); border-radius: var(--r-md);
  background: var(--surface); cursor: pointer;
  transition: border-color var(--fast) var(--ease);
}
.picker-btn:hover { border-color: var(--accent); }
.arr { transition: transform var(--fast) var(--ease); color: var(--text-muted); }
.arr.open { transform: rotate(90deg); }

.pick-backdrop { position: fixed; inset: 0; z-index: 300; background: transparent; }
.pick-drop {
  position: fixed; z-index: 301;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-lg);
  width: 248px;
  max-height: 320px;
  display: flex; flex-direction: column;
  overflow: hidden;
}
.pick-search {
  border: none; border-bottom: 1px solid var(--border);
  padding: 8px 10px; font-size: var(--fs-sm); height: 34px;
  flex-shrink: 0; background: transparent;
}
.pick-search:focus { outline: none; }

.pick-tabs {
  display: flex; gap: 2px; padding: 4px 6px;
  overflow-x: auto; flex-shrink: 0;
  border-bottom: 1px solid var(--border-light);
}
.pick-tabs::-webkit-scrollbar { height: 0; }
.tab {
  font-size: 11px; padding: 2px 8px; height: 22px;
  border: 1px solid transparent; border-radius: var(--r-full);
  background: transparent; color: var(--text-secondary);
  cursor: pointer; white-space: nowrap; flex-shrink: 0;
  transition: all var(--fast) var(--ease);
}
.tab:hover { background: var(--surface-hover); color: var(--text); }
.tab.sel { background: var(--accent-subtle); color: var(--accent); border-color: var(--accent); }

.pick-grid {
  flex: 1; overflow-y: auto;
  padding: 4px;
  contain: strict;
}
.pick-row {
  display: flex;
  height: 32px;
}
.pick-item {
  display: inline-flex; align-items: center; justify-content: center;
  width: 36px; height: 32px; padding: 0;
  border: none; border-radius: var(--r-sm);
  background: transparent; color: var(--text-secondary);
  cursor: pointer; transition: all var(--fast) var(--ease);
  flex-shrink: 0;
}
.pick-item:hover { background: var(--surface-hover); color: var(--text); }
.pick-item.sel { background: var(--accent-subtle); color: var(--accent); }

.pick-empty {
  padding: 8px 10px; font-size: var(--fs-xs); color: var(--text-muted);
  border-top: 1px solid var(--border-light); flex-shrink: 0;
}
</style>
