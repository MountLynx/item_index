<template>
  <div>
    <div class="tag-list">
      <span v-for="tag in tagStore.tags" :key="tag.id" class="tag" :class="{ selected: tag.id === selectedId }" @click="selectTag(tag.id)">
        # {{ tag.name }}
      </span>
    </div>
    <div class="new-tag">
      <input v-model="newName" placeholder="+ 新建标签" @keydown.enter="addTag" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTagStore } from '@/stores/tags'

const tagStore = useTagStore()

const selectedId = ref<number | null>(null)
const newName = ref('')

const emit = defineEmits<{ select: [id: number | null] }>()

function selectTag(id: number) {
  selectedId.value = selectedId.value === id ? null : id
  emit('select', selectedId.value)
}

async function addTag() {
  const name = newName.value.trim()
  if (name) {
    await tagStore.create(name)
    newName.value = ''
  }
}
</script>

<style scoped>
.tag-list { display: flex; flex-wrap: wrap; gap: 4px; padding: 4px; }
.tag {
  font-size: 12px; padding: 2px 8px; border-radius: 10px; cursor: pointer;
  background: var(--bg); border: 1px solid var(--border); user-select: none;
}
.tag:hover { border-color: var(--accent); }
.tag.selected { background: var(--accent); color: #fff; border-color: var(--accent); }
.new-tag input { width: 100%; font-size: 12px; padding: 2px 6px; margin-top: 4px; }
</style>
