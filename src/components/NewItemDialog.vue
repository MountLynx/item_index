<template>
  <Teleport to="body">
    <div class="overlay" @click.self="$emit('close')">
      <div class="modal">
        <h3>{{ $t('itemDialog.title') }}</h3>
        <div class="fg">
          <label>{{ $t('itemDialog.type') }}</label>
          <select v-model="typeId">
            <option v-for="t in typeStore.types" :key="t.id" :value="t.id">{{ t.name }}</option>
          </select>
        </div>
        <div class="fg">
          <label>{{ $t('itemDialog.name') }}</label>
          <input v-model="name" @keydown.enter="create" :placeholder="$t('itemDialog.name')" autofocus />
        </div>
        <div class="acts">
          <button class="ghost" @click="$emit('close')">{{ $t('itemDialog.cancel') }}</button>
          <button class="primary" @click="create" :disabled="!name.trim()">{{ $t('itemDialog.create') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTypeStore } from '@/stores/types'
import { useItemStore } from '@/stores/items'
const typeStore = useTypeStore()
const itemStore = useItemStore()
const typeId = ref(typeStore.types[0]?.id || 1)
const name = ref('')
const emit = defineEmits<{ close: [] }>()
async function create() { if (name.value.trim()) { await itemStore.create(typeId.value, name.value.trim()); emit('close') } }
</script>

<style scoped>
.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.2); display: flex; align-items: center; justify-content: center; z-index: 200; }
.modal { background: var(--surface); border-radius: var(--r-xl); padding: 24px; min-width: 340px; box-shadow: var(--shadow-lg); border: 1px solid var(--border); }
h3 { font-size: var(--fs-lg); font-weight: var(--fw-semibold); margin: 0 0 16px; }
.fg { margin-bottom: 12px; }
.fg label { display: block; font-size: var(--fs-xs); color: var(--text-secondary); margin-bottom: 4px; font-weight: var(--fw-medium); }
.fg input, .fg select { width: 100%; }
.acts { display: flex; gap: 8px; justify-content: flex-end; margin-top: 20px; }
</style>
