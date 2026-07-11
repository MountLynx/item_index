<template>
  <div class="file-tree">
    <div class="header">
      📎 附件 <span class="count" v-if="fileCount">({{ fileCount }})</span>
    </div>
    <div v-if="files" class="tree" @dragover.prevent @drop.prevent="onRootDrop">
      <FileTreeNode v-for="child in files.children" :key="child.name" :node="child" :depth="0" :item-id="itemId" @refresh="refresh" />
    </div>
    <div v-else class="empty">选择条目查看附件</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import FileTreeNode from './FileTreeNode.vue'
import type { FileNode } from '@/types/bindings'

const props = defineProps<{ itemId: string | null }>()

const files = ref<FileNode | null>(null)

const fileCount = computed(() => countFiles(files.value))

function countFiles(node: FileNode | null): number {
  if (!node) return 0
  let n = node.is_dir ? 0 : 1
  for (const c of node.children) n += countFiles(c)
  return n
}

async function refresh() {
  if (props.itemId) {
    files.value = await invoke<FileNode>('list_files', { itemId: props.itemId })
  }
}

watch(() => props.itemId, refresh)

async function onRootDrop(event: DragEvent) {
  if (!props.itemId) return
  const fileList = event.dataTransfer?.files
  if (fileList && fileList.length > 0) {
    // @ts-ignore
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
.empty { font-size: 12px; color: var(--text-secondary); padding: 12px; text-align: center; }
</style>
