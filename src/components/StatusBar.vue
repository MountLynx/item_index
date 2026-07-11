<template>
  <footer class="statusbar">
    <span class="path text-muted">{{ repoStore.repoPath || '未打开仓库' }}</span>
    <div class="right">
      <span class="count">{{ repoStore.itemCount }} 条目</span>
      <span class="dot" />
      <span class="status">就绪</span>
    </div>
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
  padding: 0 var(--space-4);
  background: var(--surface);
  border-top: 1px solid var(--border-light);
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  flex-shrink: 0;
  letter-spacing: 0.01em;
}
.right { display: flex; align-items: center; gap: var(--space-2); }
.dot {
  width: 5px; height: 5px; border-radius: var(--radius-full);
  background: var(--success);
}
</style>
