import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Item, ItemDetail, QueryParams, QueryResult } from '@/types/bindings'

let saveTimer: ReturnType<typeof setTimeout> | null = null

export const useItemStore = defineStore('items', () => {
  const items = ref<Item[]>([])
  const selectedId = ref<string | null>(null)
  const detail = ref<ItemDetail | null>(null)
  const loadingDetail = ref(false)
  const subRepoMap = ref<Record<string, string>>({})

  async function fetchList(groupId?: number | null, tagId?: number | null, typeIds?: number[]): Promise<void> {
    // Auto-resolve typeIds from active workspace when not explicitly provided
    // (centralized workspace filter — all callers benefit without manual wiring)
    let resolvedTypeIds = typeIds
    if (typeIds === undefined) {
      try {
        const { useWorkspaceStore } = await import('@/stores/workspace')
        const { useTypeStore } = await import('@/stores/types')
        const ws = useWorkspaceStore()
        const itemTypes = ws.active?.itemTypes
        if (itemTypes && itemTypes.length > 0) {
          const typeStore = useTypeStore()
          const ids = itemTypes
            .map((tn: string) => typeStore.types.find((t: any) => t.name === tn)?.id)
            .filter((id: any): id is number => id !== undefined)
          if (ids.length > 0) resolvedTypeIds = ids
        }
      } catch { /* ignore — no workspace loaded, show all */ }
    }
    items.value = await invoke<Item[]>('list_items', {
      groupId: groupId ?? null,
      tagId: tagId ?? null,
      typeIds: resolvedTypeIds ?? null,
    })
    await fetchSubRepos()
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

  async function query(params: QueryParams): Promise<QueryResult> {
    return invoke<QueryResult>('query_items', { ...params })
  }

  async function openItemFolder(id: string): Promise<void> {
    await invoke('open_item_folder', { itemId: id })
  }

  async function createSubRepo(id: string): Promise<void> {
    await invoke('create_sub_repo', { itemId: id })
    await fetchSubRepos()
  }

  async function openSubRepoWindow(id: string): Promise<void> {
    await invoke('open_sub_repo_window', { itemId: id })
  }

  async function fetchSubRepos(): Promise<void> {
    try {
      subRepoMap.value = await invoke<Record<string, string>>('list_sub_repos')
    } catch {
      subRepoMap.value = {}
    }
  }

  return { items, selectedId, detail, loadingDetail, subRepoMap, fetchList, select, clearSelection, create, update, remove, saveProperties, query, openItemFolder, createSubRepo, openSubRepoWindow, fetchSubRepos }
})
