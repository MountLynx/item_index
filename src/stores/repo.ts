import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { RepoInfo } from '@/types/bindings'
import { useDashboardStore } from './dashboard'

export const useRepoStore = defineStore('repo', () => {
  const repoPath = ref<string | null>(null)
  const itemCount = ref(0)
  const isOpen = computed(() => repoPath.value !== null)

  async function createRepo(path: string): Promise<RepoInfo> {
    const info = await invoke<RepoInfo>('create_repo', { path })
    repoPath.value = info.path
    itemCount.value = info.item_count
    // Auto-record in dashboard
    try { await useDashboardStore().addRepo(info.path) } catch { /* ignore */ }
    return info
  }

  async function openRepo(path: string, skipDashboard = false): Promise<RepoInfo> {
    const info = await invoke<RepoInfo>('open_repo', { path })
    repoPath.value = info.path
    itemCount.value = info.item_count
    // Auto-record in dashboard with current item count, unless this is a sub-repo
    if (!skipDashboard) {
      try { await useDashboardStore().addRepo(info.path, undefined, info.item_count) } catch { /* ignore */ }
    }
    return info
  }

  async function closeRepo(): Promise<void> {
    await invoke('close_repo')
    repoPath.value = null
    itemCount.value = 0
  }

  async function refresh(): Promise<void> {
    try {
      const info = await invoke<RepoInfo>('get_repo_info')
      repoPath.value = info.path
      itemCount.value = info.item_count
    } catch {
      // repo not open — ignore
    }
  }

  return { repoPath, itemCount, isOpen, createRepo, openRepo, closeRepo, refresh }
})
