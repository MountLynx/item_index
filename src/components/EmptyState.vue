<template>
  <div class="welcome">
    <div class="card">
      <TablerIcon name="database" :size="40" :stroke="1.5" class="logo" />
      <h1>{{ $t('emptyState.title') }}</h1>
      <p>{{ $t('emptyState.tagline') }}</p>
      <div class="btns">
        <button class="primary" @click="openRepo"><TablerIcon name="folder-open" :size="17" /> {{ $t('emptyState.openRepo') }}</button>
        <button @click="showCreate = !showCreate"><TablerIcon name="plus" :size="17" /> {{ $t('emptyState.createRepo') }}</button>
      </div>
      <div v-if="showCreate" class="create">
        <input v-model="path" :placeholder="$t('emptyState.selectFolder')" @keydown.enter="doCreate" />
        <div class="create-btns">
          <button class="ghost" @click="showCreate = false">{{ $t('itemDialog.cancel') }}</button>
          <button class="primary" @click="doCreate">{{ $t('itemDialog.create') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRepoStore } from '@/stores/repo'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()

const repoStore = useRepoStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()
const showCreate = ref(false)
const path = ref('')
const emit = defineEmits<{ repoOpened: [] }>()

async function openRepo() { const p = prompt(t('emptyState.selectFolder')); if (p) { await repoStore.openRepo(p); await load(); emit('repoOpened') } }
async function doCreate() { if (path.value) { await repoStore.createRepo(path.value); await load(); emit('repoOpened') } }
async function load() { await Promise.all([typeStore.fetchAll(), groupStore.fetchAll(), tagStore.fetchAll(), itemStore.fetchList()]) }
</script>

<style scoped>
.welcome { display: flex; align-items: center; justify-content: center; height: 100vh; background: var(--bg); }
.card { text-align: center; padding: 48px; max-width: 400px; width: 100%; }
.logo { color: var(--accent); margin-bottom: 12px; }
h1 { font-size: var(--fs-2xl); font-weight: var(--fw-bold); margin: 0 0 4px; }
p { font-size: var(--fs-base); color: var(--text-secondary); margin: 0 0 24px; }
.btns { display: flex; gap: 8px; justify-content: center; margin-bottom: 16px; }
.create { text-align: left; }
.create input { width: 100%; margin-bottom: 8px; }
.create-btns { display: flex; gap: 8px; justify-content: flex-end; }
</style>
