<template>
  <aside class="sidebar">
    <div class="section">
      <div class="section-header">
        <span class="section-icon">📁</span>
        <span class="section-title">分组</span>
      </div>
      <GroupTree @select="onGroupSelect" />
    </div>
    <div class="section-sep" />
    <div class="section">
      <div class="section-header">
        <span class="section-icon">🏷</span>
        <span class="section-title">标签</span>
      </div>
      <TagList @select="onTagSelect" />
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useItemStore } from '@/stores/items'
import GroupTree from './GroupTree.vue'
import TagList from './TagList.vue'

const itemStore = useItemStore()
const selectedGroupId = ref<number | null>(null)
const selectedTagId = ref<number | null>(null)

async function onGroupSelect(groupId: number | null) {
  selectedGroupId.value = groupId
  await itemStore.fetchList(selectedGroupId.value, selectedTagId.value)
}
async function onTagSelect(tagId: number | null) {
  selectedTagId.value = tagId
  await itemStore.fetchList(selectedGroupId.value, selectedTagId.value)
}
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-width); flex-shrink: 0;
  background: var(--surface);
  border-right: 1px solid var(--border);
  display: flex; flex-direction: column;
  overflow-y: auto; overflow-x: hidden;
}
.section { padding: var(--space-3) var(--space-2); }
.section-header {
  display: flex; align-items: center; gap: var(--space-1);
  padding: var(--space-1) var(--space-2); margin-bottom: var(--space-1);
}
.section-icon { font-size: var(--font-size-sm); line-height: 1; }
.section-title {
  font-size: var(--font-size-xs); font-weight: var(--weight-semibold);
  color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;
}
.section-sep { height: 1px; background: var(--border-light); margin: 0 var(--space-3); }
</style>
