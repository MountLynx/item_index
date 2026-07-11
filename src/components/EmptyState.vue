<template>
  <div class="empty-state">
    <h1>Vault</h1>
    <p>本地优先的对象管理器</p>
    <div class="buttons">
      <button class="primary" @click="openRepo">📂 打开仓库</button>
      <button class="primary" @click="createRepo">✨ 创建仓库</button>
    </div>
    <div v-if="showCreate" class="create-form">
      <input v-model="path" type="text" placeholder="仓库路径，如 C:\Users\me\MyVault" />
      <button class="primary" @click="doCreate">创建</button>
      <button @click="showCreate = false">取消</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'

const repoStore = useRepoStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()

const showCreate = ref(false)
const path = ref('')

const emit = defineEmits<{ repoOpened: [] }>()

async function openRepo() {
  // In Tauri, use the dialog plugin; for now prompt
  const p = prompt('输入仓库路径:')
  if (p) {
    await repoStore.openRepo(p)
    await loadAll()
    emit('repoOpened')
  }
}

async function createRepo() {
  showCreate.value = true
  path.value = ''
}

async function doCreate() {
  if (path.value) {
    await repoStore.createRepo(path.value)
    await loadAll()
    emit('repoOpened')
  }
}

async function loadAll() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
  ])
}
</script>

<style scoped>
.empty-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 100vh; gap: 16px;
}
h1 { font-size: 32px; }
.buttons { display: flex; gap: 12px; }
.create-form { display: flex; gap: 8px; margin-top: 12px; }
.create-form input { width: 400px; padding: 8px 12px; }
</style>
