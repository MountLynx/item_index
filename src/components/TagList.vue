<template>
  <div class="tag-list-wrap">
    <div class="tags">
      <span v-for="tag in tagStore.tags" :key="tag.id" class="tag" :class="{ active: tag.id === selectedId }" @click="selectTag(tag.id)">
        #&nbsp;{{ tag.name }}
      </span>
    </div>
    <input class="tag-input" v-model="name" placeholder="+ 新建标签" @keydown.enter="addTag" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTagStore } from '@/stores/tags'

const tagStore = useTagStore()
const selectedId = ref<number | null>(null)
const name = ref('')
const emit = defineEmits<{ select: [id: number | null] }>()

function selectTag(id: number) { selectedId.value = selectedId.value === id ? null : id; emit('select', selectedId.value) }
async function addTag() {
  const n = name.value.trim()
  if (n) { await tagStore.create(n); name.value = '' }
}
</script>

<style scoped>
.tag-list-wrap { padding: 0 var(--space-1); }
.tags { display: flex; flex-wrap: wrap; gap: var(--space-1); margin-bottom: var(--space-1); }
.tag {
  font-size: var(--font-size-xs); padding: 2px 10px; border-radius: var(--radius-full);
  cursor: pointer; user-select: none; transition: all var(--duration-fast) var(--ease-out);
  background: var(--bg); color: var(--text-secondary); border: 1px solid transparent;
}
.tag:hover { color: var(--accent); border-color: var(--accent); }
.tag.active { background: var(--accent); color: var(--accent-foreground); border-color: var(--accent); font-weight: var(--weight-medium); }
.tag-input { width: 100%; font-size: var(--font-size-xs); height: 26px; border-color: transparent; background: var(--bg); }
.tag-input:focus { border-color: var(--border-focus); background: var(--surface); }
.tag-input::placeholder { color: var(--text-muted); }
</style>
