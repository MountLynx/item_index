<template>
  <footer class="statusbar">
    <span class="path">{{ repoStore.repoPath || '未打开仓库' }}</span>
    <span class="count">{{ repoStore.itemCount }} 条目</span>
    <span class="status">⚪ 就绪</span>
  </footer>
</template>

<script setup lang="ts">
import { useRepoStore } from '@/stores/repo'
const repoStore = useRepoStore()
</script>

<style scoped>
.statusbar {
  height: var(--statusbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  border-top: 1px solid var(--border);
  background: var(--surface);
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}
.path { max-width: 500px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
