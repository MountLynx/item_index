<template>
  <div class="rp-container">
    <div class="rp-content">
      <!-- Tab: Detail -->
      <aside v-if="activeTab === 'detail'" class="rp">
        <div v-if="!detail" class="empty">
          <TablerIcon name="arrow-left" :size="28" :stroke="1" />
          <p>选择条目查看详情</p>
        </div>
        <template v-else>
          <div class="hd">
            <div class="title"><TablerIcon :name="detail.item_type.icon" :size="20" /> {{ detail.item.name }}</div>
            <div class="id font-mono">{{ detail.item.id }}</div>
          </div>

          <div class="sec"><div class="lbl">属性</div><PropertiesForm :detail="detail" /></div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">分组</div>
            <div class="chips">
              <span v-for="g in detail.groups" :key="g.id" class="chip" @click="removeGroup(g.id)">
                <TablerIcon name="folder" :size="13" />{{ g.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.groups.length === 0 && !addingGroup" class="text-muted text-xs">未分组</span>
            </div>
            <div v-if="!addingGroup" class="add-btn" @click="addingGroup = true">+ 添加分组</div>
            <div v-else class="add-row">
              <select v-model="newGroupId">
                <option :value="null" disabled>选择分组...</option>
                <option v-for="g in availableGroups" :key="g.id" :value="g.id">{{ g.name }}</option>
              </select>
              <button class="primary sm" @click="addGroup" :disabled="!newGroupId">确定</button>
              <button class="ghost sm" @click="addingGroup = false">取消</button>
            </div>
          </div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">标签</div>
            <div class="chips">
              <span v-for="t in detail.tags" :key="t.id" class="chip tag" @click="removeTag(t.id)">
                <TablerIcon name="hash" :size="13" />{{ t.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.tags.length === 0 && !addingTag" class="text-muted text-xs">无标签</span>
            </div>
            <div v-if="!addingTag" class="add-btn" @click="addingTag = true">+ 添加标签</div>
            <div v-else class="add-row">
              <select v-model="newTagId">
                <option :value="null" disabled>选择标签...</option>
                <option v-for="t in availableTags" :key="t.id" :value="t.id">{{ t.name }}</option>
              </select>
              <button class="primary sm" @click="addTag" :disabled="!newTagId">确定</button>
              <button class="ghost sm" @click="addingTag = false">取消</button>
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
    </div>

    <ActivityBar
      :tabs="tabs"
      :active="activeTab"
      @select="$emit('update:activeTab', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useItemStore } from '@/stores/items'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import PropertiesForm from './PropertiesForm.vue'
import FileTree from './FileTree.vue'
import TablerIcon from './TablerIcon.vue'
import ActivityBar from './ActivityBar.vue'
import TypeManager from './TypeManager.vue'

defineProps<{
  activeTab: 'detail' | 'types'
}>()

defineEmits<{
  'update:activeTab': [tab: string]
}>()

const tabs = [
  { id: 'detail', icon: 'file-description', title: '条目详情' },
  { id: 'types', icon: 'category', title: '类别管理' },
]

const itemStore = useItemStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()

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
