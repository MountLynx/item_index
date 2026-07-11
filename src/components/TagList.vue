<template>
  <div>
    <div class="tags">
      <span v-for="tag in tagStore.tags" :key="tag.id" class="tag" :class="{ active: tag.id === sel }" @click="toggle(tag.id)">
        <TablerIcon name="hash" :size="12" />{{ tag.name }}
      </span>
    </div>
    <input class="inp" v-model="name" placeholder="新建标签" @keydown.enter="add" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTagStore } from '@/stores/tags'
import TablerIcon from './TablerIcon.vue'

const tagStore = useTagStore()
const sel = ref<number | null>(null)
const name = ref('')
const emit = defineEmits<{ select: [id: number | null] }>()

function toggle(id: number) { sel.value = sel.value === id ? null : id; emit('select', sel.value) }
async function add() { const n = name.value.trim(); if (n) { await tagStore.create(n); name.value = '' } }
</script>

<style scoped>
.tags { display: flex; flex-wrap: wrap; gap: 4px; padding: 0 4px; margin-bottom: 6px; }
.tag {
  display: inline-flex; align-items: center; gap: 2px;
  font-size: var(--fs-xs); padding: 2px 10px; border-radius: var(--r-full);
  cursor: pointer; user-select: none; transition: all var(--fast) var(--ease);
  background: var(--bg); color: var(--text-secondary);
}
.tag:hover { color: var(--accent); }
.tag.active { background: var(--accent); color: var(--accent-fg); font-weight: var(--fw-medium); }
.inp { width: 100%; font-size: var(--fs-xs); height: 26px; border-color: transparent; }
.inp:focus { border-color: var(--accent); }
.inp::placeholder { color: var(--text-muted); }
</style>
