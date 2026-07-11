<template>
  <div>
    <div v-for="group in groupStore.tree" :key="group.id">
      <GroupTreeNode :group="group" :depth="0" :selected-id="selectedId" @select="onSelect" />
    </div>
    <div class="new-group">
      <input v-if="adding" ref="newInput" v-model="newName" placeholder="分组名" @keydown.enter="addRoot" @keydown.escape="cancelAdd" @blur="cancelAdd" />
      <span v-else class="add-btn" @click="startAdd">+ 新建分组</span>
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
const newName = ref('')
const newInput = ref<HTMLInputElement | null>(null)

const emit = defineEmits<{ select: [id: number | null] }>()

function onSelect(id: number | null) {
  selectedId.value = id
  emit('select', id)
}

async function startAdd() {
  adding.value = true
  await nextTick()
  newInput.value?.focus()
}

async function addRoot() {
  if (newName.value.trim()) {
    await groupStore.create(newName.value.trim())
    newName.value = ''
    adding.value = false
  }
}

function cancelAdd() { adding.value = false; newName.value = '' }
</script>

<style scoped>
.new-group { padding: 2px 0 2px 16px; }
.add-btn { font-size: 12px; color: var(--text-secondary); cursor: pointer; }
.add-btn:hover { color: var(--accent); }
.new-group input { width: 100%; font-size: 12px; padding: 2px 6px; }
</style>
