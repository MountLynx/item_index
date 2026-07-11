<template>
  <aside class="sidebar">
    <div class="sec">
      <div class="sec-hd" @click="clearGroup">
        <TablerIcon name="folder" :size="14" />
        <span>分组</span>
        <span v-if="gid !== null" class="clear-hint">← 显示全部</span>
      </div>
      <GroupTree :selected-id="gid" @select="onGroupSelect" />
    </div>
    <div class="sep" />
    <div class="sec">
      <div class="sec-hd" @click="clearTag">
        <TablerIcon name="tag" :size="14" />
        <span>标签</span>
        <span v-if="tid !== null" class="clear-hint">← 显示全部</span>
      </div>
      <TagList :selected-id="tid" @select="onTagSelect" />
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
async function clearGroup() { gid.value = null; await itemStore.fetchList(null, tid.value) }
async function clearTag() { tid.value = null; await itemStore.fetchList(gid.value, null) }
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
  text-transform: uppercase; letter-spacing: 0.05em; cursor: pointer;
  border-radius: var(--r-sm); transition: background var(--fast) var(--ease);
}
.sec-hd:hover { background: var(--surface-hover); }
.clear-hint { font-weight: var(--fw-normal); text-transform: none; color: var(--accent); font-size: 10px; margin-left: auto; }
.sep { height: 1px; background: var(--border); margin: 0 12px; }
</style>
