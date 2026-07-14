import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PluginManifest } from '@/types/bindings'

const VALID_EXTENDS = ['center-panel', 'right-panel', 'sidebar']

export interface LoadResult {
  component: any          // Vue component or null
  manifest: PluginManifest
}

export function usePluginLoader() {
  const loadedCache = ref<Map<string, LoadResult>>(new Map())
  const failedPlugins = ref<Set<string>>(new Set())

  async function loadPlugin(pluginName: string): Promise<LoadResult | null> {
    // Return cached result (success or failure)
    if (loadedCache.value.has(pluginName)) {
      return loadedCache.value.get(pluginName)!
    }
    if (failedPlugins.value.has(pluginName)) {
      return null
    }

    // 1. Read and validate manifest
    const manifest = await validateManifest(pluginName)
    if (!manifest) {
      failedPlugins.value.add(pluginName)
      return null
    }

    // 2. Read plugin JS
    let jsCode: string
    try {
      jsCode = await invoke<string>('read_plugin_file', { pluginName, filename: 'index.js' })
    } catch {
      console.error(`[PluginLoader] Cannot read index.js for "${pluginName}"`)
      failedPlugins.value.add(pluginName)
      return null
    }

    // 3. Execute JS to get component definition
    let component: any
    try {
      const moduleFn = new Function('exports', jsCode)
      const exports: any = {}
      moduleFn(exports)
      component = exports.default || exports
    } catch (e) {
      console.error(`[PluginLoader] Failed to execute "${pluginName}":`, e)
      failedPlugins.value.add(pluginName)
      return null
    }

    // 4. Validate component has setup/render
    if (typeof component !== 'object' && typeof component !== 'function') {
      console.error(`[PluginLoader] "${pluginName}" does not export a valid component`)
      failedPlugins.value.add(pluginName)
      return null
    }

    const result: LoadResult = { component, manifest }
    loadedCache.value.set(pluginName, result)
    return result
  }

  async function validateManifest(pluginName: string): Promise<PluginManifest | null> {
    try {
      const raw = await invoke<string>('read_plugin_file', { pluginName, filename: 'manifest.json' })
      const m: PluginManifest = JSON.parse(raw)

      // Name must match directory
      if (m.name !== pluginName) {
        console.error(`[PluginLoader] Manifest name "${m.name}" != directory "${pluginName}"`)
        return null
      }
      // extends must be valid
      if (!VALID_EXTENDS.includes(m.extends)) {
        console.error(`[PluginLoader] Invalid extends "${m.extends}"`)
        return null
      }
      return m
    } catch {
      return null
    }
  }

  return { loadPlugin, validateManifest, loadedCache, failedPlugins }
}
