<template>
  <div class="dashboard">
    <div class="dash-content">
      <div class="dash-header">
        <TablerIcon name="database" :size="36" :stroke="1.5" class="dash-logo" />
        <h1>{{ $t('dashboard.title') }}</h1>
        <p>{{ $t('dashboard.tagline') }}</p>
      </div>

      <div class="dash-grid" v-if="!loading">
        <RepoCard
          v-for="repo in store.repos"
          :key="repo.path"
          :repo="repo"
          @open="openRepo(repo)"
          @delete="removeRepo(repo)"
        />
        <RepoCreateTile
          @create="doCreate"
          @import="doImport"
        />
      </div>
      <div class="dash-loading" v-else>
        <p class="text-muted">...</p>
      </div>

      <TemplateBanner />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { useDashboardStore } from '@/stores/dashboard'
import { useRepoStore } from '@/stores/repo'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import { useSettingsStore } from '@/stores/settings'
import type { ManagedRepo } from '@/types/bindings'
import RepoCard from './RepoCard.vue'
import RepoCreateTile from './RepoCreateTile.vue'
import TemplateBanner from './TemplateBanner.vue'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()
const store = useDashboardStore()
const repoStore = useRepoStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()
const settingsStore = useSettingsStore()

const { loading } = store
const emit = defineEmits<{ repoOpened: [] }>()

onMounted(() => {
  store.fetchAll()
})

async function openRepo(repo: ManagedRepo) {
  try {
    await repoStore.openRepo(repo.path)
    // Update last_opened_at and item_count in background
    await store.addRepo(repo.path, repo.icon ?? undefined)
    await loadStores()
    emit('repoOpened')
  } catch {
    alert(t('emptyState.openFailed'))
  }
}

async function removeRepo(repo: ManagedRepo) {
  await store.removeRepo(repo.path)
}

async function doCreate() {
  const selected = await open({ directory: true, multiple: false, title: t('dashboard.selectFolder') })
  if (!selected) return
  try {
    await repoStore.createRepo(selected)
    await store.addRepo(selected)
    await loadStores()
    emit('repoOpened')
  } catch {
    alert(t('emptyState.createFailed'))
  }
}

async function doImport() {
  const selected = await open({ directory: true, multiple: false, title: t('dashboard.selectFolder') })
  if (!selected) return
  // Verify it has .index/index.db
  try {
    await repoStore.openRepo(selected)
    // It opened successfully — close it and add to managed list
    await repoStore.closeRepo()
    await store.addRepo(selected)
  } catch {
    alert(t('dashboard.notAValidRepo'))
  }
}

async function loadStores() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
    itemStore.fetchList(),
  ])
  await settingsStore.loadActivePresetFromRepo()
  settingsStore.applyTheme()
}
</script>

<style scoped>
.dashboard {
  display: flex; align-items: center; justify-content: center;
  height: 100vh; background: var(--bg);
  user-select: none;
}
.dash-content {
  display: flex; flex-direction: column; align-items: center;
  gap: 32px;
  padding: 48px 32px;
  max-width: 880px;
  width: 100%;
}
.dash-header {
  text-align: center;
}
.dash-logo {
  color: var(--accent); margin-bottom: 8px;
}
.dash-header h1 {
  font-size: var(--fs-2xl); font-weight: var(--fw-bold);
  margin: 0 0 4px; color: var(--text-heading);
}
.dash-header p {
  font-size: var(--fs-base); color: var(--text-secondary);
  margin: 0;
}
.dash-grid {
  display: flex; flex-wrap: wrap; gap: 16px;
  justify-content: center;
}
.dash-loading {
  display: flex; align-items: center; justify-content: center;
  min-height: 172px;
}
</style>
