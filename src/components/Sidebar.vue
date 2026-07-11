<template>
  <aside class="sidebar">
    <div class="section">
      <div class="section-title">📁 分组</div>
      <GroupTree @select="onGroupSelect" />
    </div>
    <div class="divider" />
    <div class="section">
      <div class="section-title">🏷 标签</div>
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
  width: var(--sidebar-width); flex-shrink: 0; border-right: 1px solid var(--border);
  background: var(--surface); display: flex; flex-direction: column; overflow-y: auto;
}
.section { padding: 8px; }
.section-title { font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--text-secondary); margin-bottom: 6px; padding: 0 4px; }
.divider { height: 1px; background: var(--border); margin: 4px 8px; }
</style>
