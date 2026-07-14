import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Item, ItemDetail } from '@/types/bindings'

let saveTimer: ReturnType<typeof setTimeout> | null = null

export const useItemStore = defineStore('items', () => {
  const items = ref<Item[]>([])
  const selectedId = ref<string | null>(null)
  const detail = ref<ItemDetail | null>(null)
  const loadingDetail = ref(false)

  async function fetchList(groupId?: number | null, tagId?: number | null, typeIds?: number[]): Promise<void> {
    items.value = await invoke<Item[]>('list_items', {
      groupId: groupId ?? null,
      tagId: tagId ?? null,
      typeIds: typeIds ?? null,
    })
  }

  async function select(id: string): Promise<void> {
    selectedId.value = id
    loadingDetail.value = true
    try {
      detail.value = await invoke<ItemDetail>('get_item', { id })
    } finally {
      loadingDetail.value = false
    }
  }

  function clearSelection(): void {
    selectedId.value = null
    detail.value = null
  }

  async function create(typeId: number, name: string): Promise<Item> {
    const item = await invoke<Item>('create_item', { typeId, name })
    items.value.unshift(item)
    return item
  }

  async function update(id: string, data: { name?: string; properties?: Record<string, unknown> }): Promise<Item> {
    const item = await invoke<Item>('update_item', { id, ...data })
    // Update in list
    const idx = items.value.findIndex(i => i.id === id)
    if (idx !== -1) items.value[idx] = item
    // Update detail if viewing
    if (detail.value && detail.value.item.id === id) {
      detail.value.item = item
    }
    return item
  }

  async function remove(id: string): Promise<void> {
    await invoke('delete_item', { id })
    items.value = items.value.filter(i => i.id !== id)
    if (selectedId.value === id) clearSelection()
  }

  // Auto-save properties with 500ms debounce
  function saveProperties(id: string, properties: Record<string, unknown>): void {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(async () => {
      await update(id, { properties })
    }, 500)
  }

  return { items, selectedId, detail, loadingDetail, fetchList, select, clearSelection, create, update, remove, saveProperties }
})
