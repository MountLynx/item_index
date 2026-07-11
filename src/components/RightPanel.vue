<template>
  <aside class="rp">
    <div v-if="!detail" class="empty">
      <TablerIcon name="arrow-left" :size="28" :stroke="1" />
      <p>选择条目查看详情</p>
    </div>
    <template v-else>
      <div class="hd">
        <div class="title"><TablerIcon :name="detail.item_type.icon" :size="20" /> {{ detail.item.name }}</div>
        <div class="id font-mono">{{ detail.item.id }}</div>
      </div>
      <div class="sec"><div class="lbl">属性</div><PropertiesForm :detail="detail" /></div>
      <div class="sep" />
      <div class="sec">
        <div class="lbl">分组</div>
        <div class="chips">
          <span v-for="g in detail.groups" :key="g.id" class="chip"><TablerIcon name="folder" :size="13" />{{ g.name }}</span>
          <span v-if="detail.groups.length === 0" class="text-muted text-xs">未分组</span>
        </div>
      </div>
      <div class="sep" />
      <div class="sec">
        <div class="lbl">标签</div>
        <div class="chips">
          <span v-for="t in detail.tags" :key="t.id" class="chip tag"><TablerIcon name="hash" :size="13" />{{ t.name }}</span>
          <span v-if="detail.tags.length === 0" class="text-muted text-xs">无标签</span>
        </div>
      </div>
      <div class="sep" />
      <div class="sec file-sec">
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
import TablerIcon from './TablerIcon.vue'

const itemStore = useItemStore()
const detail = computed(() => itemStore.detail)
</script>

<style scoped>
.rp {
  width: var(--right-w); flex-shrink: 0; border-left: 1px solid var(--border);
  background: var(--surface); display: flex; flex-direction: column; overflow-y: auto;
}
.empty { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 8px; color: var(--text-muted); }
.hd { padding: 16px 16px 12px; }
.title { display: flex; align-items: center; gap: 8px; font-size: var(--fs-lg); font-weight: var(--fw-semibold); margin-bottom: 4px; }
.id { font-size: var(--fs-xs); color: var(--text-secondary); background: var(--surface-hover); padding: 1px 6px; border-radius: var(--r-sm); display: inline-block; }
.sec { padding: 8px 16px; }
.lbl { font-size: var(--fs-xs); font-weight: var(--fw-semibold); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 8px; }
.sep { height: 1px; background: var(--border); margin: 0 16px; }
.file-sec { flex: 1; min-height: 100px; }
.chips { display: flex; flex-wrap: wrap; gap: 4px; }
.chip { display: inline-flex; align-items: center; gap: 4px; font-size: var(--fs-xs); padding: 2px 10px; border-radius: var(--r-full); background: var(--bg); color: var(--text-secondary); }
.chip.tag { color: var(--accent); background: var(--accent-subtle); }
</style>
