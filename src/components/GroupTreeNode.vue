<template>
  <div>
    <div class="tree-node" :class="{ active: group.id === selectedId }" :style="{ paddingLeft: depth * 18 + 6 + 'px' }">
      <span class="arrow" @click="toggleExpand" :class="{ open: expanded }">
        {{ group.children.length > 0 ? '▸' : '' }}
      </span>
      <span class="icon">📁</span>
      <span class="name" @click="selectGroup">{{ group.name }}</span>
    </div>
    <div v-if="expanded">
      <GroupTreeNode v-for="child in group.children" :key="child.id" :group="child" :depth="depth + 1" :selected-id="selectedId" @select="onSelect" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Group } from '@/types/bindings'

const props = defineProps<{ group: Group; depth: number; selectedId: number | null }>()
const emit = defineEmits<{ select: [id: number | null] }>()

const expanded = ref(true)

function toggleExpand() { expanded.value = !expanded.value }
function selectGroup() { emit('select', props.selectedId === props.group.id ? null : props.group.id) }
function onSelect(id: number | null) { emit('select', id) }
</script>

<style scoped>
.tree-node {
  display: flex; align-items: center; gap: var(--space-1);
  padding: var(--space-1) var(--space-2); margin: 1px var(--space-1);
  border-radius: var(--radius-md); cursor: pointer; user-select: none;
  font-size: var(--font-size-sm); transition: background var(--duration-fast) var(--ease-out);
}
.tree-node:hover { background: var(--surface-hover); }
.tree-node.active { background: var(--accent); color: var(--accent-foreground); }

.arrow { width: 12px; font-size: 8px; text-align: center; flex-shrink: 0; transition: transform var(--duration-fast) var(--ease-out); color: var(--text-muted); }
.arrow.open { transform: rotate(90deg); }
.tree-node.active .arrow { color: var(--accent-foreground); }
.icon { font-size: 12px; flex-shrink: 0; }
.name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: var(--weight-normal); }
</style>
