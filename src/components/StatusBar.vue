<template>
  <footer class="sb">
    <span class="path text-muted">{{ repoStore.repoPath || 'Index' }}</span>
    <div class="right">
      <span>{{ repoStore.itemCount }}{{ $t('statusBar.items') }}</span>
      <span class="dot" />
      <span>{{ $t('statusBar.ready') }}</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { useRepoStore } from '@/stores/repo'
const repoStore = useRepoStore()
</script>

<style scoped>
.sb {
  height: var(--status-h); display: flex; align-items: center; justify-content: space-between;
  padding: 0 16px; background: var(--surface); border-top: 1px solid var(--border);
  font-size: var(--fs-xs); color: var(--text-muted); flex-shrink: 0;
}
.right { display: flex; align-items: center; gap: 8px; }
.dot { width: 5px; height: 5px; border-radius: var(--r-full); background: var(--success); }
</style>
