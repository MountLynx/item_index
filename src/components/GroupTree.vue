<template>
  <div class="group-tree">
    <div v-for="group in groupStore.tree" :key="group.id">
      <GroupTreeNode :group="group" :depth="0" :selected-id="selectedId" @select="onSelect" />
    </div>
    <div class="add-row" @click="startAdd">
      <span v-if="!adding" class="add-hint">+ 新建分组</span>
      <input v-else ref="addInput" v-model="name" placeholder="分组名" @keydown.enter="doAdd" @keydown.escape="cancel" @blur="cancel" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useGroupStore } from '@/stores/groups'
import GroupTreeNode from './GroupTreeNode.vue'

const groupStore = useGroupStore()
const selectedId = ref<number | null>(null)
const adding = ref(false)
const name = ref('')
const addInput = ref<HTMLInputElement | null>(null)

const emit = defineEmits<{ select: [id: number | null] }>()

function onSelect(id: number | null) { selectedId.value = id; emit('select', id) }
async function startAdd() { adding.value = true; await nextTick(); addInput.value?.focus() }
async function doAdd() {
  if (name.value.trim()) { await groupStore.create(name.value.trim()); name.value = ''; adding.value = false }
}
function cancel() { adding.value = false; name.value = '' }
</script>

<style scoped>
.group-tree { padding: 0; }
.add-row { padding: var(--space-1) var(--space-2) var(--space-1) 28px; cursor: pointer; }
.add-hint { font-size: var(--font-size-xs); color: var(--text-muted); transition: color var(--duration-fast) var(--ease-out); }
.add-hint:hover { color: var(--accent); }
.add-row input { width: 100%; font-size: var(--font-size-xs); height: 26px; }
</style>
