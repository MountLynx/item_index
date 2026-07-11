<template>
  <div>
    <div class="ft-row" :style="{ paddingLeft: depth * 18 + 6 + 'px' }" :class="{ hover: dragHover }"
      @click="onClick" @dblclick="onDblClick"
      @dragover.prevent @drop.prevent="onDrop"
      @dragenter.prevent="dragHover = true" @dragleave="dragHover = false"
      @contextmenu.prevent="showMenu">
      <span class="ft-icon">{{ node.is_dir ? (expanded ? '📂' : '📁') : iconFor(node.name) }}</span>
      <span v-if="editing" class="ft-name"><input ref="editRef" v-model="editName" @keydown.enter="doRename" @keydown.escape="editing = false" @blur="doRename" /></span>
      <span v-else class="ft-name">{{ node.name }}</span>
    </div>
    <div v-if="expanded && node.is_dir">
      <FileTreeNode v-for="child in node.children" :key="child.name" :node="child" :depth="depth + 1" :item-id="itemId" @refresh="$emit('refresh')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileNode } from '@/types/bindings'

const props = defineProps<{ node: FileNode; depth: number; itemId: string }>()
const emit = defineEmits<{ refresh: [] }>()

const expanded = ref(false)
const dragHover = ref(false)
const editing = ref(false)
const editName = ref('')
const editRef = ref<HTMLInputElement | null>(null)

function iconFor(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase()
  const map: Record<string, string> = { md: '📝', jpg: '🖼', jpeg: '🖼', png: '🖼', gif: '🖼', svg: '🖼', pdf: '📕', mp3: '🎵', mp4: '🎬', zip: '📦', tar: '📦', gz: '📦' }
  return map[ext || ''] || '📄'
}

function onClick() { if (props.node.is_dir) expanded.value = !expanded.value }
async function onDblClick() { if (!props.node.is_dir) await invoke('open_file', { itemId: props.itemId, relPath: props.node.name }) }

async function onDrop(event: DragEvent) {
  dragHover.value = false
  const fl = event.dataTransfer?.files
  if (fl?.[0]) {
    // @ts-ignore
    const p = fl[0].path
    if (p) { await invoke('add_attachment', { itemId: props.itemId, sourcePath: p }); emit('refresh') }
  }
}

function showMenu(event: MouseEvent) {
  event.preventDefault()
  const choice = prompt('操作: (R)重命名 (D)删除 (N)新建文件夹(仅目录)', '')
  if (!choice) return
  const c = choice.toUpperCase()
  if (c === 'D') { if (confirm(`删除"${props.node.name}"？`)) doDelete() }
  else if (c === 'R') startRename()
  else if (c === 'N' && props.node.is_dir) { const n = prompt('文件夹名:'); if (n) { invoke('create_folder', { itemId: props.itemId, relPath: n }).then(() => emit('refresh')) } }
}

async function doDelete() { await invoke('delete_file', { itemId: props.itemId, relPath: props.node.name }); emit('refresh') }
async function startRename() { editing.value = true; editName.value = props.node.name; await nextTick(); editRef.value?.focus(); editRef.value?.select() }
async function doRename() {
  if (editing.value && editName.value && editName.value !== props.node.name) {
    await invoke('rename_file', { itemId: props.itemId, oldName: props.node.name, newName: editName.value }); emit('refresh')
  }
  editing.value = false
}
</script>

<style scoped>
.ft-row {
  display: flex; align-items: center; gap: var(--space-1);
  padding: 2px var(--space-2); margin: 1px var(--space-1);
  border-radius: var(--radius-sm); cursor: pointer; user-select: none;
  font-size: var(--font-size-sm); transition: background var(--duration-fast) var(--ease-out);
}
.ft-row:hover, .ft-row.hover { background: var(--surface-hover); }
.ft-icon { flex-shrink: 0; font-size: 14px; }
.ft-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ft-name input { width: 100%; font-size: var(--font-size-xs); height: 24px; }
</style>
