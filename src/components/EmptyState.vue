<template>
  <div class="welcome">
    <div class="welcome-card">
      <div class="logo-mark">◆</div>
      <h1 class="title">Vault</h1>
      <p class="subtitle">本地优先的对象管理器</p>
      <div class="actions">
        <button class="primary" @click="openRepo">📂 打开仓库</button>
        <button @click="showCreate = !showCreate">✨ 创建仓库</button>
      </div>
      <div v-if="showCreate" class="create-panel">
        <input v-model="path" placeholder="仓库路径，如 C:\Users\me\MyVault" @keydown.enter="doCreate" />
        <div class="create-actions">
          <button class="primary" @click="doCreate">创建</button>
          <button class="ghost" @click="showCreate = false">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'

const repoStore = useRepoStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()

const showCreate = ref(false)
const path = ref('')
const emit = defineEmits<{ repoOpened: [] }>()

async function openRepo() { const p = prompt('输入仓库路径:'); if (p) { await repoStore.openRepo(p); await loadAll(); emit('repoOpened') } }
async function doCreate() { if (path.value) { await repoStore.createRepo(path.value); await loadAll(); emit('repoOpened') } }
async function loadAll() { await Promise.all([typeStore.fetchAll(), groupStore.fetchAll(), tagStore.fetchAll(), itemStore.fetchList()]) }
</script>

<style scoped>
.welcome {
  display: flex; align-items: center; justify-content: center; height: 100vh;
  background: var(--bg);
}
.welcome-card {
  text-align: center; padding: var(--space-10); border-radius: var(--radius-2xl);
  background: var(--surface); border: 1px solid var(--border-light);
  box-shadow: var(--shadow-md); max-width: 420px; width: 100%;
}
.logo-mark { font-size: 48px; color: var(--accent); margin-bottom: var(--space-4); line-height: 1; }
.title { font-size: var(--font-size-2xl); font-weight: var(--weight-bold); color: var(--text-primary); margin: 0 0 var(--space-1); }
.subtitle { font-size: var(--font-size-base); color: var(--text-secondary); margin: 0 0 var(--space-6); }
.actions { display: flex; gap: var(--space-2); justify-content: center; margin-bottom: var(--space-4); }
.create-panel { text-align: left; }
.create-panel input { width: 100%; margin-bottom: var(--space-2); }
.create-actions { display: flex; gap: var(--space-2); justify-content: flex-end; }
</style>
