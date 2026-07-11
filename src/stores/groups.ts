import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Group } from '@/types/bindings'

export const useGroupStore = defineStore('groups', () => {
  const tree = ref<Group[]>([])

  async function fetchAll(): Promise<void> {
    tree.value = await invoke<Group[]>('list_groups')
  }

  async function create(name: string, parentId?: number | null): Promise<Group> {
    await invoke<Group>('create_group', { name, parentId: parentId ?? null })
    await fetchAll()
    return tree.value[0] // placeholder — full refresh is simpler
  }

  async function updateGroup(id: number, name: string): Promise<void> {
    await invoke<Group>('update_group', { id, name })
    await fetchAll()
  }

  async function remove(id: number): Promise<void> {
    await invoke('delete_group', { id })
    await fetchAll()
  }

  async function move(id: number, parentId: number | null, position: number): Promise<void> {
    await invoke('move_group', { id, parentId, position })
    await fetchAll()
  }

  async function addItemToGroup(itemId: string, groupId: number): Promise<void> {
    await invoke('add_item_to_group', { itemId, groupId })
  }

  async function removeItemFromGroup(itemId: string, groupId: number): Promise<void> {
    await invoke('remove_item_from_group', { itemId, groupId })
  }

  return { tree, fetchAll, create, update: updateGroup, remove, move, addItemToGroup, removeItemFromGroup }
})
