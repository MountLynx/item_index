import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import { usePluginLoader } from '@/composables/usePluginLoader'
import type { WorkspaceConfig, WorkspaceSummary } from '@/types/bindings'

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<WorkspaceSummary[]>([])
  const activeName = ref('default')
  const activeConfig = ref<WorkspaceConfig | null>(null)
  const loading = ref(false)

  const { loadPlugin, failedPlugins, loadedCache } = usePluginLoader()

  const active = computed(() => activeConfig.value)

  async function loadAll(): Promise<void> {
    loading.value = true
    try {
      workspaces.value = await invoke<WorkspaceSummary[]>('list_workspaces')
      const def = workspaces.value.find(w => w.is_default)
      if (def) {
        await activate(def.name)
      } else if (workspaces.value.length > 0) {
        await activate(workspaces.value[0].name)
      }
    } finally {
      loading.value = false
    }
  }

  async function activate(name: string): Promise<void> {
    activeName.value = name
    const cfg = await invoke<WorkspaceConfig>('read_workspace', { name })
    activeConfig.value = cfg

    // Save active workspace to state.json
    try {
      await invoke('save_state', { newState: { active_workspace: name } })
    } catch { /* state.json save optional */ }

    // Resolve item type names → ids
    const typeStore = useTypeStore()
    let typeIds: number[] | null = null
    if (cfg.itemTypes.length > 0) {
      typeIds = cfg.itemTypes
        .map(tn => typeStore.types.find(t => t.name === tn)?.id)
        .filter((id): id is number => id !== undefined)
    }

    // Refresh items with type filter
    const itemStore = useItemStore()
    await itemStore.fetchList(null, null, typeIds && typeIds.length > 0 ? typeIds : undefined)

    // Preload all plugins in centerTabs
    for (const tab of cfg.centerTabs) {
      if (tab.type === 'plugin' && tab.plugin) {
        await loadPlugin(tab.plugin)
      }
    }
  }

  async function save(config: WorkspaceConfig): Promise<void> {
    await invoke('write_workspace', { name: config.name, config })
    await loadAll()
  }

  async function remove(name: string): Promise<void> {
    await invoke('delete_workspace', { name })
    await loadAll()
  }

  async function installFromPreset(presetName: string): Promise<void> {
    await invoke<WorkspaceConfig>('install_preset', { presetName })
    await loadAll()
  }

  async function exportAsPreset(name: string): Promise<void> {
    await invoke('export_preset', { name })
  }

  return {
    workspaces, activeName, activeConfig, active, loading,
    loadAll, activate, save, remove, installFromPreset, exportAsPreset,
    loadedPlugins: loadedCache, failedPlugins,
  }
})
