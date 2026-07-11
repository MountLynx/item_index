import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ItemType, Field } from '@/types/bindings'

export const useTypeStore = defineStore('types', () => {
  const types = ref<ItemType[]>([])
  const loading = ref(false)

  const getTypeById = computed(() => (id: number) => types.value.find(t => t.id === id))

  async function fetchAll(): Promise<void> {
    loading.value = true
    try {
      types.value = await invoke<ItemType[]>('list_item_types')
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, icon?: string): Promise<ItemType> {
    const t = await invoke<ItemType>('create_item_type', { name, icon: icon ?? null })
    types.value.push(t)
    return t
  }

  async function remove(id: number): Promise<void> {
    await invoke('delete_item_type', { id })
    types.value = types.value.filter(t => t.id !== id)
  }

  async function addField(typeId: number, name: string, fieldType: string): Promise<Field> {
    const f = await invoke<Field>('add_field', { typeId, name, fieldType })
    const t = types.value.find(t => t.id === typeId)
    if (t) t.fields.push(f)
    return f
  }

  async function removeField(fieldId: number): Promise<void> {
    await invoke('remove_field', { fieldId })
    for (const t of types.value) {
      t.fields = t.fields.filter(f => f.id !== fieldId)
    }
  }

  async function reorderFields(typeId: number, fieldIds: number[]): Promise<void> {
    await invoke('reorder_fields', { typeId, fieldIds })
    const t = types.value.find(t => t.id === typeId)
    if (t) {
      t.fields.sort((a, b) => fieldIds.indexOf(a.id) - fieldIds.indexOf(b.id))
    }
  }

  return { types, loading, getTypeById, fetchAll, create, remove, addField, removeField, reorderFields }
})
