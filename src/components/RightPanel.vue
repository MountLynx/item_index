<template>
  <aside class="right-panel">
    <div v-if="!detail" class="empty">选择一个条目查看详情</div>
    <template v-else>
      <div class="item-header">
        <div class="item-name">{{ detail.item_type.icon }} {{ detail.item.name }}</div>
        <div class="item-meta">{{ detail.item.id }} · {{ detail.item_type.name }}</div>
      </div>

      <div class="section">
        <div class="section-title">📝 属性</div>
        <PropertiesForm :detail="detail" />
      </div>

      <div class="divider" />

      <div class="section">
        <div class="section-title">📁 分组</div>
        <div class="chip-list">
          <span v-for="g in detail.groups" :key="g.id" class="chip">📁 {{ g.name }}</span>
          <span v-if="detail.groups.length === 0" class="none">未分组</span>
        </div>
      </div>

      <div class="divider" />

      <div class="section">
        <div class="section-title">🏷 标签</div>
        <div class="chip-list">
          <span v-for="t in detail.tags" :key="t.id" class="chip"># {{ t.name }}</span>
          <span v-if="detail.tags.length === 0" class="none">无标签</span>
        </div>
      </div>

      <div class="divider" />

      <FileTree :item-id="detail.item.id" />
    </template>
  </aside>
</template>

<script setup lang="ts">
import { useItemStore } from '@/stores/items'
import { computed } from 'vue'
import PropertiesForm from './PropertiesForm.vue'
import FileTree from './FileTree.vue'

const itemStore = useItemStore()
const detail = computed(() => itemStore.detail)
</script>

<style scoped>
.right-panel {
  width: var(--right-panel-width); flex-shrink: 0; border-left: 1px solid var(--border);
  padding: 12px; overflow-y: auto; background: var(--surface);
}
.empty { color: var(--text-secondary); text-align: center; padding: 48px 0; }
.item-header { margin-bottom: 12px; }
.item-name { font-size: 16px; font-weight: 600; }
.item-meta { font-size: 11px; color: var(--text-secondary); font-family: monospace; margin-top: 2px; }
.section { margin-bottom: 8px; }
.section-title { font-size: 11px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px; text-transform: uppercase; }
.divider { height: 1px; background: var(--border); margin: 8px 0; }
.chip-list { display: flex; flex-wrap: wrap; gap: 4px; }
.chip { font-size: 12px; padding: 2px 8px; border-radius: 10px; background: var(--bg); border: 1px solid var(--border); }
.none { font-size: 12px; color: var(--text-secondary); }
</style>
