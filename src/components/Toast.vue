<template>
  <Teleport to="body">
    <div class="toast-container">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.type">
        <span>{{ t.message }}</span>
        <button class="close" @click="removeToast(t.id)">×</button>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface ToastItem { id: number; message: string; type: 'success' | 'error' | 'info' }

const toasts = ref<ToastItem[]>([])
let nextId = 0

function addToast(message: string, type: 'success' | 'error' | 'info' = 'info') {
  const id = nextId++
  toasts.value.push({ id, message, type })
  setTimeout(() => removeToast(id), 3000)
}

function removeToast(id: number) {
  toasts.value = toasts.value.filter(t => t.id !== id)
}

defineExpose({ success: (m: string) => addToast(m, 'success'), error: (m: string) => addToast(m, 'error'), info: (m: string) => addToast(m, 'info') })
</script>

<style scoped>
.toast-container { position: fixed; bottom: 16px; right: 16px; display: flex; flex-direction: column; gap: 8px; z-index: 300; }
.toast { display: flex; align-items: center; gap: 8px; padding: 8px 16px; border-radius: 6px; font-size: 13px; box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
.toast.success { background: #16A34A; color: #fff; }
.toast.error { background: #DC2626; color: #fff; }
.toast.info { background: var(--accent); color: #fff; }
.close { background: none; border: none; color: inherit; font-size: 16px; cursor: pointer; padding: 0; }
</style>
