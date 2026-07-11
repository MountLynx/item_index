import { ref } from 'vue'

interface ToastItem { id: number; message: string; type: 'success' | 'error' | 'info' }

// Module-level shared state so all callers share the same toast stack
const items = ref<ToastItem[]>([])
let nextId = 0

export function useToast() {
  function add(msg: string, type: ToastItem['type']) {
    const id = nextId++
    items.value.push({ id, message: msg, type })
    setTimeout(() => remove(id), 3500)
  }

  function remove(id: number) {
    items.value = items.value.filter(t => t.id !== id)
  }

  return {
    items,
    success: (m: string) => add(m, 'success'),
    error: (m: string) => add(m, 'error'),
    info: (m: string) => add(m, 'info'),
    remove,
  }
}
