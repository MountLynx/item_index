import { computed, type ComputedRef } from 'vue'
import { useItemStore } from '@/stores/items'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import type { Item, PluginManifest, ItemType, Group, Tag } from '@/types/bindings'

export interface PluginContext {
  items: ComputedRef<Item[]>
  itemTypes: ComputedRef<ItemType[]>
  groups: ComputedRef<Group[]>
  tags: ComputedRef<Tag[]>
  selectItem: (id: string) => void
  openDetail: (id: string) => void
  refreshItems: () => Promise<void>
  config: Record<string, unknown>
  filteredOut: ComputedRef<{ count: number; reason: string }>
}

export function buildPluginContext(
  manifest: PluginManifest,
  pluginConfig: Record<string, unknown> = {},
): PluginContext {
  const itemStore = useItemStore()
  const typeStore = useTypeStore()
  const groupStore = useGroupStore()
  const tagStore = useTagStore()

  const allItems = computed(() => itemStore.items)

  // Filter items by requiresFields
  const filteredItems = computed(() => {
    if (!manifest.requiresFields || manifest.requiresFields.length === 0) {
      return allItems.value
    }
    return allItems.value.filter(item => {
      const t = typeStore.types.find(tp => tp.id === item.type_id)
      if (!t) return false
      const props = item.properties as Record<string, unknown>
      return manifest.requiresFields.some(ff =>
        t.fields.some(f => f.field_type === ff && props[f.name] != null && props[f.name] !== '')
      )
    })
  })

  const filteredOut = computed(() => {
    if (!manifest.requiresFields || manifest.requiresFields.length === 0) {
      return { count: 0, reason: '' }
    }
    try {
      const count = allItems.value.length - filteredItems.value.length
      const fieldNames = manifest.requiresFields.map(f => {
        switch (f) {
          case 'date': return '日期'
          case 'number': return '数字'
          case 'text': return '文本'
          case 'checkbox': return '复选框'
          default: return f
        }
      }).join('/')
      return { count, reason: `缺少${fieldNames}字段` }
    } catch {
      return { count: 0, reason: '' }
    }
  })

  return {
    items: filteredItems,
    itemTypes: computed(() => typeStore.types),
    groups: computed(() => groupStore.tree),
    tags: computed(() => tagStore.tags),
    selectItem: (id) => { itemStore.select(id) },
    openDetail: (id) => { itemStore.select(id) },
    refreshItems: async () => { await itemStore.fetchList() },
    config: pluginConfig,
    filteredOut,
  }
}
