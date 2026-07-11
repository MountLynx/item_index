<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="$emit('close')">
      <div class="dialog">
        <h3>新建条目</h3>
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
        <div class="actions">
          <button class="primary" @click="create" :disabled="!name.trim()">创建</button>
          <button @click="$emit('close')">取消</button>
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

async function create() {
  if (name.value.trim()) {
    await itemStore.create(typeId.value, name.value.trim())
    emit('close')
  }
}
</script>

<style scoped>
.dialog-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; z-index: 200; }
.dialog { background: var(--surface); border-radius: 8px; padding: 24px; min-width: 320px; box-shadow: 0 4px 24px rgba(0,0,0,0.2); }
h3 { margin-bottom: 16px; }
.form-group { margin-bottom: 12px; }
.form-group label { display: block; font-size: 12px; color: var(--text-secondary); margin-bottom: 4px; }
.form-group input, .form-group select { width: 100%; padding: 6px 10px; }
.actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }
</style>
