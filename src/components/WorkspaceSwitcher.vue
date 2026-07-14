<template>
  <div class="ws-switcher" ref="switcherRef">
    <button class="ws-btn" @click="open = !open">
      <TablerIcon :name="activeWs?.icon || 'layout'" :size="16" />
      <span class="ws-label">{{ activeWs?.name || '工作区' }}</span>
      <TablerIcon name="chevron-down" :size="12" />
    </button>
    <div v-if="open" class="ws-drop">
      <div v-for="ws in wsStore.workspaces" :key="ws.name" class="ws-item"
        :class="{ active: ws.name === wsStore.activeName }"
        @click="switchTo(ws.name)">
        <TablerIcon :name="ws.icon" :size="16" />
        <span>{{ ws.name }}</span>
        <span v-if="ws.is_default" class="ws-badge">默认</span>
      </div>
      <div class="ws-sep"></div>
      <div class="ws-item ws-action" @click="openSettings">
        <TablerIcon name="settings" :size="16" />
        <span>管理工作区...</span>
      </div>
    </div>
    <div v-if="open" class="ws-backdrop" @click="open = false"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import TablerIcon from './TablerIcon.vue'

const wsStore = useWorkspaceStore()
const open = ref(false)
const openSettings = inject<() => void>('openSettings', () => {})

const activeWs = computed(() =>
  wsStore.workspaces.find(w => w.name === wsStore.activeName)
)

async function switchTo(name: string) {
  if (name === wsStore.activeName) { open.value = false; return }
  await wsStore.activate(name)
  open.value = false
}
</script>

<style scoped>
.ws-switcher { position: relative; }
.ws-btn {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 10px; height: 30px;
  border: 1px solid var(--border); border-radius: var(--r-md);
  background: var(--surface); color: var(--text);
  font-size: var(--fs-xs); cursor: pointer;
  transition: background var(--fast) var(--ease);
}
.ws-btn:hover { background: var(--surface-hover); }
.ws-label { font-weight: var(--fw-medium); max-width: 100px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ws-drop {
  position: absolute; top: 100%; left: 0; margin-top: 4px; z-index: 150;
  background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--r-lg); min-width: 200px; padding: 4px;
  box-shadow: var(--shadow-md);
}
.ws-item {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; border-radius: var(--r-sm);
  font-size: var(--fs-sm); cursor: pointer;
  transition: background var(--fast) var(--ease);
}
.ws-item:hover { background: var(--surface-hover); }
.ws-item.active { background: var(--accent-subtle); font-weight: var(--fw-semibold); }
.ws-badge { font-size: 10px; color: var(--accent); margin-left: auto; }
.ws-sep { border-top: 1px solid var(--border); margin: 4px 0; }
.ws-action { color: var(--text-secondary); }
.ws-backdrop { position: fixed; inset: 0; z-index: 140; }
</style>
