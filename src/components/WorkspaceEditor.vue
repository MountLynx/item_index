<template>
  <div class="ws-edit">
    <div class="ws-edit-header">
      <button class="icon-btn" @click="emit('back')">
        <TablerIcon name="arrow-left" :size="16" />
      </button>
      <h3>编辑工作区：{{ local.name }}</h3>
    </div>

    <div class="ws-edit-body">
      <!-- Name + Icon -->
      <div class="edit-row">
        <label>名称</label>
        <input v-model="local.name" class="name-input" />
      </div>
      <div class="edit-row">
        <label>图标</label>
        <IconPicker v-model="local.icon" />
      </div>

      <!-- Item type checkboxes -->
      <div class="edit-section">
        <label class="section-label">条目类型</label>
        <label v-for="t in typeStore.types" :key="t.id" class="check-row">
          <input type="checkbox" :value="t.name" v-model="local.itemTypes" />
          <TablerIcon :name="t.icon" :size="14" />
          <span>{{ t.name }}</span>
        </label>
        <p class="hint">仅显示已勾选的条目类型</p>
      </div>

      <!-- Center tabs -->
      <div class="edit-section">
        <label class="section-label">中间面板页签</label>
        <div v-for="(tab, i) in local.centerTabs" :key="i" class="tab-row">
          <span class="drag-handle">&#8801;</span>
          <span v-if="tab.type === 'list'" class="tab-label">列表</span>
          <span v-else class="tab-label">{{ tab.plugin }}</span>
          <span class="tab-badge">{{ tab.type === 'list' ? '内置' : '插件' }}</span>
          <button class="icon-btn sm" @click="removeTab(i)" :disabled="local.centerTabs.length <= 1">
            <TablerIcon name="trash" :size="14" />
          </button>
        </div>
        <div class="add-tab-row">
          <select v-model="newTabType">
            <option value="list">内置列表</option>
            <option v-for="p in centerPanelPlugins" :key="p.name" :value="p.name">{{ p.title }}</option>
          </select>
          <input v-model="newTabLabel" placeholder="页签标题" class="label-input" />
          <button @click="addTab" :disabled="!newTabLabel.trim()">+ 添加页签</button>
        </div>
        <div class="edit-row">
          <label>默认页签</label>
          <select v-model="local.defaultTab">
            <option v-for="tab in local.centerTabs" :key="tab.label" :value="tab.plugin || 'list'">
              {{ tab.label }}
            </option>
          </select>
        </div>
      </div>

      <!-- Sidebar / RightPanel (placeholder) -->
      <div class="edit-section muted">
        <label class="section-label">侧边栏附加 (即将推出)</label>
        <p class="hint">暂无插件</p>
      </div>
      <div class="edit-section muted">
        <label class="section-label">右侧栏附加 (即将推出)</label>
        <p class="hint">暂无插件</p>
      </div>
    </div>

    <div class="ws-edit-footer">
      <button @click="emit('back')">取消</button>
      <button class="primary" @click="save">保存</button>
      <button @click="setAsDefault">设为默认</button>
      <button @click="exportPreset">导出预设</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useWorkspaceStore } from '@/stores/workspace'
import { useTypeStore } from '@/stores/types'
import type { WorkspaceConfig, PluginManifest, CenterTab } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'
import IconPicker from './IconPicker.vue'

const props = defineProps<{ workspaceName: string }>()
const emit = defineEmits<{ back: [] }>()

const wsStore = useWorkspaceStore()
const typeStore = useTypeStore()
const centerPanelPlugins = ref<PluginManifest[]>([])

const local = reactive<WorkspaceConfig>({
  name: '',
  icon: 'layout',
  itemTypes: [],
  centerTabs: [{ type: 'list', label: '列表', icon: 'list' }],
  defaultTab: 'list',
  rightPanelAddons: [],
  sidebarAddons: [],
})

const newTabType = ref('list')
const newTabLabel = ref('')

async function load() {
  const cfg = await invoke<WorkspaceConfig>('read_workspace', { name: props.workspaceName })
  Object.assign(local, cfg)
  const plugins = await invoke<PluginManifest[]>('list_installed_plugins')
  centerPanelPlugins.value = plugins.filter(p => p.extends === 'center-panel')
}

function addTab() {
  const label = newTabLabel.value.trim()
  if (!label) return
  const tab: CenterTab = newTabType.value === 'list'
    ? { type: 'list', label, icon: 'list' }
    : { type: 'plugin', plugin: newTabType.value, label, icon: undefined }
  local.centerTabs.push(tab)
  newTabLabel.value = ''
}

function removeTab(i: number) {
  local.centerTabs.splice(i, 1)
}

async function save() {
  await wsStore.save({ ...local })
  emit('back')
}

async function setAsDefault() {
  const name = local.name
  await save()
  await wsStore.activate(name)
  emit('back')
}

async function exportPreset() {
  await save()
  await wsStore.exportAsPreset(local.name)
  alert(`工作区 "${local.name}" 已导出为预设`)
}

watch(() => props.workspaceName, load, { immediate: true })
</script>

<style scoped>
.ws-edit {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 0;
}

.ws-edit-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}
.ws-edit-header h3 {
  margin: 0;
  font-size: var(--fs-base);
  font-weight: var(--fw-semibold);
}

.ws-edit-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.ws-edit-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
  flex-shrink: 0;
}

.edit-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
}
.edit-row > label {
  font-size: var(--fs-sm);
  font-weight: var(--fw-medium);
  color: var(--text);
  flex-shrink: 0;
  margin-right: 12px;
}

.name-input {
  flex: 1;
  height: 32px;
  padding: 0 10px;
  font-size: var(--fs-sm);
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
  color: var(--text);
}
.name-input:focus {
  outline: none;
  border-color: var(--accent);
}

.edit-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.section-label {
  font-size: var(--fs-sm);
  font-weight: var(--fw-semibold);
  color: var(--text-heading);
  margin-bottom: 4px;
}

.check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  font-size: var(--fs-sm);
  cursor: pointer;
}
.check-row input[type="checkbox"] {
  margin: 0;
}

.hint {
  font-size: var(--fs-xs);
  color: var(--text-muted);
  margin: 2px 0 0;
}

/* Tab rows */
.tab-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
}
.drag-handle {
  color: var(--text-muted);
  cursor: grab;
  user-select: none;
  font-size: 16px;
  line-height: 1;
}
.tab-label {
  flex: 1;
  font-size: var(--fs-sm);
  font-weight: var(--fw-medium);
}
.tab-badge {
  font-size: var(--fs-xs);
  color: var(--text-muted);
  background: var(--surface-hover);
  padding: 2px 6px;
  border-radius: var(--r-sm);
}

.add-tab-row {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-top: 4px;
}
.add-tab-row select,
.add-tab-row .label-input {
  height: 30px;
  font-size: var(--fs-sm);
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
  color: var(--text);
  padding: 0 8px;
}
.add-tab-row select { width: 130px; }
.add-tab-row .label-input { flex: 1; min-width: 0; }
.add-tab-row .label-input:focus { outline: none; border-color: var(--accent); }
.add-tab-row button {
  height: 30px;
  font-size: var(--fs-sm);
  padding: 0 10px;
  white-space: nowrap;
}

.edit-section.muted {
  opacity: 0.6;
  pointer-events: none;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: var(--r-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
}
.icon-btn:hover { background: var(--surface-hover); color: var(--text); }
.icon-btn:disabled { opacity: 0.4; cursor: default; }
.icon-btn.sm { width: 24px; height: 24px; }
</style>
