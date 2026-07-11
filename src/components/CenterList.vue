<template>
  <div class="center-list" @click.self="itemStore.clearSelection()">
    <div v-if="items.length === 0" class="empty">
      <p>暂无条目</p>
      <button class="primary" @click="$emit('newItem')">+ 创建第一个条目</button>
    </div>
    <div
      v-for="item in items"
      :key="item.id"
      class="item-row"
      :class="{ selected: item.id === selectedId }"
      @click="selectItem(item.id)"
      @contextmenu.prevent="showContextMenu($event, item)"
    >
      <span class="icon">{{ getIcon(item.type_id) }}</span>
      <span class="name">{{ item.name }}</span>
      <span class="meta">{{ getTypeName(item.type_id) }} · {{ timeAgo(item.updated_at) }}</span>
    </div>

    <!-- Context Menu -->
    <div v-if="contextMenu.show" class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
      <div class="menu-item danger" @click="deleteItem">🗑 删除条目</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import type { Item } from '@/types/bindings'

const itemStore = useItemStore()
const typeStore = useTypeStore()

const items = computed(() => itemStore.items)
const selectedId = computed(() => itemStore.selectedId)

const contextMenu = ref({ show: false, x: 0, y: 0, item: null as Item | null })

defineEmits<{ newItem: [] }>()

function getIcon(typeId: number): string {
  return typeStore.getTypeById(typeId)?.icon || '📄'
}

function getTypeName(typeId: number): string {
  return typeStore.getTypeById(typeId)?.name || '?'
}

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const days = Math.floor(diff / 86400000)
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  return `${days} 天前`
}

async function selectItem(id: string) {
  await itemStore.select(id)
  contextMenu.value.show = false
}

function showContextMenu(event: MouseEvent, item: Item) {
  contextMenu.value = { show: true, x: event.clientX, y: event.clientY, item }
}

async function deleteItem() {
  if (contextMenu.value.item && confirm(`确定删除"${contextMenu.value.item.name}"？`)) {
    await itemStore.remove(contextMenu.value.item.id)
  }
  contextMenu.value.show = false
}
</script>

<style scoped>
.center-list { flex: 1; overflow-y: auto; padding: 8px 0; }
.empty { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 48px; color: var(--text-secondary); }
.item-row {
  display: flex; align-items: center; gap: 8px; padding: 8px 16px; cursor: pointer;
  border-bottom: 1px solid var(--border); user-select: none;
}
.item-row:hover { background: var(--surface); }
.item-row.selected { background: var(--accent); color: #fff; }
.item-row.selected .meta { color: rgba(255,255,255,0.7); }
.icon { font-size: 18px; }
.name { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta { font-size: 12px; color: var(--text-secondary); }
.context-menu {
  position: fixed; background: var(--surface); border: 1px solid var(--border);
  border-radius: 4px; padding: 4px 0; z-index: 100; min-width: 140px; box-shadow: 0 2px 8px rgba(0,0,0,0.15);
}
.menu-item { padding: 6px 16px; cursor: pointer; font-size: 13px; }
.menu-item:hover { background: var(--border); }
.menu-item.danger { color: var(--danger); }
</style>
