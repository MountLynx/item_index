<template>
  <div>
    <div class="file-row" :style="{ paddingLeft: depth * 16 + 8 + 'px' }" :class="{ hover: dragging }"
      @click="onClick" @dblclick="onDblClick" @dragover.prevent="dragging = true" @dragleave="dragging = false"
      @drop.prevent="onDrop" @contextmenu.prevent="showMenu">
      <span class="icon">{{ node.is_dir ? (expanded ? '📂' : '📁') : '📄' }}</span>
      <span v-if="editing" class="name"><input ref="editInput" v-model="editName" @keydown.enter="doRename" @keydown.escape="editing = false" @blur="doRename" /></span>
      <span v-else class="name">{{ node.name }}</span>
      <span v-if="dragging" class="drop-indicator">释放以添加</span>
    </div>
    <div v-if="expanded && node.is_dir">
      <FileTreeNode v-for="child in node.children" :key="child.name" :node="child" :depth="depth + 1" :item-id="itemId" @refresh="$emit('refresh')" />
    </div>

    <div v-if="contextMenu.show" class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
      <div v-if="node.is_dir" class="menu-item" @click="newFolder">📁 新建文件夹</div>
      <div class="menu-item" @click="startRename">✏ 重命名</div>
      <div class="menu-item danger" @click="deleteFile">🗑 删除</div>
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
const dragging = ref(false)
const editing = ref(false)
const editName = ref('')
const editInput = ref<HTMLInputElement | null>(null)
const contextMenu = ref({ show: false, x: 0, y: 0 })

function onClick() {
  if (props.node.is_dir) expanded.value = !expanded.value
}

async function onDblClick() {
  if (!props.node.is_dir) {
    await invoke('open_file', { itemId: props.itemId, relPath: props.node.name })
  }
}

async function onDrop(event: DragEvent) {
  dragging.value = false
  const files = event.dataTransfer?.files
  if (files && files.length > 0) {
    // @ts-ignore: Tauri exposes path on File objects
    const path = files[0].path
    if (path) {
      await invoke('add_attachment', { itemId: props.itemId, sourcePath: path })
      emit('refresh')
    }
  }
}

function showMenu(event: MouseEvent) {
  contextMenu.value = { show: true, x: event.clientX, y: event.clientY }
  setTimeout(() => { contextMenu.value.show = false }, 3000)
}

async function newFolder() {
  const name = prompt('文件夹名:')
  if (name) {
    await invoke('create_folder', { itemId: props.itemId, relPath: name })
    emit('refresh')
  }
  contextMenu.value.show = false
}

async function startRename() {
  contextMenu.value.show = false
  editing.value = true
  editName.value = props.node.name
  await nextTick()
  editInput.value?.focus()
  editInput.value?.select()
}

async function doRename() {
  if (editing.value && editName.value !== props.node.name) {
    await invoke('rename_file', { itemId: props.itemId, oldName: props.node.name, newName: editName.value })
    emit('refresh')
  }
  editing.value = false
}

async function deleteFile() {
  contextMenu.value.show = false
  if (confirm(`确定删除"${props.node.name}"？`)) {
    await invoke('delete_file', { itemId: props.itemId, relPath: props.node.name })
    emit('refresh')
  }
}
</script>

<style scoped>
.file-row {
  display: flex; align-items: center; gap: 4px; padding: 2px 4px; cursor: pointer; border-radius: 3px;
  font-size: 13px; user-select: none;
}
.file-row:hover, .file-row.hover { background: var(--border); }
.icon { flex-shrink: 0; font-size: 14px; }
.name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.name input { font-size: 12px; width: 100%; }
.drop-indicator { font-size: 11px; color: var(--accent); margin-left: auto; }
.context-menu {
  position: fixed; background: var(--surface); border: 1px solid var(--border);
  border-radius: 4px; padding: 4px 0; z-index: 100; min-width: 140px; box-shadow: 0 2px 8px rgba(0,0,0,0.15);
}
.menu-item { padding: 6px 16px; cursor: pointer; font-size: 13px; }
.menu-item:hover { background: var(--border); }
.menu-item.danger { color: var(--danger); }
</style>
