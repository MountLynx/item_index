<template>
  <aside class="sidebar">
    <div class="sec">
      <div class="sec-hd"><TablerIcon name="folder" :size="14" /> 分组</div>
      <GroupTree @select="onGroupSelect" />
    </div>
    <div class="sep" />
    <div class="sec">
      <div class="sec-hd"><TablerIcon name="tag" :size="14" /> 标签</div>
      <TagList @select="onTagSelect" />
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useItemStore } from '@/stores/items'
import GroupTree from './GroupTree.vue'
import TagList from './TagList.vue'
import TablerIcon from './TablerIcon.vue'

const itemStore = useItemStore()
const gid = ref<number | null>(null)
const tid = ref<number | null>(null)

async function onGroupSelect(id: number | null) { gid.value = id; await itemStore.fetchList(gid.value, tid.value) }
async function onTagSelect(id: number | null) { tid.value = id; await itemStore.fetchList(gid.value, tid.value) }
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-w); flex-shrink: 0; background: var(--surface);
  border-right: 1px solid var(--border); display: flex; flex-direction: column;
  overflow-y: auto;
}
.sec { padding: 12px 8px; }
.sec-hd {
  display: flex; align-items: center; gap: 6px; padding: 4px 8px; margin-bottom: 4px;
  font-size: var(--fs-xs); font-weight: var(--fw-semibold); color: var(--text-muted);
  text-transform: uppercase; letter-spacing: 0.05em;
}
.sep { height: 1px; background: var(--border); margin: 0 12px; }
</style>
