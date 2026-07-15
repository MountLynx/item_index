<template>
  <div class="plugins-panel">
    <!-- Upper: Global Plugin Library -->
    <div class="pp-section">
      <div class="pp-section-header">
        <h3>全局插件库</h3>
        <div class="pp-section-actions">
          <button class="sm" @click="showMarketplace = true">插件市场</button>
          <button class="sm primary" @click="importPlugin" :disabled="importing">
            {{ importing ? '导入中…' : '＋ 导入' }}
          </button>
        </div>
      </div>
      <div class="pp-search">
        <input v-model="searchQuery" placeholder="搜索插件…" class="search-input" />
      </div>
      <div class="pp-scroll">
        <div v-if="loading" class="pp-loading">加载中…</div>
        <div v-else-if="filteredGlobal.length === 0" class="pp-empty">
          {{ searchQuery ? '无匹配插件' : '暂无插件。从插件市场获取或点击「＋ 导入」' }}
        </div>
        <div v-for="p in filteredGlobal" :key="p.name" class="pp-card">
          <div class="pp-card-icon"><TablerIcon :name="p.icon" :size="22" /></div>
          <div class="pp-card-info">
            <div class="pp-card-name">{{ p.title }} <span class="pp-version">v{{ p.version }}</span></div>
            <div class="pp-card-author" v-if="p.author">{{ p.author }}</div>
            <div class="pp-card-desc" v-if="p.description">{{ p.description }}</div>
            <div class="pp-card-meta">{{ p.extends }}<span v-if="p.requiresFields.length"> · 字段: {{ p.requiresFields.join(', ') }}</span></div>
          </div>
          <div class="pp-card-actions">
            <button v-if="installedInRepo.has(p.name)" class="sm" disabled>✓ 已安装</button>
            <button v-else class="sm" @click="installToRepo(p.name)" :disabled="installing === p.name">{{ installing === p.name ? '…' : '安装到仓库' }}</button>
            <button class="sm danger" @click="deleteGlobal(p.name)" :disabled="deleting === p.name">{{ deleting === p.name ? '…' : '✕' }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Lower: Repo Installed Plugins -->
    <div class="pp-section">
      <div class="pp-section-header">
        <h3>仓库已安装 ({{ repoPlugins.length }})</h3>
      </div>
      <div class="pp-scroll">
        <div v-if="repoPlugins.length === 0" class="pp-empty">暂无已安装插件。从上方全局库安装，或打开插件市场获取。</div>
        <div v-for="p in repoPlugins" :key="p.name" class="pp-card">
          <div class="pp-card-icon"><TablerIcon :name="p.icon" :size="22" /></div>
          <div class="pp-card-info">
            <div class="pp-card-name">{{ p.title }} <span class="pp-version">v{{ p.version }}</span></div>
            <div class="pp-card-usage" v-if="repoUsage[p.name]">
              <span v-if="repoUsage[p.name]!.workspaces.length">被工作区使用: {{ repoUsage[p.name]!.workspaces.join(', ') }}</span>
              <span v-else class="unused">未被任何工作区使用</span>
            </div>
          </div>
          <div class="pp-card-actions">
            <button class="sm danger" @click="uninstallFromRepo(p.name)" :disabled="uninstalling === p.name">{{ uninstalling === p.name ? '…' : '卸载' }}</button>
          </div>
        </div>
      </div>
    </div>

    <PluginMarketplace v-if="showMarketplace" @close="showMarketplace = false" @installed="onMarketplaceInstalled" />

    <PluginDetail
      v-if="importPreview"
      :manifest="importPreview.manifest"
      :usage="importPreview.usage"
      :mode="importPreview.exists ? 'overwrite' : 'import'"
      @confirm="confirmImport"
      @cancel="importPreview = null"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { open, ask, message } from '@tauri-apps/plugin-dialog'
import { useWorkspaceStore } from '@/stores/workspace'
import type { PluginManifest, PluginUsage } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'
import PluginMarketplace from './PluginMarketplace.vue'
import PluginDetail from './PluginDetail.vue'

const wsStore = useWorkspaceStore()

const plugins = ref<PluginManifest[]>([])
const repoPlugins = ref<PluginManifest[]>([])
const repoUsage = ref<Record<string, { workspaces: string[] }>>({})
const loading = ref(false)
const importing = ref(false)
const installing = ref<string | null>(null)
const deleting = ref<string | null>(null)
const uninstalling = ref<string | null>(null)
const searchQuery = ref('')
const showMarketplace = ref(false)

interface ImportPreview {
  manifest: PluginManifest
  usage: PluginUsage | null
  exists: boolean
  sourcePath: string
}
const importPreview = ref<ImportPreview | null>(null)

const installedInRepo = computed(() => new Set(repoPlugins.value.map(p => p.name)))

const filteredGlobal = computed(() => {
  if (!searchQuery.value.trim()) return plugins.value
  const q = searchQuery.value.toLowerCase()
  return plugins.value.filter(p =>
    p.title.toLowerCase().includes(q) ||
    p.name.toLowerCase().includes(q) ||
    (p.author && p.author.toLowerCase().includes(q)) ||
    (p.description && p.description.toLowerCase().includes(q))
  )
})

async function loadAll() {
  loading.value = true
  try {
    const [global, repo] = await Promise.all([
      wsStore.listGlobalPlugins().catch(() => [] as PluginManifest[]),
      wsStore.listInstalledPlugins().catch(() => [] as PluginManifest[]),
    ])
    plugins.value = global
    repoPlugins.value = repo

    for (const rp of repo) {
      try {
        const usage = await wsStore.checkPluginUsage(rp.name).catch(() => null)
        const workspaces = usage && usage.repos.length > 0 ? ['已使用'] : []
        repoUsage.value[rp.name] = { workspaces }
      } catch {
        repoUsage.value[rp.name] = { workspaces: [] }
      }
    }
  } finally {
    loading.value = false
  }
}

async function importPlugin() {
  const selected = await open({ directory: true, multiple: false, title: '选择插件文件夹' })
  if (!selected) return
  importing.value = true
  try {
    await wsStore.importPluginToGlobal(selected as string)
    await loadAll()
    await message('插件已导入到全局库', { title: '导入成功', kind: 'info' })
  } catch (e: any) {
    const errStr = String(e)
    if (errStr.includes('already exists')) {
      const dirName = (selected as string).split(/[/\\]/).pop() || ''
      const usage = await wsStore.checkPluginUsage(dirName).catch(() => null)
      importPreview.value = {
        manifest: plugins.value.find(p => p.name === dirName) || { name: dirName, version: '?', title: dirName, icon: 'puzzle', extends: 'center-panel', requiresFields: [], author: '', description: '' },
        usage,
        exists: true,
        sourcePath: selected as string,
      }
      return
    }
    await message(`导入失败: ${e}`, { title: '导入插件', kind: 'error' })
  } finally {
    importing.value = false
  }
}

async function confirmImport() {
  if (!importPreview.value) return
  try {
    const name = importPreview.value.manifest.name
    if (importPreview.value.exists) {
      await wsStore.deleteGlobalPlugin(name)
    }
    await wsStore.importPluginToGlobal(importPreview.value.sourcePath)
    await loadAll()
    importPreview.value = null
    await message('插件已导入', { title: '导入成功', kind: 'info' })
  } catch (e) {
    await message(`导入失败: ${e}`, { title: '导入插件', kind: 'error' })
  }
}

async function installToRepo(pluginName: string) {
  installing.value = pluginName
  try {
    await wsStore.installPlugin(pluginName)
    repoPlugins.value = await wsStore.listInstalledPlugins()
  } catch (e) {
    await message(`安装失败: ${e}`, { title: '安装插件', kind: 'error' })
  } finally {
    installing.value = null
  }
}

async function deleteGlobal(pluginName: string) {
  let usage: PluginUsage | null = null
  try { usage = await wsStore.checkPluginUsage(pluginName) } catch {}

  let warnMsg = `确定从全局库删除插件 "${pluginName}"？`
  if (usage && (usage.repos.length > 0 || usage.presets.length > 0)) {
    warnMsg += '\n\n⚠ 此插件仍被以下引用：'
    if (usage.repos.length > 0) warnMsg += `\n仓库: ${usage.repos.join(', ')}`
    if (usage.presets.length > 0) warnMsg += `\n预设: ${usage.presets.join(', ')}`
    warnMsg += '\n\n删除后相关预设将无法重新安装此插件。'
  }

  const ok = await ask(warnMsg, { title: '删除插件', kind: 'warning' })
  if (!ok) return

  deleting.value = pluginName
  try {
    await wsStore.deleteGlobalPlugin(pluginName)
    await loadAll()
  } catch (e) {
    await message(`删除失败: ${e}`, { title: '删除插件', kind: 'error' })
  } finally {
    deleting.value = null
  }
}

async function uninstallFromRepo(pluginName: string) {
  const ok = await ask(
    `确定从仓库卸载插件 "${pluginName}"？\n\n如果工作区正在使用此插件，卸载将被拒绝。`,
    { title: '卸载插件', kind: 'warning' }
  )
  if (!ok) return

  uninstalling.value = pluginName
  try {
    await wsStore.uninstallPlugin(pluginName)
    repoPlugins.value = await wsStore.listInstalledPlugins()
  } catch (e) {
    await message(`卸载失败: ${e}`, { title: '卸载插件', kind: 'error' })
  } finally {
    uninstalling.value = null
  }
}

function onMarketplaceInstalled() {
  showMarketplace.value = false
  loadAll()
}

onMounted(loadAll)
defineExpose({ loadAll })
</script>

<style scoped>
.plugins-panel { display: flex; flex-direction: column; height: 100%; gap: 0; }
.pp-section { flex: 1; display: flex; flex-direction: column; min-height: 0; border-bottom: 1px solid var(--border); }
.pp-section:last-child { border-bottom: none; }
.pp-section-header { display: flex; align-items: center; justify-content: space-between; padding: 8px 0; flex-shrink: 0; }
.pp-section-header h3 { margin: 0; font-size: var(--fs-sm); font-weight: var(--fw-semibold); }
.pp-section-actions { display: flex; gap: 6px; }
.pp-search { padding-bottom: 8px; flex-shrink: 0; }
.search-input { width: 100%; height: 30px; padding: 0 10px; font-size: var(--fs-sm); border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); color: var(--text); box-sizing: border-box; }
.search-input:focus { outline: none; border-color: var(--accent); }
.pp-scroll { flex: 1; overflow-y: auto; min-height: 0; }
.pp-loading, .pp-empty { text-align: center; color: var(--text-muted); font-size: var(--fs-sm); padding: 20px 0; }
.pp-card { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); margin-bottom: 6px; }
.pp-card-icon { flex-shrink: 0; color: var(--text-secondary); margin-top: 1px; }
.pp-card-info { flex: 1; min-width: 0; }
.pp-card-name { font-size: var(--fs-sm); font-weight: var(--fw-medium); color: var(--text); }
.pp-version { font-size: var(--fs-xs); color: var(--text-muted); font-weight: var(--fw-normal); }
.pp-card-author { font-size: var(--fs-xs); color: var(--text-secondary); margin-top: 1px; }
.pp-card-desc { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.pp-card-meta { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 2px; }
.pp-card-usage { font-size: var(--fs-xs); color: var(--text-secondary); margin-top: 2px; }
.pp-card-usage .unused { color: var(--warning); }
.pp-card-actions { display: flex; gap: 4px; flex-shrink: 0; align-items: center; }
.sm { font-size: var(--fs-xs); height: 26px; padding: 0 8px; }
.danger { color: var(--danger); }
</style>
