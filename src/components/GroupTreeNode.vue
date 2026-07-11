<template>
  <div>
    <div class="tree-row" :class="{ selected: group.id === selectedId }" :style="{ paddingLeft: depth * 16 + 8 + 'px' }">
      <span class="toggle" @click="toggleExpand">
        {{ group.children.length > 0 ? (expanded ? '▾' : '▸') : ' ' }}
      </span>
      <span class="name" @click="selectGroup">{{ group.name }}</span>
    </div>
    <div v-if="expanded">
      <GroupTreeNode v-for="child in group.children" :key="child.id" :group="child" :depth="depth + 1" :selected-id="selectedId" @select="onSelect" />
    </div>
    <div v-if="expanded && adding" class="tree-row" :style="{ paddingLeft: (depth + 1) * 16 + 8 + 'px' }">
      <input ref="newInput" v-model="newName" placeholder="子分组名" @keydown.enter="addChild" @keydown.escape="cancelAdd" @blur="cancelAdd" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useGroupStore } from '@/stores/groups'
import type { Group } from '@/types/bindings'

const props = defineProps<{ group: Group; depth: number; selectedId: number | null }>()
const emit = defineEmits<{ select: [id: number | null] }>()

const groupStore = useGroupStore()
const expanded = ref(true)
const adding = ref(false)
const newName = ref('')
const newInput = ref<HTMLInputElement | null>(null)

function toggleExpand() { expanded.value = !expanded.value }

function selectGroup() { emit('select', props.selectedId === props.group.id ? null : props.group.id) }

function onSelect(id: number | null) { emit('select', id) }

async function startAdd() {
  adding.value = true
  await nextTick()
  newInput.value?.focus()
}

async function addChild() {
  if (newName.value.trim()) {
    await groupStore.create(newName.value.trim(), props.group.id)
    newName.value = ''
    adding.value = false
  }
}

function cancelAdd() { adding.value = false; newName.value = '' }
</script>

<style scoped>
.tree-row {
  display: flex; align-items: center; gap: 4px; padding: 3px 4px; cursor: pointer; border-radius: 3px;
  font-size: 13px; user-select: none;
}
.tree-row:hover { background: var(--border); }
.tree-row.selected { background: var(--accent); color: #fff; }
.toggle { width: 14px; text-align: center; font-size: 10px; flex-shrink: 0; }
.name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
input { width: calc(100% - 20px); font-size: 12px; padding: 1px 4px; }
</style>
