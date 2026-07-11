<template>
  <aside class="right-panel">
    <div v-if="!detail" class="empty-hint">
      <span class="hint-icon">←</span>
      <p>选择条目查看详情</p>
    </div>
    <template v-else>
      <div class="item-header">
        <div class="item-title">
          <span class="item-icon">{{ detail.item_type.icon }}</span>
          <span class="item-name">{{ detail.item.name }}</span>
        </div>
        <div class="item-id font-mono text-muted">{{ detail.item.id }}</div>
      </div>

      <div class="panel-section">
        <div class="section-label">属性</div>
        <PropertiesForm :detail="detail" />
      </div>

      <div class="section-sep" />

      <div class="panel-section">
        <div class="section-label">分组</div>
        <div class="chip-row">
          <span v-for="g in detail.groups" :key="g.id" class="chip">📁 {{ g.name }}</span>
          <span v-if="detail.groups.length === 0" class="text-muted text-sm">未分组</span>
        </div>
      </div>

      <div class="section-sep" />

      <div class="panel-section">
        <div class="section-label">标签</div>
        <div class="chip-row">
          <span v-for="t in detail.tags" :key="t.id" class="chip tag-chip"># {{ t.name }}</span>
          <span v-if="detail.tags.length === 0" class="text-muted text-sm">无标签</span>
        </div>
      </div>

      <div class="section-sep" />

      <div class="panel-section file-section">
        <FileTree :item-id="detail.item.id" />
      </div>
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useItemStore } from '@/stores/items'
import PropertiesForm from './PropertiesForm.vue'
import FileTree from './FileTree.vue'

const itemStore = useItemStore()
const detail = computed(() => itemStore.detail)
</script>

<style scoped>
.right-panel {
  width: var(--right-panel-width); flex-shrink: 0;
  border-left: 1px solid var(--border);
  background: var(--surface);
  display: flex; flex-direction: column;
  overflow-y: auto;
}
.empty-hint {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 100%; color: var(--text-muted); gap: var(--space-2);
}
.hint-icon { font-size: 32px; opacity: 0.3; }

.item-header { padding: var(--space-4) var(--space-4) var(--space-3); }
.item-title { display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-1); }
.item-icon { font-size: 20px; }
.item-name { font-size: var(--font-size-lg); font-weight: var(--weight-semibold); }
.item-id {
  font-size: var(--font-size-xs); color: var(--text-muted);
  background: var(--surface-hover); padding: 1px 6px; border-radius: var(--radius-xs);
  display: inline-block; margin-top: 2px;
}

.panel-section { padding: var(--space-2) var(--space-4); }
.section-label {
  font-size: var(--font-size-xs); font-weight: var(--weight-semibold);
  color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;
  margin-bottom: var(--space-2);
}
.section-sep { height: 1px; background: var(--border-light); margin: 0 var(--space-4); }
.file-section { flex: 1; min-height: 120px; }

.chip-row { display: flex; flex-wrap: wrap; gap: var(--space-1); }
.chip {
  font-size: var(--font-size-xs); padding: 2px 10px; border-radius: var(--radius-full);
  background: var(--bg); border: 1px solid var(--border-light); color: var(--text-secondary);
}
.tag-chip { color: var(--accent); border-color: var(--accent-subtle); background: var(--accent-subtle); }

.text-sm { font-size: var(--font-size-xs); }
</style>
