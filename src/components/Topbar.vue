<template>
  <header class="topbar">
    <span class="repo-name">{{ repoStore.repoPath || 'Vault' }}</span>
    <div class="actions">
      <button class="primary" @click="$emit('newItem')">+ 新建条目</button>
      <button @click="themeStore.toggle()" title="切换主题">🌓</button>
      <button @click="$emit('settings')" title="设置">⚙</button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'

const repoStore = useRepoStore()
const themeStore = useThemeStore()

defineEmits<{ newItem: []; settings: [] }>()
</script>

<style scoped>
.topbar {
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
  flex-shrink: 0;
}
.repo-name { font-weight: 600; max-width: 400px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.actions { display: flex; gap: 6px; }
</style>
