<template>
  <div class="file-tree">
    <div class="ft-header">
      <span class="ft-title">附件</span>
      <span v-if="files && !loading" class="ft-count">{{ fileCount }}</span>
    </div>
    <div v-if="loading" class="ft-status text-muted">
      <span class="spinner" /> 加载中...
    </div>
    <div v-else-if="!itemId" class="ft-status text-muted">选择条目查看附件</div>
    <div v-else-if="files" class="ft-tree" :class="{ over: dragOver }"
      @dragover.prevent @drop.prevent="onDrop"
      @dragenter.prevent="dragOver = true" @dragleave.prevent="dragOver = false">
      <FileTreeNode v-for="child in files.children" :key="child.name" :node="child" :depth="0" :item-id="itemId!" @refresh="refresh" />
      <div v-if="files.children.length === 0" class="ft-drop-hint text-muted">拖入文件到此处</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import FileTreeNode from './FileTreeNode.vue'
import type { FileNode } from '@/types/bindings'

const props = defineProps<{ itemId: string | null }>()

const files = ref<FileNode | null>(null)
const loading = ref(false)
const dragOver = ref(false)
let lastItemId: string | null = null

function countFiles(node: FileNode | null): number {
  if (!node) return 0
  let n = node.is_dir ? 0 : 1
  for (const c of node.children) n += countFiles(c)
  return n
}

const fileCount = computed(() => countFiles(files.value))

async function refresh() {
  const id = props.itemId
  if (!id) { files.value = null; lastItemId = null; return }
  if (loading.value) return
  if (id === lastItemId && files.value) return
  loading.value = true
  try { files.value = await invoke<FileNode>('list_files', { itemId: id }); lastItemId = id }
  catch { files.value = null }
  finally { loading.value = false }
}

watch(() => props.itemId, refresh)

async function onDrop(event: DragEvent) {
  dragOver.value = false
  if (!props.itemId) return
  const fl = event.dataTransfer?.files
  if (fl && fl.length > 0) {
    // @ts-ignore: Tauri File has .path
    const p = fl[0].path
    if (p) { await invoke('add_attachment', { itemId: props.itemId, sourcePath: p }); await refresh() }
  }
}
</script>

<style scoped>
.file-tree { }
.ft-header {
  display: flex; align-items: baseline; gap: var(--space-1); margin-bottom: var(--space-1);
}
.ft-title { font-size: var(--font-size-xs); font-weight: var(--weight-semibold); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.ft-count { font-size: var(--font-size-xs); color: var(--text-muted); font-weight: var(--weight-normal); }
.ft-status { font-size: var(--font-size-xs); padding: var(--space-3); text-align: center; display: flex; align-items: center; justify-content: center; gap: var(--space-2); }
.ft-tree { min-height: 48px; border: 1px dashed transparent; border-radius: var(--radius-md); padding: 2px 0; transition: border-color var(--duration-fast) var(--ease-out); }
.ft-tree.over { border-color: var(--accent); background: var(--accent-subtle); }
.ft-drop-hint { font-size: var(--font-size-xs); padding: var(--space-4); text-align: center; }

.spinner {
  width: 14px; height: 14px; border: 2px solid var(--border); border-top-color: var(--accent);
  border-radius: var(--radius-full); animation: spin 0.6s linear infinite; display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
