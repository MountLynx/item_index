<template>
  <div class="rp-container">
    <div class="rp-content">
      <!-- Tab: Detail -->
      <aside v-if="activeTab === 'detail'" class="rp">
        <div v-if="!detail" class="empty">
          <TablerIcon name="arrow-left" :size="28" :stroke="1" />
          <p>{{ $t('rightPanel.selectItem') }}</p>
        </div>
        <template v-else>
          <div class="hd">
            <div class="title"><TablerIcon :name="detail.item_type.icon" :size="20" /> {{ detail.item.name }}</div>
            <div class="id font-mono">{{ detail.item.id }}</div>
          </div>

          <div class="sec"><div class="lbl">{{ $t('rightPanel.properties') }}</div><PropertiesForm :detail="detail" /></div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">{{ $t('rightPanel.groups') }}</div>
            <div class="chips">
              <span v-for="g in detail.groups" :key="g.id" class="chip" @click="removeGroup(g.id)">
                <TablerIcon name="folder" :size="13" />{{ g.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.groups.length === 0 && !addingGroup" class="text-muted text-xs">{{ $t('rightPanel.ungrouped') }}</span>
            </div>
            <div v-if="!addingGroup" class="add-btn" @click="addingGroup = true">{{ $t('rightPanel.addGroup') }}</div>
            <div v-else class="add-row">
              <select v-model="newGroupId">
                <option :value="null" disabled>{{ $t('rightPanel.selectGroup') }}</option>
                <option v-for="g in availableGroups" :key="g.id" :value="g.id">{{ g.name }}</option>
              </select>
              <button class="primary sm" @click="addGroup" :disabled="!newGroupId">{{ $t('rightPanel.confirm') }}</button>
              <button class="ghost sm" @click="addingGroup = false">{{ $t('rightPanel.cancel') }}</button>
            </div>
          </div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">{{ $t('rightPanel.tags') }}</div>
            <div class="chips">
              <span v-for="t in detail.tags" :key="t.id" class="chip tag" @click="removeTag(t.id)">
                <TablerIcon name="hash" :size="13" />{{ t.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.tags.length === 0 && !addingTag" class="text-muted text-xs">{{ $t('rightPanel.noTags') }}</span>
            </div>
            <div v-if="!addingTag" class="add-btn" @click="addingTag = true">{{ $t('rightPanel.addTag') }}</div>
            <div v-else class="add-row">
              <select v-model="newTagId">
                <option :value="null" disabled>{{ $t('rightPanel.selectTag') }}</option>
                <option v-for="t in availableTags" :key="t.id" :value="t.id">{{ t.name }}</option>
              </select>
              <button class="primary sm" @click="addTag" :disabled="!newTagId">{{ $t('rightPanel.confirm') }}</button>
              <button class="ghost sm" @click="addingTag = false">{{ $t('rightPanel.cancel') }}</button>
            </div>
          </div>
          <div class="sep" />

          <div class="sec file-sec">
            <FileTree :item-id="detail.item.id" />
          </div>
        </template>
      </aside>

      <!-- Tab: Type Manager -->
      <TypeManager v-else-if="activeTab === 'types'" />

      <!-- Tab: Plugins (rightPanelAddons) -->
      <aside v-else-if="activeTab === 'plugins'" class="rp">
        <div v-if="addonPlugins.length === 0" class="empty">
          <TablerIcon name="plug-connected" :size="28" :stroke="1" />
          <p>暂无插件，在 Workspace 设置中添加</p>
        </div>
        <div v-for="addon in addonPlugins" :key="addon.plugin" class="addon-panel">
          <div v-if="addon.loading" class="cp-placeholder">
            <div class="cp-spinner"></div>
            <p>加载 "{{ addon.plugin }}"…</p>
          </div>
          <div v-else-if="addon.error" class="cp-placeholder error">
            <TablerIcon name="alert-triangle" :size="18" />
            <p>{{ addon.error }}</p>
          </div>
          <component v-else-if="addon.component" :is="addon.component" :context="addon.context" />
        </div>
      </aside>
    </div>

    <ActivityBar
      :tabs="tabs"
      :active="activeTab"
      @select="$emit('update:activeTab', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, markRaw } from 'vue'
import { useI18n } from 'vue-i18n'
import { useItemStore } from '@/stores/items'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useWorkspaceStore } from '@/stores/workspace'
import { usePluginLoader } from '@/composables/usePluginLoader'
import { buildPluginContext } from '@/composables/usePluginContext'
import PropertiesForm from './PropertiesForm.vue'
import FileTree from './FileTree.vue'
import TablerIcon from './TablerIcon.vue'
import ActivityBar from './ActivityBar.vue'
import TypeManager from './TypeManager.vue'

const { t } = useI18n()

defineProps<{
  activeTab: 'detail' | 'types' | 'plugins'
}>()

defineEmits<{
  'update:activeTab': [tab: string]
}>()

const tabs = computed(() => {
  const list = [
    { id: 'detail', icon: 'file-description', title: t('rightPanel.detail') },
    { id: 'types', icon: 'category', title: t('common.category') },
  ]
  const addons = wsStore.active?.rightPanelAddons
  if (addons?.length) {
    // Use first addon's icon, fallback to plug-connected
    const icon = addonIcons.value.get(addons[0].plugin) || addons[0].plugin || 'plug-connected'
    const title = addons.length === 1
      ? (addonTitles.value.get(addons[0].plugin) || addons[0].plugin)
      : '插件'
    list.push({ id: 'plugins', icon, title })
  }
  return list
})

const itemStore = useItemStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const wsStore = useWorkspaceStore()
const { loadPlugin } = usePluginLoader()

// ── Right-panel addon plugins ──
interface AddonState {
  plugin: string
  component: any
  context: any
  loading: boolean
  error: string | null
}
const addonPlugins = ref<AddonState[]>([])
const addonIcons = ref<Map<string, string>>(new Map())
const addonTitles = ref<Map<string, string>>(new Map())

async function loadAddonPlugins() {
  const addons = wsStore.active?.rightPanelAddons || []
  const states: AddonState[] = addons.map(a => ({
    plugin: a.plugin,
    component: null,
    context: null,
    loading: true,
    error: null,
  }))
  addonPlugins.value = states

  for (const state of addonPlugins.value) {
    try {
      const result = await loadPlugin(state.plugin)
      state.component = markRaw(result.component)
      state.context = buildPluginContext(result.manifest, 
        addons.find(a => a.plugin === state.plugin)?.config || {})
      state.loading = false
      addonIcons.value.set(state.plugin, result.manifest.icon || 'plug-connected')
      addonTitles.value.set(state.plugin, result.manifest.title || state.plugin)
    } catch (e: any) {
      state.error = e.message || '加载失败'
      state.loading = false
    }
  }
}

watch(() => wsStore.active, () => { loadAddonPlugins() }, { immediate: true })

const detail = computed(() => itemStore.detail)
const addingGroup = ref(false)
const addingTag = ref(false)
const newGroupId = ref<number | null>(null)
const newTagId = ref<number | null>(null)

const availableGroups = computed(() => {
  if (!detail.value) return []
  const ids = new Set(detail.value.groups.map(g => g.id))
  return flattenGroups(groupStore.tree).filter(g => !ids.has(g.id))
})

const availableTags = computed(() => {
  if (!detail.value) return []
  const ids = new Set(detail.value.tags.map(t => t.id))
  return tagStore.tags.filter(t => !ids.has(t.id))
})

function flattenGroups(groups: any[]): any[] {
  return groups.flatMap(g => [g, ...flattenGroups(g.children || [])])
}

async function addGroup() {
  if (!newGroupId.value || !detail.value) return
  await groupStore.addItemToGroup(detail.value.item.id, newGroupId.value)
  await itemStore.select(detail.value.item.id)
  addingGroup.value = false; newGroupId.value = null
}

async function removeGroup(groupId: number) {
  if (!detail.value) return
  await groupStore.removeItemFromGroup(detail.value.item.id, groupId)
  await itemStore.select(detail.value.item.id)
}

async function addTag() {
  if (!newTagId.value || !detail.value) return
  await tagStore.addToItem(detail.value.item.id, newTagId.value)
  await itemStore.select(detail.value.item.id)
  addingTag.value = false; newTagId.value = null
}

async function removeTag(tagId: number) {
  if (!detail.value) return
  await tagStore.removeFromItem(detail.value.item.id, tagId)
  await itemStore.select(detail.value.item.id)
}
</script>

<style scoped>
.rp-container {
  display: flex;
  flex-shrink: 0;
  height: 100%;
}

.rp-content {
  width: var(--right-w);
  flex-shrink: 0;
  overflow: hidden;
}

.rp {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  border-left: 1px solid var(--border);
  background: var(--surface);
}

.empty {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; height: 100%; gap: 8px; color: var(--text-muted);
}

.hd { padding: 16px 16px 12px; }
.title {
  display: flex; align-items: center; gap: 8px;
  font-size: var(--fs-lg); font-weight: var(--fw-semibold); margin-bottom: 4px;
}
.id {
  font-size: var(--fs-xs); color: var(--text-secondary);
  background: var(--surface-hover); padding: 1px 6px;
  border-radius: var(--r-sm); display: inline-block;
}

.sec { padding: 8px 16px; }
.lbl {
  font-size: var(--fs-xs); font-weight: var(--fw-semibold);
  color: var(--text-muted); text-transform: uppercase;
  letter-spacing: 0.05em; margin-bottom: 8px;
}
.sep { height: 1px; background: var(--border); margin: 0 16px; }
.file-sec { flex: 1; min-height: 100px; }

.chips { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 4px; }
.chip {
  display: inline-flex; align-items: center; gap: 4px; font-size: var(--fs-xs);
  padding: 2px 8px 2px 10px; border-radius: var(--r-full); cursor: pointer;
  background: var(--bg); color: var(--text-secondary);
  transition: all var(--fast) var(--ease);
}
.chip:hover { background: var(--danger-subtle); color: var(--danger); }
.chip.tag { color: var(--accent); background: var(--accent-subtle); }
.chip.tag:hover { background: var(--danger-subtle); color: var(--danger); }

.add-btn {
  font-size: var(--fs-xs); color: var(--text-muted);
  cursor: pointer; padding: 2px 0;
}
.add-btn:hover { color: var(--accent); }

.add-row { display: flex; gap: 4px; margin-top: 4px; align-items: center; }
.add-row select { font-size: var(--fs-xs); height: 28px; flex: 1; }
.sm { font-size: var(--fs-xs); height: 26px; }
</style>
