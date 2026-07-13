<template>
  <div class="create-tile" @click="showMenu = !showMenu">
    <div class="create-plus">+</div>
    <div class="create-label">{{ $t('dashboard.createOrImport') }}</div>
    <div v-if="showMenu" class="create-menu" @click.stop>
      <button @click="showMenu = false; $emit('create')">
        <TablerIcon name="plus" :size="15" /> {{ $t('dashboard.newRepo') }}
      </button>
      <button @click="showMenu = false; $emit('import')">
        <TablerIcon name="folder-open" :size="15" /> {{ $t('dashboard.importRepo') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TablerIcon from './TablerIcon.vue'

defineEmits<{ create: []; import: [] }>()
const showMenu = ref(false)
</script>

<style scoped>
.create-tile {
  width: 180px; height: 140px;
  border: 2px dashed var(--border-strong);
  border-radius: var(--r-lg);
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 6px;
  cursor: pointer;
  position: relative;
  transition: border-color var(--fast) var(--ease), background var(--fast) var(--ease);
  user-select: none;
}
.create-tile:hover {
  border-color: var(--accent);
  background: var(--accent-subtle);
}
.create-plus {
  font-size: 32px; font-weight: 300;
  color: var(--text-muted); line-height: 1;
  transition: color var(--fast) var(--ease);
}
.create-tile:hover .create-plus { color: var(--accent); }
.create-label {
  font-size: var(--fs-sm);
  color: var(--text-secondary);
}
.create-menu {
  position: absolute; top: 100%; left: 50%;
  transform: translateX(-50%);
  margin-top: 8px;
  background: var(--bg);
  border: 1px solid var(--border-strong);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-md);
  padding: 4px;
  z-index: 10;
  min-width: 180px;
}
.create-menu button {
  display: flex; align-items: center; gap: 8px;
  width: 100%; text-align: left;
  padding: 8px 12px;
  border: none; background: transparent;
  font-size: var(--fs-sm); cursor: pointer;
  color: var(--text);
  border-radius: var(--r-sm);
  height: auto;
}
.create-menu button:hover { background: var(--surface-hover); }
</style>
