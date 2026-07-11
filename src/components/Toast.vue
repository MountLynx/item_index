<template>
  <Teleport to="body">
    <div class="toast-stack">
      <TransitionGroup name="toast">
        <div v-for="t in items" :key="t.id" class="toast" :class="t.type">
          <span>{{ t.message }}</span>
          <button class="close-btn" @click="remove(t.id)">×</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface T { id: number; message: string; type: 'success' | 'error' | 'info' }
const items = ref<T[]>([])
let nextId = 0

function add(message: string, type: T['type']) {
  const id = nextId++
  items.value.push({ id, message, type })
  setTimeout(() => remove(id), 3500)
}
function remove(id: number) { items.value = items.value.filter(t => t.id !== id) }

defineExpose({ success: (m: string) => add(m, 'success'), error: (m: string) => add(m, 'error'), info: (m: string) => add(m, 'info') })
</script>

<style scoped>
.toast-stack { position: fixed; bottom: var(--space-4); right: var(--space-4); display: flex; flex-direction: column; gap: var(--space-2); z-index: 300; pointer-events: none; }
.toast {
  display: flex; align-items: center; gap: var(--space-2); padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-lg); font-size: var(--font-size-sm); font-weight: var(--weight-medium);
  box-shadow: var(--shadow-lg); pointer-events: auto; min-width: 200px;
}
.toast.success { background: var(--success); color: #fff; }
.toast.error { background: var(--danger); color: #fff; }
.toast.info { background: var(--surface-raised); color: var(--text-primary); border: 1px solid var(--border); }
.close-btn { background: none; border: none; color: inherit; font-size: 18px; cursor: pointer; padding: 0; line-height: 1; opacity: 0.7; }
.close-btn:hover { opacity: 1; }

.toast-enter-active { transition: all var(--duration-slow) var(--ease-out); }
.toast-leave-active { transition: all var(--duration-fast) var(--ease-in); }
.toast-enter-from { opacity: 0; transform: translateY(12px) scale(0.95); }
.toast-leave-to { opacity: 0; transform: translateY(-8px) scale(0.95); }
</style>
