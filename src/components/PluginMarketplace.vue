<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="marketplace-modal">
      <div class="mp-header">
        <h3>插件市场</h3>
        <button class="icon-btn" @click="emit('close')"><TablerIcon name="x" :size="16" /></button>
      </div>
      <div class="mp-body">
        <div v-if="error" class="mp-error"><p>{{ error }}</p><button @click="fetchIndex">重试</button></div>
        <div v-else-if="loading" class="mp-loading">加载中…</div>
        <template v-else>
          <div class="mp-sidebar">
            <button v-for="ext in filterOptions" :key="ext.value" :class="{ active: activeFilter === ext.value }" @click="activeFilter = ext.value">{{ ext.label }}</button>
          </div>
          <div class="mp-list">
            <div v-for="entry in filteredEntries" :key="entry.name" class="mp-card">
              <div class="mp-card-icon"><TablerIcon :name="entry.icon" :size="24" /></div>
              <div class="mp-card-info">
                <div class="mp-card-name">{{ entry.title }} <span class="mp-version">v{{ entry.version }}</span></div>
                <div class="mp-card-author">{{ entry.author }}</div>
                <div class="mp-card-desc">{{ entry.description }}</div>
              </div>
              <div class="mp-card-actions">
                <span v-if="isInstalledLatest(entry)" class="mp-installed-badge">已安装</span>
                <span v-else-if="hasUpdate(entry)" class="mp-update-badge">可更新</span>
                <button class="sm primary" @click="install(entry)" :disabled="installing === entry.name">
                  {{ installing === entry.name ? '安装中…' : hasUpdate(entry) ? '更新' : '安装' }}
                </button>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { message, ask } from '@tauri-apps/plugin-dialog'
import { useWorkspaceStore } from '@/stores/workspace'
import type { PluginIndexEntry, PluginManifest } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const emit = defineEmits<{ close: []; installed: [] }>()
const wsStore = useWorkspaceStore()

const index = ref<PluginIndexEntry[]>([])
const globalPlugins = ref<PluginManifest[]>([])
const loading = ref(false)
const error = ref('')
const installing = ref<string | null>(null)
const activeFilter = ref('all')

const filterOptions = [
  { value: 'all', label: '全部' },
  { value: 'center-panel', label: '中间面板' },
  { value: 'right-panel', label: '右侧栏' },
  { value: 'sidebar', label: '侧边栏' },
]

const filteredEntries = computed(() => {
  if (activeFilter.value === 'all') return index.value
  return index.value.filter(e => e.extends === activeFilter.value)
})

function isInstalledLatest(entry: PluginIndexEntry) {
  const local = globalPlugins.value.find(p => p.name === entry.name)
  return local != null && local.version === entry.version
}

function hasUpdate(entry: PluginIndexEntry) {
  const local = globalPlugins.value.find(p => p.name === entry.name)
  return local != null && local.version !== entry.version
}

async function fetchIndex() {
  loading.value = true; error.value = ''
  try {
    const [idx, global] = await Promise.all([
      wsStore.fetchMarketplaceIndex(),
      wsStore.listGlobalPlugins().catch(() => [] as PluginManifest[]),
    ])
    index.value = idx.plugins
    globalPlugins.value = global
  } catch (e: any) {
    error.value = `获取插件列表失败: ${e}`
  } finally { loading.value = false }
}

async function install(entry: PluginIndexEntry) {
  const existing = globalPlugins.value.find(p => p.name === entry.name)
  if (existing && existing.version !== entry.version) {
    try {
      const usage = await wsStore.checkPluginUsage(entry.name)
      if (usage.repos.length > 0 || usage.presets.length > 0) {
        let msg = `全局库已存在 "${entry.name}" v${existing.version}，将覆盖为 v${entry.version}。`
        if (usage.repos.length > 0) msg += `\n\n受影响的仓库: ${usage.repos.join(', ')}`
        if (usage.presets.length > 0) msg += `\n受影响的预设: ${usage.presets.join(', ')}`
        const ok = await ask(msg, { title: '覆盖插件', kind: 'warning' })
        if (!ok) return
      }
    } catch {}
  }

  installing.value = entry.name
  try {
    await wsStore.downloadMarketplacePlugin(entry.downloadUrl, entry.sha256)
    emit('installed')
    await message(`"${entry.title}" 安装成功`, { title: '插件市场', kind: 'info' })
  } catch (e) {
    await message(`安装失败: ${e}`, { title: '插件市场', kind: 'error' })
  } finally { installing.value = null }
}

onMounted(fetchIndex)
</script>

<style scoped>
.modal-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; }
.marketplace-modal { width: 680px; max-height: 80vh; background: var(--bg); border-radius: var(--r-lg); box-shadow: 0 8px 32px rgba(0,0,0,0.2); display: flex; flex-direction: column; overflow: hidden; }
.mp-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.mp-header h3 { margin: 0; font-size: var(--fs-base); font-weight: var(--fw-semibold); }
.mp-body { flex: 1; display: flex; min-height: 0; overflow: hidden; }
.mp-error, .mp-loading { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-muted); font-size: var(--fs-sm); }
.mp-sidebar { width: 120px; flex-shrink: 0; display: flex; flex-direction: column; gap: 2px; padding: 12px 8px; border-right: 1px solid var(--border); }
.mp-sidebar button { text-align: left; padding: 6px 10px; font-size: var(--fs-sm); border: none; border-radius: var(--r-sm); background: transparent; color: var(--text-secondary); cursor: pointer; }
.mp-sidebar button:hover { background: var(--surface-hover); }
.mp-sidebar button.active { background: var(--surface-active); color: var(--text); font-weight: var(--fw-medium); }
.mp-list { flex: 1; overflow-y: auto; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
.mp-card { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); }
.mp-card-icon { flex-shrink: 0; color: var(--text-secondary); margin-top: 1px; }
.mp-card-info { flex: 1; min-width: 0; }
.mp-card-name { font-size: var(--fs-sm); font-weight: var(--fw-medium); color: var(--text); }
.mp-version { font-size: var(--fs-xs); color: var(--text-muted); font-weight: var(--fw-normal); }
.mp-card-author { font-size: var(--fs-xs); color: var(--text-secondary); margin-top: 1px; }
.mp-card-desc { font-size: var(--fs-xs); color: var(--text-muted); margin-top: 2px; }
.mp-card-actions { flex-shrink: 0; display: flex; align-items: center; gap: 6px; }
.mp-installed-badge { font-size: var(--fs-xs); color: var(--success); }
.mp-update-badge { font-size: var(--fs-xs); color: var(--accent); }
.sm { font-size: var(--fs-xs); height: 28px; padding: 0 10px; }
.icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; padding: 0; border: none; border-radius: var(--r-sm); background: transparent; color: var(--text-secondary); cursor: pointer; }
.icon-btn:hover { background: var(--surface-hover); }
</style>
