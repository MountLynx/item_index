<template>
  <div>
    <div class="row" :class="{ sel: group.id === selectedId, over: dragOver }"
      :style="{ paddingLeft: depth * 18 + 8 + 'px' }"
      @dragover.prevent="onDragOver" @dragleave="dragOver = false"
      @drop.prevent="onDrop">
      <span class="arr" :class="{ open: expanded }" @click="expanded = !expanded">
        <TablerIcon v-if="group.children.length" name="chevron-right" :size="12" />
      </span>
      <TablerIcon :name="expanded ? 'folder-open' : 'folder'" :size="15" />
      <span class="name" @click="selectGroup">{{ group.name }}</span>
    </div>
    <div v-if="expanded">
      <GroupTreeNode v-for="child in group.children" :key="child.id" :group="child" :depth="depth + 1" :selected-id="selectedId" @select="onSelect" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useGroupStore } from '@/stores/groups'
import type { Group } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const props = defineProps<{ group: Group; depth: number; selectedId: number | null }>()
const emit = defineEmits<{ select: [id: number | null] }>()

const groupStore = useGroupStore()
const expanded = ref(true)
const dragOver = ref(false)

function selectGroup() { emit('select', props.selectedId === props.group.id ? null : props.group.id) }
function onSelect(id: number | null) { emit('select', id) }

function onDragOver(e: DragEvent) {
  if (e.dataTransfer?.types.includes('text/plain')) {
    dragOver.value = true
    e.dataTransfer!.dropEffect = 'move'
  }
}

async function onDrop(e: DragEvent) {
  dragOver.value = false
  const itemId = e.dataTransfer?.getData('text/plain')
  if (itemId) {
    await groupStore.addItemToGroup(itemId, props.group.id)
  }
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
</style>
