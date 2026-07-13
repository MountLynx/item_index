<template>
  <div class="app">
    <EmptyState v-if="!repoStore.isOpen" @repo-opened="onRepoOpened" />

    <template v-else>
      <Titlebar @new-item="showNewItem = true" @open-type-manager="rightTab = 'types'" />
      <div class="main">
        <Sidebar />
        <CenterList @new-item="showNewItem = true" />
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
import { provide, ref, onMounted } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'
import { useSettingsStore } from '@/stores/settings'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import EmptyState from '@/components/EmptyState.vue'
import Titlebar from '@/components/Titlebar.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import Sidebar from '@/components/Sidebar.vue'
import CenterList from '@/components/CenterList.vue'
import RightPanel from '@/components/RightPanel.vue'
import StatusBar from '@/components/StatusBar.vue'
import NewItemDialog from '@/components/NewItemDialog.vue'
import Toast from '@/components/Toast.vue'

const repoStore = useRepoStore()
const themeStore = useThemeStore()
const settingsStore = useSettingsStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()

const showNewItem = ref(false)
const rightTab = ref<'detail' | 'types'>('detail')
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
const settingsRef = ref<InstanceType<typeof SettingsModal> | null>(null)
provide('openSettings', () => settingsRef.value?.open())

async function onRepoOpened() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
    itemStore.fetchList(),
  ])
  await settingsStore.loadActivePresetFromRepo()
  settingsStore.applyTheme()
}

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
