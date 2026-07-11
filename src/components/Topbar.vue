<template>
  <header class="topbar">
    <div class="left">
      <TablerIcon name="database" :size="22" :stroke="1.5" />
      <span class="repo-name">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Vault' }}</span>
    </div>
    <div class="actions">
      <button class="primary" @click="$emit('newItem')">
        <TablerIcon name="plus" :size="16" /> 新建条目
      </button>
      <button class="icon-btn" @click="themeStore.toggle()" :title="themeStore.mode === 'light' ? '深色模式' : '浅色模式'">
        <TablerIcon :name="themeStore.mode === 'light' ? 'moon' : 'sun'" :size="18" />
      </button>
      <button class="icon-btn" @click="$emit('settings')" title="设置">
        <TablerIcon name="settings" :size="18" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'
import TablerIcon from './TablerIcon.vue'

const repoStore = useRepoStore()
const themeStore = useThemeStore()

function basename(p: string): string { return p.split(/[/\\]/).pop() || p }
defineEmits<{ newItem: []; settings: [] }>()
</script>

<style scoped>
.topbar {
  height: var(--topbar-h); display: flex; align-items: center; justify-content: space-between;
  padding: 0 var(--space-4, 16px); background: var(--surface); border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.left { display: flex; align-items: center; gap: 8px; min-width: 0; color: var(--accent); }
.repo-name { font-weight: var(--fw-semibold); font-size: var(--fs-base); color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.actions { display: flex; align-items: center; gap: 4px; }
</style>
