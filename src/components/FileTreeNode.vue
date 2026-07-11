<template>
  <div>
    <div class="row" :style="{ paddingLeft: depth * 18 + 8 + 'px' }" :class="{ hover: dragHover }"
      @click="onClick" @dblclick="onDblClick"
      @dragenter.prevent="onDragEnter" @dragover.prevent
      @dragleave.prevent="onDragLeave" @drop.prevent="onDrop"
      @contextmenu.prevent="ctx">
      <TablerIcon :name="node.is_dir ? (expanded ? 'folder-open' : 'folder') : iconFor(node.name)" :size="15" />
      <span v-if="editing" class="nm"><input ref="eRef" v-model="editName" @keydown.enter="doRename" @keydown.escape="editing = false" @blur="doRename" /></span>
      <span v-else class="nm">{{ node.name }}</span>
    </div>
    <div v-if="expanded && node.is_dir">
      <FileTreeNode v-for="c in node.children" :key="c.name" :node="c" :depth="depth + 1" :item-id="itemId" @refresh="$emit('refresh')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileNode } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const props = defineProps<{ node: FileNode; depth: number; itemId: string }>()
const emit = defineEmits<{ refresh: [] }>()

const expanded = ref(false)
const dragHover = ref(false)
const dragCounter = ref(0)
const editing = ref(false)
const editName = ref('')
const eRef = ref<HTMLInputElement | null>(null)

const extMap: Record<string, string> = { md: 'file-text', jpg: 'photo', jpeg: 'photo', png: 'photo', gif: 'photo', svg: 'photo', pdf: 'book', mp3: 'music', mp4: 'video', zip: 'file-zip', tar: 'file-zip', gz: 'file-zip' }
function iconFor(n: string) { return extMap[n.split('.').pop()?.toLowerCase() || ''] || 'file' }

function onClick() { if (props.node.is_dir) expanded.value = !expanded.value }
async function onDblClick() { if (!props.node.is_dir) await invoke('open_file', { itemId: props.itemId, relPath: props.node.name }) }

function onDragEnter() { dragCounter.value++; dragHover.value = true }
function onDragLeave() { dragCounter.value--; if (dragCounter.value <= 0) { dragCounter.value = 0; dragHover.value = false } }
async function onDrop(e: DragEvent) {
  e.stopPropagation() // prevent FileTree.onDrop from firing (double copy)
  dragCounter.value = 0; dragHover.value = false
  const f = e.dataTransfer?.files?.[0]; if (f) {
    // @ts-ignore
    const p = f.path; if (p) { await invoke('add_attachment', { itemId: props.itemId, sourcePath: p }); emit('refresh') }
  }
}
function ctx() {
  const c = prompt('操作: (R)重命名 (D)删除 (N)新建文件夹(仅目录)', '')
  if (!c) return
  const a = c.toUpperCase()
  if (a === 'D') { if (confirm(`删除"${props.node.name}"？`)) doDelete() }
  else if (a === 'R') startRename()
  else if (a === 'N' && props.node.is_dir) { const n = prompt('文件夹名:'); if (n) { invoke('create_folder', { itemId: props.itemId, relPath: n }).then(() => emit('refresh')) } }
}
async function doDelete() { await invoke('delete_file', { itemId: props.itemId, relPath: props.node.name }); emit('refresh') }
async function startRename() { editing.value = true; editName.value = props.node.name; await nextTick(); eRef.value?.focus(); eRef.value?.select() }
async function doRename() { if (editing.value && editName.value && editName.value !== props.node.name) { await invoke('rename_file', { itemId: props.itemId, oldName: props.node.name, newName: editName.value }); emit('refresh') } editing.value = false }
</script>

<style scoped>
.row {
  display: flex; align-items: center; gap: 4px;
  padding: 2px 8px; margin: 1px 4px; border-radius: var(--r-sm);
  cursor: pointer; user-select: none; font-size: var(--fs-sm);
  transition: background var(--fast) var(--ease);
}
.row:hover, .row.hover { background: var(--surface-hover); }
.nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.nm input { width: 100%; font-size: var(--fs-xs); height: 22px; }
</style>
