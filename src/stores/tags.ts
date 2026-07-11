import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Tag } from '@/types/bindings'

export const useTagStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([])

  async function fetchAll(): Promise<void> {
    tags.value = await invoke<Tag[]>('list_tags')
  }

  async function create(name: string): Promise<Tag> {
    const tag = await invoke<Tag>('create_tag', { name })
    tags.value.push(tag)
    return tag
  }

  async function remove(id: number): Promise<void> {
    await invoke('delete_tag', { id })
    tags.value = tags.value.filter(t => t.id !== id)
  }

  async function addToItem(itemId: string, tagId: number): Promise<void> {
    await invoke('add_tag_to_item', { itemId, tagId })
  }

  async function removeFromItem(itemId: string, tagId: number): Promise<void> {
    await invoke('remove_tag_from_item', { itemId, tagId })
  }

  return { tags, fetchAll, create, remove, addToItem, removeFromItem }
})
