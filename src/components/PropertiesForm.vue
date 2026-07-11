<template>
  <div class="properties-form" v-if="detail">
    <div class="field" v-for="field in detail.item_type.fields" :key="field.id">
      <label>{{ field.name }}</label>
      <input v-if="field.field_type === 'text'" :value="getValue(field.name)" @input="setValue(field.name, ($event.target as HTMLInputElement).value)" @blur="save" />
      <input v-else-if="field.field_type === 'checkbox'" type="checkbox" :checked="!!getValue(field.name)" @change="toggleCheck(field.name)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useItemStore } from '@/stores/items'
import type { ItemDetail } from '@/types/bindings'

const props = defineProps<{ detail: ItemDetail }>()
const itemStore = useItemStore()

function getValue(name: string): string {
  const p = props.detail?.item.properties as Record<string, unknown>
  return String(p?.[name] ?? '')
}

function setValue(name: string, value: string) {
  const p = props.detail!.item.properties as Record<string, unknown>
  p[name] = value
}

function toggleCheck(name: string) {
  const p = props.detail!.item.properties as Record<string, unknown>
  p[name] = !p[name]
  save()
}

function save() {
  const p = props.detail!.item.properties as Record<string, unknown>
  itemStore.saveProperties(props.detail!.item.id, { ...p })
}
</script>

<style scoped>
.properties-form { padding: 8px 0; }
.field { margin-bottom: 8px; }
.field label { display: block; font-size: 11px; color: var(--text-secondary); margin-bottom: 2px; }
.field input[type="text"] { width: 100%; }
.field input[type="checkbox"] { width: 18px; height: 18px; cursor: pointer; }
</style>
