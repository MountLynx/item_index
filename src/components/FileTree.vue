<template>
  <div class="file-tree">
    <div class="header">
      📎 附件 <span class="count" v-if="files">({{ fileCount }})</span>
    </div>
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="!itemId" class="empty">选择条目查看附件</div>
    <div v-else-if="files" class="tree" @dragover.prevent @drop.prevent="onRootDrop">
      <FileTreeNode v-for="child in files.children" :key="child.name" :node="child" :depth="0" :item-id="itemId" @refresh="refresh" />
      <div v-if="files.children.length === 0" class="empty">空空如也，拖入文件到此处</div>
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
let lastItemId: string | null = null

const fileCount = computed(() => countFiles(files.value))

function countFiles(node: FileNode | null): number {
  if (!node) return 0
  let n = node.is_dir ? 0 : 1
  for (const c of node.children) n += countFiles(c)
  return n
}

async function refresh() {
  const id = props.itemId
  if (!id) {
    files.value = null
    lastItemId = null
    return
  }
  if (loading.value) return // prevent duplicate
  if (id === lastItemId && files.value) return // already loaded
  loading.value = true
  try {
    files.value = await invoke<FileNode>('list_files', { itemId: id })
    lastItemId = id
  } catch {
    files.value = null
  } finally {
    loading.value = false
  }
}

watch(() => props.itemId, refresh, { immediate: false })

async function onRootDrop(event: DragEvent) {
  if (!props.itemId) return
  const fileList = event.dataTransfer?.files
  if (fileList && fileList.length > 0) {
    // @ts-ignore: Tauri exposes path on File objects
    const path = fileList[0].path
    if (path) {
      await invoke('add_attachment', { itemId: props.itemId, sourcePath: path })
      await refresh()
    }
  }
}
</script>

<style scoped>
.file-tree { padding: 8px 0; }
.header { font-size: 13px; font-weight: 600; margin-bottom: 4px; color: var(--text); }
.count { font-weight: 400; font-size: 11px; color: var(--text-secondary); }
.tree { min-height: 40px; border: 1px dashed transparent; border-radius: 4px; }
.tree:hover { border-color: var(--border); }
.loading { font-size: 12px; color: var(--text-secondary); padding: 8px; text-align: center; }
.empty { font-size: 12px; color: var(--text-secondary); padding: 12px; text-align: center; }
</style>
