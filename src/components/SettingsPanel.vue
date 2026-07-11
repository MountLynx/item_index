<template>
  <Teleport to="body">
    <div class="dialog-overlay" @click.self="$emit('close')">
      <div class="dialog">
        <h3>⚙ 设置</h3>
        <div class="section">
          <h4>条目类型</h4>
          <div v-for="t in typeStore.types" :key="t.id" class="type-row" :class="{ sel: t.id === selectedTypeId }" @click="selectType(t.id)">
            <span>{{ t.icon }} {{ t.name }}</span>
            <button v-if="t.id > 2" class="danger small" @click.stop="removeType(t.id)">删除</button>
            <span v-else class="preset">预设</span>
          </div>
          <div class="new-type">
            <input v-model="newTypeName" placeholder="新类型名称" @keydown.enter="addType" />
            <button class="primary small" @click="addType">+ 添加</button>
          </div>
        </div>

        <div v-if="selectedType" class="section">
          <h4>{{ selectedType.icon }} {{ selectedType.name }} — 字段</h4>
          <div v-for="f in selectedType.fields" :key="f.id" class="field-row">
            <span>{{ f.name }} ({{ f.field_type }})</span>
            <button class="danger small" @click="removeField(f.id)">删除</button>
          </div>
          <div class="new-field">
            <input v-model="newFieldName" placeholder="字段名" />
            <select v-model="newFieldType">
              <option value="text">文本</option>
              <option value="checkbox">复选框</option>
            </select>
            <button class="primary small" @click="addField">+</button>
          </div>
        </div>
        <div class="actions">
          <button @click="$emit('close')">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTypeStore } from '@/stores/types'

const typeStore = useTypeStore()

const newTypeName = ref('')
const newFieldName = ref('')
const newFieldType = ref('text')
const selectedTypeId = ref<number | null>(null)

const selectedType = computed(() => typeStore.types.find(t => t.id === selectedTypeId.value))

function selectType(id: number) { selectedTypeId.value = selectedTypeId.value === id ? null : id }

async function addType() {
  if (newTypeName.value.trim()) {
    await typeStore.create(newTypeName.value.trim())
    newTypeName.value = ''
  }
}

async function removeType(id: number) {
  if (confirm('确定删除此类型？')) await typeStore.remove(id)
}

async function addField() {
  if (selectedTypeId.value && newFieldName.value.trim()) {
    await typeStore.addField(selectedTypeId.value, newFieldName.value.trim(), newFieldType.value)
    newFieldName.value = ''
  }
}

async function removeField(fieldId: number) {
  await typeStore.removeField(fieldId)
}

defineEmits<{ close: [] }>()
</script>

<style scoped>
.dialog-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center; z-index: 200; }
.dialog { background: var(--surface); border-radius: 8px; padding: 24px; min-width: 400px; max-height: 80vh; overflow-y: auto; box-shadow: 0 4px 24px rgba(0,0,0,0.2); }
h3 { margin-bottom: 12px; }
h4 { font-size: 13px; margin: 12px 0 6px; color: var(--text-secondary); }
.type-row, .field-row { display: flex; align-items: center; justify-content: space-between; padding: 4px 6px; font-size: 13px; cursor: pointer; border-radius: 3px; }
.type-row:hover { background: var(--border); }
.type-row.sel { background: var(--accent); color: #fff; }
.type-row.sel .preset { color: rgba(255,255,255,0.6); }
.preset { font-size: 11px; color: var(--text-secondary); }
button.small { font-size: 11px; padding: 2px 8px; }
.new-type, .new-field { display: flex; gap: 4px; margin-top: 6px; }
.new-type input, .new-field input { flex: 1; }
.new-field select { width: 100px; }
.actions { margin-top: 16px; text-align: right; }
</style>
