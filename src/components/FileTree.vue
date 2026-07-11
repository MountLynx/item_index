<template>
  <div>
    <div class="hd">
      <TablerIcon name="paperclip" :size="13" /> 附件
      <span v-if="files && !loading" class="count">{{ fileCount }}</span>
    </div>
    <div v-if="loading" class="status text-muted"><span class="spin" /> 加载中...</div>
    <div v-else-if="!itemId" class="status text-muted">选择条目查看附件</div>
    <div v-else-if="files" class="tree" :class="{ over: dragOver }"
      @dragover.prevent @drop.prevent="onDrop"
      @dragenter.prevent="dragOver = true" @dragleave.prevent="dragOver = false">
      <FileTreeNode v-for="child in files.children" :key="child.name" :node="child" :depth="0" :item-id="itemId!" @refresh="refresh" />
      <div v-if="files.children.length === 0" class="status text-muted">拖入文件</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import FileTreeNode from './FileTreeNode.vue'
import TablerIcon from './TablerIcon.vue'
import type { FileNode } from '@/types/bindings'

const props = defineProps<{ itemId: string | null }>()
const files = ref<FileNode | null>(null)
const loading = ref(false)
const dragOver = ref(false)
let last: string | null = null

function count(n: FileNode | null): number { if (!n) return 0; let x = n.is_dir ? 0 : 1; for (const c of n.children) x += count(c); return x }
const fileCount = computed(() => count(files.value))

async function refresh() {
  const id = props.itemId; if (!id) { files.value = null; last = null; return }
  if (loading.value) return; if (id === last && files.value) return
  loading.value = true
  try { files.value = await invoke<FileNode>('list_files', { itemId: id }); last = id } catch { files.value = null } finally { loading.value = false }
}
watch(() => props.itemId, refresh)

async function onDrop(e: DragEvent) {
  dragOver.value = false; if (!props.itemId) return
  const f = e.dataTransfer?.files?.[0]; if (f) {
    // @ts-ignore
    const p = f.path; if (p) { await invoke('add_attachment', { itemId: props.itemId, sourcePath: p }); await refresh() }
  }
}
</script>

<style scoped>
.hd { display: flex; align-items: baseline; gap: 4px; font-size: var(--fs-xs); font-weight: var(--fw-semibold); color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }
.count { font-weight: var(--fw-normal); }
.status { font-size: var(--fs-xs); padding: 16px; text-align: center; }
.tree { min-height: 48px; border: 1px dashed transparent; border-radius: var(--r-md); padding: 2px 0; transition: border-color var(--fast) var(--ease), background var(--fast) var(--ease); }
.tree.over { border-color: var(--accent); background: var(--accent-subtle); }
.spin { width: 14px; height: 14px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: var(--r-full); animation: spin 0.6s linear infinite; display: inline-block; vertical-align: middle; margin-right: 4px; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
