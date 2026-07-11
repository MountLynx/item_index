<template>
  <Teleport to="body">
    <div class="overlay" @click.self="$emit('close')">
      <div class="panel">
        <h3 class="panel-title">⚙ 设置</h3>

        <div class="section">
          <h4>条目类型</h4>
          <div v-for="t in typeStore.types" :key="t.id" class="row clickable" :class="{ sel: t.id === selectedTypeId }" @click="selectType(t.id)">
            <span>{{ t.icon }} {{ t.name }}</span>
            <button v-if="t.id > 2" class="ghost sm danger-text" @click.stop="removeType(t.id)">删除</button>
            <span v-else class="badge">预设</span>
          </div>
          <div class="add-row">
            <input v-model="newTypeName" placeholder="新类型名称" @keydown.enter="addType" />
            <button class="primary sm" @click="addType">+ 添加</button>
          </div>
        </div>

        <div v-if="selectedType" class="section">
          <h4>{{ selectedType.icon }} {{ selectedType.name }} — 字段</h4>
          <div v-for="f in selectedType.fields" :key="f.id" class="row">
            <span class="text-muted">{{ f.field_type === 'checkbox' ? '☑' : '✎' }} {{ f.name }} <span class="type-tag">{{ f.field_type }}</span></span>
            <button class="ghost sm danger-text" @click="removeField(f.id)">删除</button>
          </div>
          <div class="add-row">
            <input v-model="newFieldName" placeholder="字段名" />
            <select v-model="newFieldType">
              <option value="text">文本</option>
              <option value="checkbox">复选框</option>
            </select>
            <button class="primary sm" @click="addField">+</button>
          </div>
        </div>

        <div class="panel-actions">
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
async function addType() { if (newTypeName.value.trim()) { await typeStore.create(newTypeName.value.trim()); newTypeName.value = '' } }
async function removeType(id: number) { if (confirm('确定删除此类型？')) await typeStore.remove(id) }
async function addField() { if (selectedTypeId.value && newFieldName.value.trim()) { await typeStore.addField(selectedTypeId.value, newFieldName.value.trim(), newFieldType.value); newFieldName.value = '' } }
async function removeField(fieldId: number) { await typeStore.removeField(fieldId) }

defineEmits<{ close: [] }>()
</script>

<style scoped>
.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.3); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 200; }
.panel { background: var(--surface-raised); border-radius: var(--radius-xl); padding: var(--space-6); min-width: 440px; max-height: 80vh; overflow-y: auto; box-shadow: var(--shadow-xl); border: 1px solid var(--border); }
.panel-title { font-size: var(--font-size-lg); font-weight: var(--weight-semibold); margin: 0 0 var(--space-4); }
.section { margin-bottom: var(--space-5); }
.section h4 { font-size: var(--font-size-sm); color: var(--text-secondary); margin: 0 0 var(--space-2); font-weight: var(--weight-semibold); }

.row { display: flex; align-items: center; justify-content: space-between; padding: var(--space-1) var(--space-2); font-size: var(--font-size-sm); border-radius: var(--radius-sm); }
.row.clickable { cursor: pointer; transition: background var(--duration-fast) var(--ease-out); }
.row.clickable:hover { background: var(--surface-hover); }
.row.sel { background: var(--accent); color: var(--accent-foreground); }
.row.sel .text-muted, .row.sel .type-tag, .row.sel .badge { color: rgba(255,255,255,0.7); }

.badge { font-size: var(--font-size-xs); color: var(--text-muted); }
.type-tag { font-size: 10px; color: var(--text-muted); background: var(--bg); padding: 0 4px; border-radius: 2px; }
.sm { font-size: var(--font-size-xs); height: 28px; }
.danger-text { color: var(--danger); border-color: transparent; }

.add-row { display: flex; gap: var(--space-1); margin-top: var(--space-2); }
.add-row input { flex: 1; }
.add-row select { width: 100px; }

.panel-actions { display: flex; justify-content: flex-end; margin-top: var(--space-4); }
</style>
