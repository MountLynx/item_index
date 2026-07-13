<template>
  <div>
    <div class="row" :class="{ sel: group.id === selectedId, over: dragOver }"
      :style="{ paddingLeft: depth * 18 + 8 + 'px' }"
      @click="selectGroup" @contextmenu.prevent="showCtx"
      @dragenter.prevent="onDragEnter" @dragover.prevent="onDragOver"
      @dragleave.prevent="onDragLeave" @drop.prevent="onDrop">
      <span class="arr" :class="{ open: expanded }" @click.stop="expanded = !expanded">
        <TablerIcon v-if="group.children.length" name="chevron-right" :size="12" />
      </span>
      <TablerIcon :name="expanded ? 'folder-open' : 'folder'" :size="15" />
      <span class="name">{{ group.name }}</span>
    </div>
    <div v-if="expanded">
      <GroupTreeNode v-for="child in group.children" :key="child.id" :group="child" :depth="depth + 1" :selected-id="selectedId" @select="onSelect" />
    </div>
    <div v-if="expanded && adding" class="row" :style="{ paddingLeft: (depth + 1) * 18 + 8 + 'px' }">
      <input ref="addInp" v-model="addName" :placeholder="$t('groupTree.groupName')" @keydown.enter="doAdd" @keydown.escape="cancel" @blur="cancel" />
    </div>

    <Teleport to="body">
      <div v-if="ctx.show" class="ctx-overlay" @click="ctx.show = false" @contextmenu.prevent="ctx.show = false">
        <div class="ctx-menu" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
          <button class="ctx-item" @click="startAdd">+ {{ $t('groupTree.newSubGroup') }}</button>
          <button class="ctx-item danger" @click="doDelete">{{ $t('common.delete') }}</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useGroupStore } from '@/stores/groups'
import { useToast } from '@/composables/toast'
import type { Group } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()
const props = defineProps<{ group: Group; depth: number; selectedId: number | null }>()
const emit = defineEmits<{ select: [id: number | null] }>()

const groupStore = useGroupStore()
const toast = useToast()
const expanded = ref(true)
const dragCounter = ref(0)
const dragOver = ref(false)
const adding = ref(false)
const addName = ref('')
const addInp = ref<HTMLInputElement | null>(null)
const ctx = reactive({ show: false, x: 0, y: 0 })

function selectGroup() { emit('select', props.selectedId === props.group.id ? null : props.group.id) }
function onSelect(id: number | null) { emit('select', id) }

// ── Drop target handlers — Vue template `.prevent` modifier handles preventDefault ──

function onDragEnter(_e: DragEvent) {
  dragCounter.value++
  dragOver.value = true
}

function onDragOver(e: DragEvent) {
  e.dataTransfer!.dropEffect = 'move'
}

function onDragLeave(_e: DragEvent) {
  dragCounter.value--
  if (dragCounter.value <= 0) { dragCounter.value = 0; dragOver.value = false }
}

async function onDrop(e: DragEvent) {
  dragCounter.value = 0
  dragOver.value = false
  const itemId = e.dataTransfer?.getData('text/plain')
  if (itemId) {
    await groupStore.addItemToGroup(itemId, props.group.id)
    toast.success(t('groupTree.addedToGroup', { name: props.group.name }))
  }
}

function showCtx(e: MouseEvent) { ctx.show = true; ctx.x = e.clientX; ctx.y = e.clientY }
async function startAdd() { ctx.show = false; adding.value = true; await nextTick(); addInp.value?.focus() }
async function doAdd() {
  if (addName.value.trim()) { await groupStore.create(addName.value.trim(), props.group.id); addName.value = ''; adding.value = false }
}
function cancel() { adding.value = false; addName.value = '' }
async function doDelete() {
  ctx.show = false
  if (confirm(t('groupTree.confirmDeleteGroup', { name: props.group.name }))) await groupStore.remove(props.group.id)
}
</script>

<style scoped>
.row {
  display: flex; align-items: center; gap: 4px;
  padding: 3px 8px; margin: 1px 4px; border-radius: var(--r-md);
  cursor: pointer; user-select: none; font-size: var(--fs-sm);
  transition: background var(--fast) var(--ease), border-color var(--fast) var(--ease);
  border: 1px solid transparent;
}
.row:hover { background: var(--surface-hover); }
.row.sel { background: var(--accent); color: var(--accent-fg); }
.row.over { border-color: var(--accent); background: var(--accent-subtle); }
.arr { width: 14px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; color: var(--text-muted); transition: transform var(--fast) var(--ease); }
.arr.open { transform: rotate(90deg); }
.row.sel .arr { color: var(--accent-fg); }
.name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.row input { width: 100%; font-size: var(--fs-xs); height: 26px; }

.ctx-overlay { position: fixed; inset: 0; z-index: 100; }
.ctx-menu {
  position: fixed; background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--r-lg); padding: 4px; min-width: 160px; z-index: 101;
  box-shadow: var(--shadow-md);
}
.ctx-item {
  display: flex; width: 100%; padding: 8px 12px; font-size: var(--fs-sm);
  border-radius: var(--r-sm); border: none; background: none; cursor: pointer;
  color: var(--text); text-align: left; height: auto;
}
.ctx-item:hover { background: var(--surface-hover); }
.ctx-item.danger { color: var(--danger); }
.ctx-item.danger:hover { background: var(--danger-subtle); }
</style>
