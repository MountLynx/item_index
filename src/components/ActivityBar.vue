<template>
  <nav class="act-bar">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-btn"
      :class="{ active: active === tab.id }"
      :title="tab.title"
      @click="$emit('select', tab.id)"
    >
      <TablerIcon :name="tab.icon" :size="20" />
    </button>
  </nav>
</template>

<script setup lang="ts">
import TablerIcon from './TablerIcon.vue'

defineProps<{
  tabs: { id: string; icon: string; title: string }[]
  active: string
}>()

defineEmits<{
  select: [tabId: string]
}>()
</script>

<style scoped>
.act-bar {
  width: 48px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
  border-left: 1px solid var(--border);
  background: var(--surface);
}

.tab-btn {
  width: 36px;
  height: 36px;
  padding: 0;
  border: none;
  border-radius: var(--r-md);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--fast) var(--ease);
}

.tab-btn:hover {
  color: var(--text-secondary);
  background: var(--surface-hover);
}

.tab-btn.active {
  color: var(--accent);
  background: var(--accent-subtle);
}
</style>
