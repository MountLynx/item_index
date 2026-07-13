import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ManagedRepo } from '@/types/bindings'

export const useDashboardStore = defineStore('dashboard', () => {
  const repos = ref<ManagedRepo[]>([])
  const loading = ref(false)

  async function fetchAll() {
    loading.value = true
    try {
      repos.value = await invoke<ManagedRepo[]>('list_managed_repos')
    } catch {
      repos.value = []
    } finally {
      loading.value = false
    }
  }

  async function addRepo(path: string, icon?: string, itemCount?: number) {
    repos.value = await invoke<ManagedRepo[]>('add_managed_repo', {
      path,
      icon: icon ?? null,
      name: null,
      itemCount: itemCount ?? null,
    })
  }

  async function removeRepo(path: string) {
    repos.value = await invoke<ManagedRepo[]>('remove_managed_repo', { path })
  }

  async function updateIcon(path: string, icon: string) {
    repos.value = await invoke<ManagedRepo[]>('update_repo_icon', { path, icon })
  }

  return { repos, loading, fetchAll, addRepo, removeRepo, updateIcon }
})
