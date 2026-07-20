<template>
  <div class="cl">
    <div v-if="items.length === 0" class="empty" @click="$emit('newItem')">
      <TablerIcon name="clipboard" :size="40" :stroke="1" class="empty-icon" />
      <p class="empty-text">{{ $t('centerList.empty') }}</p>
      <button class="primary">{{ $t('centerList.newItem') }}</button>
    </div>
    <div v-else class="list">
      <div v-for="item in items" :key="item.id" class="row" :class="{ sel: item.id === selectedId }"
        @click="selectItem(item.id)" @dblclick="handleDoubleClick(item)" @contextmenu.prevent="showMenu($event, item)">
        <span class="grip" draggable="true" @dragstart="onDragStart($event, item.id)" @click.stop :title="$t('centerList.dragToGroup')">
          <TablerIcon name="grip-vertical" :size="14" />
        </span>
        <TablerIcon :name="typeIcon(item.type_id)" :size="19" />
        <TablerIcon v-if="itemStore.subRepoMap[item.id]" name="database" :size="12" class="repo-badge"
          :title="$t('centerList.subRepo')" />
        <div class="body">
          <span class="name">{{ item.name }}</span>
          <span class="meta">{{ typeName(item.type_id) }} &middot; {{ ago(item.updated_at) }}</span>
        </div>
      </div>
    </div>
    <Teleport to="body">
      <div v-if="menu.show" class="menu-overlay" @click="menu.show = false" @contextmenu.prevent="menu.show = false">
        <div class="menu" :style="{ left: menu.x + 'px', top: menu.y + 'px' }">
          <button v-if="menu.item && !itemStore.subRepoMap[menu.item.id]" class="menu-item" @click="createSubRepoAction">
            <TablerIcon name="database" :size="15" /> {{ $t('centerList.createSubRepo') }}
          </button>
          <button class="menu-item" @click="deleteItem"><TablerIcon name="trash" :size="15" /> {{ $t('centerList.deleteItem') }}</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import type { Item } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()
const itemStore = useItemStore()
const typeStore = useTypeStore()
const items = computed(() => itemStore.items)
const selectedId = computed(() => itemStore.selectedId)
const menu = reactive({ show: false, x: 0, y: 0, item: null as Item | null })

defineEmits<{ newItem: [] }>()

function typeIcon(id: number) { return typeStore.getTypeById(id)?.icon || 'file' }
function typeName(id: number) { return typeStore.getTypeById(id)?.name || '?' }
function ago(iso: string): string {
  const m = Math.floor((Date.now() - new Date(iso).getTime()) / 60000)
  if (m < 1) return t('centerList.justNow'); if (m < 60) return `${m}${t('centerList.minAgo')}`
  const h = Math.floor(m / 60); if (h < 24) return `${h}${t('centerList.hourAgo')}`
  const d = Math.floor(h / 24); if (d === 1) return t('centerList.yesterday'); if (d < 30) return `${d}${t('centerList.dayAgo')}`
  return `${Math.floor(d / 30)}${t('centerList.monthAgo')}`
}

function onDragStart(e: DragEvent, id: string) {
  e.dataTransfer!.setData('text/plain', id)
  e.dataTransfer!.effectAllowed = 'move'
}

async function selectItem(id: string) { await itemStore.select(id); menu.show = false }

async function handleDoubleClick(item: Item) {
  const isSubRepo = itemStore.subRepoMap[item.id]
  if (isSubRepo) {
    await itemStore.openSubRepoWindow(item.id)
  } else {
    await itemStore.openItemFolder(item.id)
  }
}

async function createSubRepoAction() {
  if (menu.item) {
    await itemStore.createSubRepo(menu.item.id)
  }
  menu.show = false
}

function showMenu(e: MouseEvent, item: Item) { menu.show = true; menu.x = e.clientX; menu.y = e.clientY; menu.item = item }
async function deleteItem() { if (menu.item && confirm(t('centerList.confirmDelete', { name: menu.item.name }))) await itemStore.remove(menu.item.id); menu.show = false }
</script>

<style scoped>
.cl { flex: 1; overflow-y: auto; background: var(--bg); }
.empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 12px; color: var(--text-secondary); }
.empty-icon { opacity: 0.2; }
.empty-text { font-size: var(--fs-base); }
.list { padding: 4px 0; }
.row {
  display: flex; align-items: center; gap: 10px;
  padding: 4px 12px 4px 8px; margin: 1px 8px; border-radius: var(--r-md);
  cursor: pointer; user-select: none; transition: background var(--fast) var(--ease);
}
.row:hover { background: var(--surface-hover); }
.row:hover .grip { opacity: 1; }
.row.sel { background: var(--accent); color: var(--accent-fg); }
.row.sel .meta { color: rgba(255,255,255,0.65); }
.grip { opacity: 0; cursor: grab; flex-shrink: 0; display: flex; align-items: center; transition: opacity var(--fast) var(--ease); color: var(--text-muted); }
.grip:active { cursor: grabbing; }
.row.sel .grip { color: var(--accent-fg); opacity: 0.8; }
.body { display: flex; flex-direction: column; min-width: 0; gap: 1px; }
.name { font-weight: var(--fw-medium); font-size: var(--fs-base); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta { font-size: var(--fs-xs); color: var(--text-secondary); }
.menu-overlay { position: fixed; inset: 0; z-index: 100; }
.menu {
  position: fixed; background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--r-lg); padding: 4px; min-width: 160px; z-index: 101;
  box-shadow: var(--shadow-md);
}
.menu-item {
  display: flex; align-items: center; gap: 8px; width: 100%;
  padding: 8px 12px; font-size: var(--fs-sm); border-radius: var(--r-sm);
  border: none; background: none; cursor: pointer; height: auto; color: var(--danger);
}
.menu-item:hover { background: var(--danger-subtle); }
.repo-badge { color: var(--accent); flex-shrink: 0; margin-left: -4px; }
</style>
