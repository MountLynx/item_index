<template>
  <Teleport to="body">
    <div class="overlay" @click.self="$emit('close')">
      <div class="modal">
        <h3 class="modal-title">新建条目</h3>
        <div class="form-group">
          <label>类型</label>
          <select v-model="typeId">
            <option v-for="t in typeStore.types" :key="t.id" :value="t.id">{{ t.icon }} {{ t.name }}</option>
          </select>
        </div>
        <div class="form-group">
          <label>名称</label>
          <input v-model="name" @keydown.enter="create" placeholder="条目名称" autofocus />
        </div>
        <div class="modal-actions">
          <button class="ghost" @click="$emit('close')">取消</button>
          <button class="primary" @click="create" :disabled="!name.trim()">创建</button>
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
.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.3); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 200; }
.modal { background: var(--surface-raised); border-radius: var(--radius-xl); padding: var(--space-6); min-width: 360px; box-shadow: var(--shadow-xl); border: 1px solid var(--border); }
.modal-title { font-size: var(--font-size-lg); font-weight: var(--weight-semibold); margin: 0 0 var(--space-4); }
.form-group { margin-bottom: var(--space-3); }
.form-group label { display: block; font-size: var(--font-size-xs); color: var(--text-muted); margin-bottom: var(--space-1); font-weight: var(--weight-medium); }
.form-group input, .form-group select { width: 100%; }
.modal-actions { display: flex; gap: var(--space-2); justify-content: flex-end; margin-top: var(--space-5); }
</style>
