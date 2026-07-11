<template>
  <div class="center-list">
    <div v-if="items.length === 0" class="empty-state" @click="$emit('newItem')">
      <div class="empty-icon">📋</div>
      <p class="empty-text">暂无条目</p>
      <button class="primary">+ 创建第一个条目</button>
    </div>
    <div v-else class="list">
      <div
        v-for="item in items" :key="item.id"
        class="item-row" :class="{ active: item.id === selectedId }"
        @click="selectItem(item.id)"
        @contextmenu.prevent="showMenu($event, item)"
      >
        <span class="type-icon">{{ getIcon(item.type_id) }}</span>
        <div class="item-body">
          <span class="item-name">{{ item.name }}</span>
          <span class="item-meta">{{ getTypeName(item.type_id) }} &middot; {{ timeAgo(item.updated_at) }}</span>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="menu.show" class="context-overlay" @click="menu.show = false" @contextmenu.prevent="menu.show = false">
        <div class="context-menu" :style="{ left: menu.x + 'px', top: menu.y + 'px' }">
          <button class="menu-item danger" @click="deleteItem">🗑 删除条目</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import type { Item } from '@/types/bindings'

const itemStore = useItemStore()
const typeStore = useTypeStore()

const items = computed(() => itemStore.items)
const selectedId = computed(() => itemStore.selectedId)

const menu = reactive({ show: false, x: 0, y: 0, item: null as Item | null })

defineEmits<{ newItem: [] }>()

function getIcon(typeId: number) { return typeStore.getTypeById(typeId)?.icon || '📄' }
function getTypeName(typeId: number) { return typeStore.getTypeById(typeId)?.name || '?' }

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins} 分钟前`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs} 小时前`
  const days = Math.floor(hrs / 24)
  if (days === 1) return '昨天'
  if (days < 30) return `${days} 天前`
  return `${Math.floor(days / 30)} 月前`
}

async function selectItem(id: string) {
  await itemStore.select(id)
  menu.show = false
}

function showMenu(e: MouseEvent, item: Item) {
  menu.show = true; menu.x = e.clientX; menu.y = e.clientY; menu.item = item
}

async function deleteItem() {
  if (menu.item && confirm(`确定删除"${menu.item.name}"？`)) {
    await itemStore.remove(menu.item.id)
  }
  menu.show = false
}
</script>

<style scoped>
.center-list { flex: 1; overflow-y: auto; background: var(--bg); }

.empty-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 100%; gap: var(--space-3); color: var(--text-muted);
}
.empty-icon { font-size: 48px; opacity: 0.4; }
.empty-text { font-size: var(--font-size-base); margin-bottom: var(--space-1); }

.list { padding: var(--space-1) 0; }

.item-row {
  display: flex; align-items: center; gap: var(--space-3);
  padding: var(--space-2) var(--space-4); margin: 1px var(--space-2);
  cursor: pointer; border-radius: var(--radius-md);
  transition: background var(--duration-fast) var(--ease-out);
  user-select: none;
}
.item-row:hover { background: var(--surface-hover); }
.item-row.active {
  background: var(--accent); color: var(--accent-foreground);
}
.item-row.active .item-meta { color: rgba(255,255,255,0.7); }

.type-icon { font-size: 20px; flex-shrink: 0; line-height: 1; }

.item-body { display: flex; flex-direction: column; min-width: 0; gap: 1px; }
.item-name { font-weight: var(--weight-medium); font-size: var(--font-size-base); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.item-meta { font-size: var(--font-size-xs); color: var(--text-muted); }

.context-overlay { position: fixed; inset: 0; z-index: 100; }
.context-menu {
  position: fixed;
  background: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-1);
  min-width: 160px;
  box-shadow: var(--shadow-lg);
  z-index: 101;
}
.menu-item {
  display: flex; align-items: center; gap: var(--space-2);
  width: 100%; padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-sm); border-radius: var(--radius-sm);
  border: none; background: none; cursor: pointer; height: auto;
  color: var(--danger); transition: background var(--duration-fast) var(--ease-out);
}
.menu-item:hover { background: var(--danger-subtle); }
</style>
