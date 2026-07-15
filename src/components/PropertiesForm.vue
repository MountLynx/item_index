<template>
  <div class="properties-form" v-if="detail">
    <div class="field" v-for="field in detail.item_type.fields" :key="field.id">
      <label class="field-label"><TablerIcon :name="field.icon" :size="14" /> {{ field.name }}</label>
      <div v-if="field.field_type === 'text'" class="field-input-wrap">
                <input :value="getValue(field.name)" @input="setValue(field.name, ($event.target as HTMLInputElement).value)" @blur="save" :placeholder="field.name" />
      </div>
      <div v-else-if="field.field_type === 'number'" class="field-input-wrap">
        <input type="number" :value="getValue(field.name)" @input="setValue(field.name, ($event.target as HTMLInputElement).value, 'number')" @blur="save" :placeholder="field.name" />
      </div>
      <div v-else-if="field.field_type === 'date'" class="field-input-wrap">
        <input type="date" :value="getDateValue(field.name)" @input="setValue(field.name, ($event.target as HTMLInputElement).value)" @blur="save" />
      </div>
      <label v-else-if="field.field_type === 'checkbox'" class="checkbox-wrap">
        <input type="checkbox" :checked="!!getValue(field.name)" @change="toggleCheck(field.name)" />
        <span class="check-label">{{ field.label || field.name }}</span>
      </label>
    </div>
    <div v-if="detail.item_type.fields.length === 0" class="text-muted" style="font-size:var(--font-size-xs)">{{ $t('propertiesForm.noCustomFields') }}</div>
  </div>
</template>

<script setup lang="ts">
import { useItemStore } from '@/stores/items'
import type { ItemDetail } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const props = defineProps<{ detail: ItemDetail }>()
const itemStore = useItemStore()

function getValue(name: string): string {
  const p = props.detail?.item.properties as Record<string, unknown>
  const v = p?.[name]
  if (v === null || v === undefined) return ''
  return String(v)
}
/** Extract YYYY-MM-DD from stored value (handles ISO strings, Bug #8) */
function getDateValue(name: string): string {
  const p = props.detail?.item.properties as Record<string, unknown>
  const v = p?.[name]
  if (!v) return ''
  const s = String(v)
  // Already YYYY-MM-DD
  if (/^\d{4}-\d{2}-\d{2}/.test(s)) return s.slice(0, 10)
  // Try parsing as ISO
  try {
    const d = new Date(s)
    if (!isNaN(d.getTime())) return d.toISOString().slice(0, 10)
  } catch {}
  return s.slice(0, 10)
}
function setValue(name: string, value: string, fieldType?: string) {
  const p = props.detail!.item.properties as Record<string, unknown>
  if (fieldType === 'number') {
    p[name] = value === '' ? null : Number(value)  // Bug #7: store as number
  } else if (fieldType === 'date') {
    p[name] = value  // YYYY-MM-DD from date input
  } else {
    p[name] = value
  }
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
.properties-form { display: flex; flex-direction: column; gap: 8px; }
.field-label { display: block; font-size: var(--fs-xs); color: var(--text-secondary); margin-bottom: 2px; font-weight: var(--fw-medium); }
.field-input-wrap input { width: 100%; }
.checkbox-wrap { display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 4px 0; }
.checkbox-wrap input[type="checkbox"] { width: 16px; height: 16px; cursor: pointer; accent-color: var(--accent); }
.check-label { font-size: var(--fs-sm); color: var(--text-secondary); }
</style>
