<template>
  <div class="cp">
    <!-- Tab bar -->
    <div class="cp-tabs" v-if="wsStore.active">
      <button v-for="tab in wsStore.active.centerTabs" :key="tab.plugin || 'list'"
        :class="{ active: activeTab === (tab.plugin || 'list') }"
        @click="activeTab = (tab.plugin || 'list')">
        <TablerIcon :name="tab.icon || 'file'" :size="14" />
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <!-- Tab content -->
    <div class="cp-content">
      <template v-if="!activeTab || activeTab === 'list'">
        <CenterList @new-item="$emit('newItem')" />
      </template>
      <template v-else>
        <div v-if="pluginLoading" class="cp-placeholder">
          <div class="cp-spinner"></div>
          <p>加载插件 "{{ activeTab }}"…</p>
        </div>
        <div v-else-if="pluginError" class="cp-placeholder error">
          <TablerIcon name="alert-triangle" :size="24" />
          <p>{{ pluginError }}</p>
          <button class="retry-btn" @click="loadActivePlugin()">重试</button>
        </div>
        <component v-else-if="pluginComponent" :is="pluginComponent" :context="pluginContext" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, shallowRef, markRaw } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { usePluginLoader } from '@/composables/usePluginLoader'
import { buildPluginContext } from '@/composables/usePluginContext'
import CenterList from './CenterList.vue'
import TablerIcon from './TablerIcon.vue'

const wsStore = useWorkspaceStore()
const { loadPlugin, clearError } = usePluginLoader()

const activeTab = ref<string>('list')
const pluginComponent = shallowRef<any>(null)
const pluginContext = ref<any>(null)
const pluginLoading = ref(false)
const pluginError = ref<string | null>(null)

defineEmits<{ newItem: [] }>()

// Sync active tab when workspace changes
watch(() => wsStore.active, async (cfg) => {
  if (!cfg) return
  activeTab.value = cfg.defaultTab || cfg.centerTabs[0]?.plugin || 'list'
  await loadActivePlugin()
}, { immediate: true })

async function loadActivePlugin() {
  const tabName = activeTab.value
  if (!tabName || tabName === 'list') {
    pluginComponent.value = null
    pluginLoading.value = false
    pluginError.value = null
    return
  }

  pluginLoading.value = true
  pluginError.value = null
  if (tabName) clearError(tabName)

  try {
    const result = await loadPlugin(tabName)
    pluginComponent.value = markRaw(result.component)
    const tab = wsStore.active?.centerTabs.find(t => t.plugin === tabName)
    pluginContext.value = buildPluginContext(result.manifest, tab?.config || {})
  } catch (e: any) {
    pluginComponent.value = null
    pluginError.value = e.message || '插件加载失败'
  } finally {
    pluginLoading.value = false
  }
}

watch(activeTab, () => { loadActivePlugin() })
</script>

<style scoped>
.cp { flex: 1; overflow: hidden; display: flex; flex-direction: column; background: var(--bg); }
.cp-tabs {
  display: flex; gap: 2px; padding: 6px 8px;
  background: var(--bg); border-bottom: 1px solid var(--border); flex-shrink: 0;
}
.cp-tabs button {
  display: flex; align-items: center; gap: 4px;
  padding: 4px 12px; height: 28px; font-size: var(--fs-xs); font-weight: var(--fw-medium);
  border: none; border-radius: var(--r-sm);
  background: transparent; color: var(--text-secondary);
  cursor: pointer; transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
}
.cp-tabs button:hover { background: var(--surface-hover); color: var(--text); }
.cp-tabs button.active { background: var(--surface-active); color: var(--text); font-weight: var(--fw-semibold); }
.cp-content { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
.cp-placeholder { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 8px; color: var(--text-muted); font-size: var(--fs-sm); }
.cp-placeholder.error { color: var(--danger); }
.cp-spinner { width: 24px; height: 24px; border: 3px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.retry-btn { font-size: var(--fs-sm); padding: 4px 16px; border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); color: var(--text); cursor: pointer; }
.retry-btn:hover { background: var(--surface-hover); }
</style>
