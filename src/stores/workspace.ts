import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useItemStore } from '@/stores/items'
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
        await activate(def.key)
      } else if (workspaces.value.length > 0) {
        await activate(workspaces.value[0].key)
      }
    } finally {
      loading.value = false
    }
  }

  async function activate(name: string): Promise<void> {
    activeName.value = name
    const cfg = await invoke<WorkspaceConfig>('read_workspace', { name })
    // Defensive defaults — older/partial configs may miss camelCase fields
    const itemTypes = cfg.itemTypes ?? []
    const centerTabs = cfg.centerTabs ?? []
    const defaultTab = cfg.defaultTab ?? ''
    activeConfig.value = { ...cfg, itemTypes, centerTabs, defaultTab }

    // Save active workspace to state.json
    try {
      await invoke('save_state', { newState: { active_workspace: name } })
    } catch { /* state.json save optional */ }

    // Refresh items — fetchList auto-resolves typeIds from active workspace config
    const itemStore = useItemStore()
    await itemStore.fetchList()

    // Preload all plugins in centerTabs
    for (const tab of centerTabs) {
      if (tab.type === 'plugin' && tab.plugin) {
        try {
          await loadPlugin(tab.plugin)
        } catch {
          // Plugin failed to load — error already stored in failedPlugins map
        }
      }
    }
  }

  async function save(config: WorkspaceConfig, originalKey?: string): Promise<void> {
    // If renamed, delete old file first (Bug #3)
    if (originalKey && originalKey !== config.name) {
      try { await invoke('delete_workspace', { name: originalKey }) } catch {}
    }
    await invoke('write_workspace', { name: config.name, config })
    await loadAll()
  }

  async function remove(key: string): Promise<void> {
    await invoke('delete_workspace', { name: key })
    await loadAll()
  }

  async function installFromPreset(presetName: string): Promise<void> {
    const cfg = await invoke<WorkspaceConfig>('install_preset', { presetName })
    await loadAll()
    // Auto-activate the newly installed workspace
    await activate(cfg.name)
  }

  async function exportAsPreset(name: string): Promise<void> {
    await invoke('export_preset', { name })
  }

  async function listGlobalPlugins(): Promise<import('@/types/bindings').PluginManifest[]> {
    return await invoke<import('@/types/bindings').PluginManifest[]>('list_global_plugins')
  }

  async function listInstalledPlugins(): Promise<import('@/types/bindings').PluginManifest[]> {
    return await invoke<import('@/types/bindings').PluginManifest[]>('list_installed_plugins')
  }

  async function installPlugin(pluginName: string): Promise<void> {
    await invoke('install_plugin', { pluginName })
  }

  async function uninstallPlugin(pluginName: string): Promise<void> {
    await invoke('uninstall_plugin_from_repo', { pluginName })
  }

  async function checkPluginUsage(pluginName: string): Promise<import('@/types/bindings').PluginUsage> {
    return await invoke<import('@/types/bindings').PluginUsage>('check_plugin_usage', { pluginName })
  }

  async function deleteGlobalPlugin(pluginName: string): Promise<void> {
    await invoke('delete_plugin', { pluginName })
  }

  async function importPluginToGlobal(sourcePath: string): Promise<void> {
    await invoke('install_plugin_to_global', { sourcePath })
  }

  async function fetchMarketplaceIndex(): Promise<import('@/types/bindings').PluginIndex> {
    return await invoke<import('@/types/bindings').PluginIndex>('fetch_marketplace_index')
  }

  async function downloadMarketplacePlugin(url: string, sha256: string): Promise<void> {
    await invoke('download_marketplace_plugin', { url, expectedSha256: sha256 })
  }

  return {
    workspaces, activeName, activeConfig, active, loading,
    loadAll, activate, save, remove, installFromPreset, exportAsPreset,
    listGlobalPlugins, listInstalledPlugins, installPlugin, uninstallPlugin,
    checkPluginUsage, deleteGlobalPlugin, importPluginToGlobal,
    fetchMarketplaceIndex, downloadMarketplacePlugin,
    loadedPlugins: loadedCache, failedPlugins,
  }
})
