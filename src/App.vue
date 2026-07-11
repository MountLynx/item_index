<template>
  <div class="app" :class="{ dark: themeStore.mode === 'dark' }">
    <EmptyState v-if="!repoStore.isOpen" @repo-opened="onRepoOpened" />

    <template v-else>
      <Topbar @new-item="showNewItem = true" @settings="showSettings = true" />
      <div class="main">
        <Sidebar />
        <CenterList @new-item="showNewItem = true" />
        <RightPanel />
      </div>
      <StatusBar />
    </template>

    <NewItemDialog v-if="showNewItem" @close="showNewItem = false" />
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
    <Toast ref="toastRef" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import EmptyState from '@/components/EmptyState.vue'
import Topbar from '@/components/Topbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import CenterList from '@/components/CenterList.vue'
import RightPanel from '@/components/RightPanel.vue'
import StatusBar from '@/components/StatusBar.vue'
import NewItemDialog from '@/components/NewItemDialog.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'
import Toast from '@/components/Toast.vue'

const repoStore = useRepoStore()
const themeStore = useThemeStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()

const showNewItem = ref(false)
const showSettings = ref(false)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

async function onRepoOpened() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
  ])
}
</script>

<style>
@import '@/assets/theme.css';
</style>

<style scoped>
.app { height: 100vh; display: flex; flex-direction: column; }
.main { display: flex; flex: 1; overflow: hidden; }
</style>
