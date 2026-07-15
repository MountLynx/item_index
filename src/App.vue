<template>
  <div class="app">
    <RepoDashboard v-if="!repoStore.isOpen" />

    <template v-else>
      <Titlebar @new-item="showNewItem = true" @open-type-manager="rightTab = 'types'" />
      <div class="main">
        <Sidebar />
        <CenterPanel @new-item="showNewItem = true" />
        <RightPanel v-model:activeTab="rightTab" />
      </div>
      <StatusBar />
    </template>

    <NewItemDialog v-if="showNewItem" @close="showNewItem = false" />
    <Toast ref="toastRef" />
    <SettingsModal ref="settingsRef" />
  </div>
</template>

<script setup lang="ts">
import { provide, ref, onMounted, watch } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'
import { useSettingsStore } from '@/stores/settings'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import RepoDashboard from '@/components/RepoDashboard.vue'
import Titlebar from '@/components/Titlebar.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import Sidebar from '@/components/Sidebar.vue'
import CenterPanel from '@/components/CenterPanel.vue'
import RightPanel from '@/components/RightPanel.vue'
import StatusBar from '@/components/StatusBar.vue'
import NewItemDialog from '@/components/NewItemDialog.vue'
import Toast from '@/components/Toast.vue'
import { useWorkspaceStore } from '@/stores/workspace'

const repoStore = useRepoStore()
const themeStore = useThemeStore()
const settingsStore = useSettingsStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()
const workspaceStore = useWorkspaceStore()

const showNewItem = ref(false)
const rightTab = ref<'detail' | 'types'>('detail')
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
const settingsRef = ref<InstanceType<typeof SettingsModal> | null>(null)
provide('openSettings', () => settingsRef.value?.open())

// Use watch instead of @repo-opened event to avoid timing issues where
// RepoDashboard is unmounted before the event reaches App.vue (Bug #9).
let repoOpenedHandled = false
watch(() => repoStore.isOpen, async (isOpen) => {
  if (!isOpen || repoOpenedHandled) return
  repoOpenedHandled = true
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
    itemStore.fetchList(),
  ])
  // Load workspaces after types are available
  await workspaceStore.loadAll()
  await settingsStore.loadActivePresetFromRepo()
  settingsStore.applyTheme()
})

onMounted(() => {
  settingsStore.load()
  themeStore.init()
  settingsStore.applyTheme()
})
</script>

<style>
@import '@/assets/theme.css';
</style>

<style scoped>
.app { height: 100vh; display: flex; flex-direction: column; }
.main { display: flex; flex: 1; overflow: hidden; }
</style>
