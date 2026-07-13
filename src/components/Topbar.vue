<template>
  <header class="topbar">
    <div class="left">
      <TablerIcon name="database" :size="22" :stroke="1.5" />
      <span class="repo-name">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Index' }}</span>
    </div>
    <div class="actions">
      <button class="primary" @click="$emit('newItem')">
        <TablerIcon name="plus" :size="16" /> 新建条目
      </button>
      <button class="icon-btn" @click="settingsRef?.open()" title="设置">
        <TablerIcon name="settings" :size="18" />
      </button>
      <button class="icon-btn" @click="$emit('openTypeManager')" title="类别管理">
        <TablerIcon name="category" :size="18" />
      </button>
    </div>
    <SettingsModal ref="settingsRef" />
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import TablerIcon from './TablerIcon.vue'
import SettingsModal from './SettingsModal.vue'

const repoStore = useRepoStore()
const settingsRef = ref<InstanceType<typeof SettingsModal> | null>(null)

function basename(p: string): string { return p.split(/[/\\]/).pop() || p }
defineEmits<{ newItem: []; openTypeManager: [] }>()
</script>

<style scoped>
.topbar {
  height: var(--topbar-h); display: flex; align-items: center; justify-content: space-between;
  padding: 0 16px; background: var(--surface); border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.left { display: flex; align-items: center; gap: 8px; min-width: 0; color: var(--accent); }
.repo-name { font-weight: var(--fw-semibold); font-size: var(--fs-base); color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.actions { display: flex; align-items: center; gap: 4px; }
</style>
