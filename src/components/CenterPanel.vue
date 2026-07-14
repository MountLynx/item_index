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
        <div v-if="!pluginComponent" class="cp-placeholder">
          <TablerIcon name="alert-triangle" :size="24" />
          <p>插件 "{{ activeTab }}" 加载失败或未安装</p>
        </div>
        <component v-else :is="pluginComponent" :context="pluginContext" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, shallowRef } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { usePluginLoader } from '@/composables/usePluginLoader'
import { buildPluginContext } from '@/composables/usePluginContext'
import CenterList from './CenterList.vue'
import TablerIcon from './TablerIcon.vue'

const wsStore = useWorkspaceStore()
const { loadPlugin } = usePluginLoader()

const activeTab = ref<string>('list')
const pluginComponent = shallowRef<any>(null)
const pluginContext = ref<any>(null)

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
    return
  }

  const result = await loadPlugin(tabName)
  if (result) {
    pluginComponent.value = result.component
    // Find tab config
    const tab = wsStore.active?.centerTabs.find(t => t.plugin === tabName)
    pluginContext.value = buildPluginContext(result.manifest, tab?.config || {})
  } else {
    pluginComponent.value = null
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
</style>
