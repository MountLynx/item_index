<template>
  <div>
    <GroupTreeNode v-for="g in groupStore.tree" :key="g.id" :group="g" :depth="0" :selected-id="selectedId" @select="onSelect" />
    <div class="add" @click="startAdd">
      <span v-if="!adding" class="hint">+ 新建根分组</span>
      <input v-else ref="inp" v-model="name" placeholder="分组名" @keydown.enter="doAdd" @keydown.escape="cancel" @blur="cancel" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useGroupStore } from '@/stores/groups'
import GroupTreeNode from './GroupTreeNode.vue'

const props = defineProps<{ selectedId: number | null }>()
const emit = defineEmits<{ select: [id: number | null] }>()

const groupStore = useGroupStore()
const adding = ref(false)
const name = ref('')
const inp = ref<HTMLInputElement | null>(null)

function onSelect(id: number | null) { emit('select', id) }
async function startAdd() { adding.value = true; await nextTick(); inp.value?.focus() }
async function doAdd() { if (name.value.trim()) { await groupStore.create(name.value.trim()); name.value = ''; adding.value = false } }
function cancel() { adding.value = false; name.value = '' }
</script>

<style scoped>
.add { padding: 4px 8px 4px 28px; cursor: pointer; }
.hint { font-size: var(--fs-xs); color: var(--text-muted); }
.hint:hover { color: var(--accent); }
.add input { width: 100%; font-size: var(--fs-xs); height: 26px; }
</style>
