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

        <div v-if="search && !loadErr && !iconNames.length" class="pick-empty">
          加载中…
        </div>
        <div v-if="search && loadErr" class="pick-empty pick-err">
          ⚠ 图标加载失败
          <button class="retry-btn" @click="retry">重试</button>
        </div>
        <div v-if="search && !loadErr && iconNames.length && !results.length" class="pick-empty">
          无匹配，可输入 emoji 直接使用
        </div>
      </div>
    </Teleport>
  </div>

  <div v-if="open" class="pick-backdrop" @click="open = false" />
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import TablerIcon from './TablerIcon.vue'

defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()

const COMMON: string[] = [
  'circle', 'file', 'folder', 'star', 'heart', 'check', 'x',
  'plus', 'minus', 'search', 'settings', 'user', 'users', 'calendar',
  'clock', 'bell', 'mail', 'message', 'photo', 'camera', 'music',
  'video', 'map', 'lock', 'key', 'trash', 'edit', 'pencil',
  'book', 'bookmark', 'tag', 'flag', 'link', 'globe', 'home',
  'phone', 'download', 'upload', 'share', 'filter', 'copy',
  'database', 'code', 'chart-bar', 'list', 'info-circle',
  'alert-circle', 'help-circle', 'rocket', 'package',
]

const open = ref(false)
const search = ref('')
const rootEl = ref<HTMLElement | null>(null)
const searchEl = ref<HTMLInputElement | null>(null)
const gridEl = ref<HTMLElement | null>(null)
const dropStyle = ref<Record<string, string>>({})

const iconNames = ref<string[]>([])
const scrollTop = ref(0)
const loadErr = ref('')

const COLS = 6
const ROW_H = 32
const VISIBLE_ROWS = 9
const BUFFER = 3
const MAX_RESULTS = 200

async function loadIcons() {
  if (iconNames.value.length) return
  try {
    const mod = await import('@/assets/icon-names')
    const seen = new Set<string>()
    const flat: string[] = []
    for (const g of mod.ICON_GROUPS) {
      for (const n of g.icons) {
        if (!seen.has(n)) {
          seen.add(n)
          flat.push(n)
        }
      }
    }
    iconNames.value = flat
  } catch (e) {
    loadErr.value = String(e)
    console.error('Failed to load icon names:', e)
  }
}

function retry() {
  loadErr.value = ''
  iconNames.value = []
  loadIcons()
}

const results = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return [] as string[]
  const matched: string[] = []
  for (const name of iconNames.value) {
    if (name.includes(q)) matched.push(name)
    if (matched.length >= MAX_RESULTS) break
  }
  return matched
})

const displayIcons = computed(() =>
  search.value.trim() ? results.value : COMMON
)

const visibleRows = computed(() => {
  const list = displayIcons.value
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

const totalHeight = computed(() =>
  Math.max(1, Math.ceil(displayIcons.value.length / COLS)) * ROW_H
)

const offsetY = computed(() => {
  const list = displayIcons.value
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

watch(search, () => { scrollTop.value = 0 })
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
  min-height: 200px;
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
.pick-err {
  display: flex; align-items: center; gap: 8px;
}
.retry-btn {
  background: var(--accent); color: var(--accent-fg);
  border: none; border-radius: var(--r-sm);
  padding: 2px 8px; font-size: var(--fs-xs); cursor: pointer;
}
.retry-btn:hover { background: var(--accent-hover); }
</style>
