import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PluginManifest } from '@/types/bindings'

const VALID_EXTENDS = ['center-panel', 'right-panel', 'sidebar']
const TIMEOUT_MS = 10_000

// Module-level singletons — shared across all callers
const _loadedCache = ref<Map<string, LoadResult>>(new Map())
const _failedPlugins = ref<Map<string, string>>(new Map()) // name → error message

export interface LoadResult {
  component: any
  manifest: PluginManifest
}

export function usePluginLoader() {
  const loadedCache = _loadedCache
  const failedPlugins = _failedPlugins

  async function loadPlugin(pluginName: string): Promise<LoadResult> {
    // Return cached result
    if (loadedCache.value.has(pluginName)) {
      return loadedCache.value.get(pluginName)!
    }
    // Check if previously failed — throw with the stored reason
    if (failedPlugins.value.has(pluginName)) {
      throw new Error(failedPlugins.value.get(pluginName)!)
    }

    const raceResult = await Promise.race([
      doLoad(pluginName),
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('加载超时')), TIMEOUT_MS)
      ),
    ])

    return raceResult
  }

  async function doLoad(pluginName: string): Promise<LoadResult> {
    // 1. Read and validate manifest
    let manifest: PluginManifest
    try {
      const raw = await invoke<string>('read_plugin_file', {
        pluginName,
        filename: 'manifest.json',
      })
      manifest = JSON.parse(raw)
      if (manifest.name !== pluginName) {
        throw new Error(`插件名不匹配: "${manifest.name}" !== "${pluginName}"`)
      }
      if (!VALID_EXTENDS.includes(manifest.extends)) {
        throw new Error(`无效的扩展点: "${manifest.extends}"`)
      }
    } catch (e: any) {
      const msg = `manifest.json 格式错误: ${e.message || e}`
      failedPlugins.value.set(pluginName, msg)
      throw new Error(msg)
    }

    // 2. Read plugin JS
    let jsCode: string
    try {
      jsCode = await invoke<string>('read_plugin_file', {
        pluginName,
        filename: 'index.js',
      })
    } catch {
      const msg = '无法读取插件文件，可能文件已损坏'
      failedPlugins.value.set(pluginName, msg)
      throw new Error(msg)
    }

    // 3. Execute JS
    let componentDef: any
    try {
      const moduleFn = new Function('exports', jsCode)
      const exports: any = {}
      moduleFn(exports)
      componentDef = exports.default || exports
    } catch (e: any) {
      const msg = `插件代码执行错误: ${e.message || e}`
      failedPlugins.value.set(pluginName, msg)
      throw new Error(msg)
    }

    // 3b. Factory function support
    if (typeof componentDef === 'function') {
      try {
        const { h, ref, computed, watch, onMounted } = await import('vue')
        componentDef = componentDef({ h, ref, computed, watch, onMounted })
      } catch (e: any) {
        const msg = `插件工厂函数执行错误: ${e.message || e}`
        failedPlugins.value.set(pluginName, msg)
        throw new Error(msg)
      }
    }

    // 4. Validate component
    if (typeof componentDef !== 'object' && typeof componentDef !== 'function') {
      const msg = '插件未导出有效的 Vue 组件'
      failedPlugins.value.set(pluginName, msg)
      throw new Error(msg)
    }

    const result: LoadResult = { component: componentDef, manifest }
    loadedCache.value.set(pluginName, result)
    return result
  }

  function clearError(pluginName: string) {
    failedPlugins.value.delete(pluginName)
  }

  return { loadPlugin, clearError, loadedCache, failedPlugins }
}
