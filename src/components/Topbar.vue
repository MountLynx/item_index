<template>
  <header class="topbar">
    <div class="left">
      <span class="logo">◆</span>
      <span class="repo-name">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Vault' }}</span>
    </div>
    <div class="actions">
      <button class="primary" @click="$emit('newItem')">
        <span class="plus">+</span> 新建条目
      </button>
      <div class="sep" />
      <button class="icon" @click="themeStore.toggle()" :title="themeStore.mode === 'light' ? '深色模式' : '浅色模式'">
        {{ themeStore.mode === 'light' ? '◑' : '◐' }}
      </button>
      <button class="icon" @click="$emit('settings')" title="设置">⚙</button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'

const repoStore = useRepoStore()
const themeStore = useThemeStore()

function basename(p: string): string {
  return p.split(/[/\\]/).pop() || p
}

defineEmits<{ newItem: []; settings: [] }>()
</script>

<style scoped>
.topbar {
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-4);
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  z-index: 10;
}

.left {
  display: flex; align-items: center; gap: var(--space-2);
  min-width: 0;
}
.logo {
  font-size: var(--font-size-xl); color: var(--accent);
  line-height: 1; flex-shrink: 0;
}
.repo-name {
  font-weight: var(--weight-semibold); font-size: var(--font-size-base);
  color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

.actions { display: flex; align-items: center; gap: var(--space-1); }
.plus { font-size: 16px; line-height: 0; margin-right: -2px; }
.sep { width: 1px; height: 20px; background: var(--border); margin: 0 var(--space-1); }
</style>
