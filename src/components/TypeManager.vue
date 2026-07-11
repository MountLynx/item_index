<template>
  <div class="tm">
    <!-- ═══ Level 1: Type List ═══ -->
    <template v-if="view === 'list'">
      <div class="tm-header">
        <h3>类别管理</h3>
      </div>

      <div class="type-list">
        <div
          v-for="t in typeStore.types"
          :key="t.id"
          class="type-row"
          :class="{ editing: inlineEditId === t.id }"
        >
          <!-- Normal state -->
          <template v-if="inlineEditId !== t.id">
            <div class="type-row-main" @click="enterFieldView(t.id)">
              <TablerIcon :name="t.icon" :size="18" />
              <span class="type-name">{{ t.name }}</span>
              <span v-if="t.id <= 2" class="preset-badge">预设</span>
            </div>
            <div class="type-row-actions">
              <button
                class="icon-btn sm"
                title="编辑"
                @click.stop="startEditType(t)"
              >
                <TablerIcon name="pencil" :size="14" />
              </button>
              <button
                v-if="t.id > 2"
                class="icon-btn sm danger"
                title="删除"
                @click.stop="deleteTarget = { kind: 'type', id: t.id, name: t.name }"
              >
                <TablerIcon name="dots" :size="14" />
              </button>
            </div>
          </template>

          <!-- Inline edit state -->
          <template v-else>
            <div class="edit-row">
              <input
                v-model="editIcon"
                class="icon-input"
                placeholder="图标"
                @keydown.enter="saveEditType(t.id)"
                @keydown.escape="cancelEditType"
              />
              <input
                v-model="editName"
                class="name-input"
                placeholder="类别名称"
                @keydown.enter="saveEditType(t.id)"
                @keydown.escape="cancelEditType"
              />
              <button class="icon-btn sm primary" @click="saveEditType(t.id)">
                <TablerIcon name="check" :size="14" />
              </button>
              <button class="icon-btn sm" @click="cancelEditType">
                <TablerIcon name="x" :size="14" />
              </button>
            </div>
          </template>
        </div>
      </div>

      <!-- New type creation row -->
      <div class="new-row">
        <template v-if="!showNewType">
          <button class="add-btn" @click="showNewType = true">
            <TablerIcon name="plus" :size="14" /> 新建类别
          </button>
        </template>
        <template v-else>
          <div class="edit-row">
            <input
              v-model="newIcon"
              class="icon-input"
              placeholder="图标"
              @keydown.enter="createType"
              @keydown.escape="showNewType = false"
            />
            <input
              v-model="newName"
              class="name-input"
              placeholder="类别名称"
              @keydown.enter="createType"
              @keydown.escape="showNewType = false"
            />
            <button class="icon-btn sm primary" @click="createType">
              <TablerIcon name="check" :size="14" />
            </button>
            <button class="icon-btn sm" @click="showNewType = false; newName = ''; newIcon = ''">
              <TablerIcon name="x" :size="14" />
            </button>
          </div>
        </template>
      </div>
    </template>

    <!-- ═══ Level 2: Field View ═══ -->
    <template v-else-if="view === 'field' && currentType">
      <!-- Breadcrumb -->
      <div class="tm-header">
        <button class="back-btn" @click="backToList">
          <TablerIcon name="arrow-left" :size="16" />
        </button>
        <h3>{{ currentType.name }}</h3>
      </div>

      <!-- Category edit area -->
      <div class="cat-edit-section">
        <div class="section-label">类别信息</div>
        <template v-if="!editingCategory">
          <div class="cat-display" @click="startEditCategory">
            <TablerIcon :name="currentType.icon" :size="18" />
            <span>{{ currentType.name }}</span>
            <button class="icon-btn sm">
              <TablerIcon name="pencil" :size="13" />
            </button>
          </div>
        </template>
        <template v-else>
          <div class="edit-row">
            <input
              v-model="catIcon"
              class="icon-input"
              placeholder="图标"
              @keydown.enter="saveCategory"
              @keydown.escape="editingCategory = false"
            />
            <input
              v-model="catName"
              class="name-input"
              placeholder="类别名称"
              @keydown.enter="saveCategory"
              @keydown.escape="editingCategory = false"
            />
            <button class="icon-btn sm primary" @click="saveCategory">
              <TablerIcon name="check" :size="14" />
            </button>
            <button class="icon-btn sm" @click="editingCategory = false">
              <TablerIcon name="x" :size="14" />
            </button>
          </div>
        </template>
      </div>

      <!-- Fields list -->
      <div class="fields-section">
        <div class="section-label">
          字段
          <span class="count">{{ currentType.fields.length }} 项</span>
        </div>

        <div v-if="currentType.fields.length === 0" class="empty-hint">
          暂无字段，点击下方添加
        </div>

        <div
          v-for="f in currentType.fields"
          :key="f.id"
          class="field-row"
          :class="{ editing: editingFieldId === f.id }"
          draggable="true"
          @dragstart="onDragStart($event, f.id)"
          @dragover.prevent="onDragOver($event, f.id)"
          @drop="onDrop($event, f.id)"
          @dragend="onDragEnd"
        >
          <!-- Normal state -->
          <template v-if="editingFieldId !== f.id">
            <div class="field-main" @dblclick="startEditField(f)">
              <span class="drag-handle">
                <TablerIcon name="grip-vertical" :size="14" />
              </span>
              <TablerIcon :name="f.icon" :size="16" />
              <span class="field-name">{{ f.name }}</span>
              <span class="type-tag">{{ f.field_type === 'text' ? '文本' : '复选框' }}</span>
              <span v-if="f.field_type === 'checkbox' && f.label" class="label-hint">{{ f.label }}</span>
            </div>
            <button
              class="icon-btn sm danger"
              title="删除字段"
              @click.stop="deleteTarget = { kind: 'field', id: f.id, name: f.name }"
            >
              <TablerIcon name="dots" :size="14" />
            </button>
          </template>

          <!-- Inline edit state -->
          <template v-else>
            <div class="field-edit-row">
              <input
                v-model="fieldEditIcon"
                class="icon-input-sm"
                placeholder="图标"
              />
              <input
                v-model="fieldEditName"
                class="name-input"
                placeholder="字段名"
              />
              <select v-model="fieldEditType" class="type-select">
                <option value="text">文本</option>
                <option value="checkbox">复选框</option>
              </select>
              <input
                v-if="fieldEditType === 'checkbox'"
                v-model="fieldEditLabel"
                class="label-input"
                placeholder="显示文字（可选）"
              />
              <button class="icon-btn sm primary" @click="saveEditField(f.id)">
                <TablerIcon name="check" :size="14" />
              </button>
              <button class="icon-btn sm" @click="cancelEditField">
                <TablerIcon name="x" :size="14" />
              </button>
            </div>
          </template>
        </div>
      </div>

      <!-- New field row -->
      <div class="new-row">
        <template v-if="!showNewField">
          <button class="add-btn" @click="showNewField = true">
            <TablerIcon name="plus" :size="14" /> 添加字段
          </button>
        </template>
        <template v-else>
          <div class="field-edit-row">
            <input
              v-model="newFieldIcon"
              class="icon-input-sm"
              placeholder="图标"
              @keydown.enter="addField"
              @keydown.escape="showNewField = false"
            />
            <input
              v-model="newFieldName"
              class="name-input"
              placeholder="字段名"
              @keydown.enter="addField"
              @keydown.escape="showNewField = false"
            />
            <select v-model="newFieldType" class="type-select">
              <option value="text">文本</option>
              <option value="checkbox">复选框</option>
            </select>
            <input
              v-if="newFieldType === 'checkbox'"
              v-model="newFieldLabel"
              class="label-input"
              placeholder="显示文字（可选）"
            />
            <button class="icon-btn sm primary" @click="addField">
              <TablerIcon name="check" :size="14" />
            </button>
            <button class="icon-btn sm" @click="showNewField = false; newFieldName = ''; newFieldIcon = ''; newFieldLabel = ''">
              <TablerIcon name="x" :size="14" />
            </button>
          </div>
        </template>
      </div>
    </template>

    <!-- Delete popover (rendered in both views) -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="popover-overlay" @click.self="deleteTarget = null">
        <div class="popover">
          <p class="popover-msg">
            确定删除{{ deleteTarget.kind === 'type' ? '类别' : '字段' }} "{{ deleteTarget.name }}" 吗？
          </p>
          <div class="popover-acts">
            <button class="sm" @click="deleteTarget = null">取消</button>
            <button class="sm danger" @click="confirmDelete">确认删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTypeStore } from '@/stores/types'
import { useToast } from '@/composables/toast'
import TablerIcon from './TablerIcon.vue'
import type { ItemType, Field } from '@/types/bindings'

const typeStore = useTypeStore()
const toast = useToast()

// ── View state ──
const view = ref<'list' | 'field'>('list')
const editingTypeId = ref<number | null>(null)

// ── Inline edit: type name/icon in list view ──
const inlineEditId = ref<number | null>(null)
const editName = ref('')
const editIcon = ref('')

// ── New type creation ──
const showNewType = ref(false)
const newName = ref('')
const newIcon = ref('')

// ── Category edit in field view ──
const editingCategory = ref(false)
const catName = ref('')
const catIcon = ref('')

// ── Field inline edit ──
const editingFieldId = ref<number | null>(null)
const fieldEditName = ref('')
const fieldEditIcon = ref('')
const fieldEditType = ref<'text' | 'checkbox'>('text')
const fieldEditLabel = ref('')

// ── New field ──
const showNewField = ref(false)
const newFieldName = ref('')
const newFieldIcon = ref('')
const newFieldType = ref<'text' | 'checkbox'>('text')
const newFieldLabel = ref('')

// ── Delete popover ──
const deleteTarget = ref<{ kind: 'type' | 'field'; id: number; name: string } | null>(null)

// ── Drag reorder ──
const dragId = ref<number | null>(null)
const dragOverId = ref<number | null>(null)

// ── Computed ──
const currentType = computed(() =>
  typeStore.types.find(t => t.id === editingTypeId.value) ?? null
)

// ── Navigation ──
function enterFieldView(typeId: number) {
  editingTypeId.value = typeId
  view.value = 'field'
}

function backToList() {
  view.value = 'list'
  editingTypeId.value = null
  editingCategory.value = false
  editingFieldId.value = null
  showNewField.value = false
}

// ── Type CRUD ──
function startEditType(t: ItemType) {
  inlineEditId.value = t.id
  editName.value = t.name
  editIcon.value = t.icon
}

function cancelEditType() {
  inlineEditId.value = null
  editName.value = ''
  editIcon.value = ''
}

async function saveEditType(id: number) {
  const name = editName.value.trim()
  if (!name) return
  try {
    await typeStore.update(id, name, editIcon.value || 'file')
    toast.success('类别已更新')
    inlineEditId.value = null
  } catch (e) {
    toast.error('更新失败: ' + e)
  }
}

async function createType() {
  const name = newName.value.trim()
  if (!name) return
  try {
    await typeStore.create(name, newIcon.value || 'file')
    toast.success('类别已创建')
    newName.value = ''
    newIcon.value = ''
    showNewType.value = false
  } catch (e) {
    toast.error('创建失败: ' + e)
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const { kind, id } = deleteTarget.value
  try {
    if (kind === 'type') {
      await typeStore.remove(id)
      toast.success('类别已删除')
    } else {
      await typeStore.removeField(id)
      toast.success('字段已删除')
    }
  } catch (e) {
    toast.error('删除失败: ' + e)
  }
  deleteTarget.value = null
}

// ── Category edit ──
function startEditCategory() {
  if (!currentType.value) return
  editingCategory.value = true
  catName.value = currentType.value.name
  catIcon.value = currentType.value.icon
}

async function saveCategory() {
  if (!currentType.value) return
  const name = catName.value.trim()
  if (!name) return
  try {
    await typeStore.update(currentType.value.id, name, catIcon.value || currentType.value.icon)
    toast.success('类别已更新')
    editingCategory.value = false
  } catch (e) {
    toast.error('更新失败: ' + e)
  }
}

// ── Field CRUD ──
function startEditField(f: Field) {
  editingFieldId.value = f.id
  fieldEditName.value = f.name
  fieldEditIcon.value = f.icon
  fieldEditType.value = f.field_type as 'text' | 'checkbox'
  fieldEditLabel.value = f.label || ''
}

function cancelEditField() {
  editingFieldId.value = null
}

async function saveEditField(fieldId: number) {
  const name = fieldEditName.value.trim()
  if (!name) return
  try {
    await typeStore.updateField(
      fieldId,
      name,
      fieldEditType.value,
      fieldEditIcon.value || 'circle',
      fieldEditLabel.value || ''
    )
    toast.success('字段已更新')
    editingFieldId.value = null
  } catch (e) {
    toast.error('更新失败: ' + e)
  }
}

async function addField() {
  if (!currentType.value || !newFieldName.value.trim()) return
  try {
    await typeStore.addField(
      currentType.value.id,
      newFieldName.value.trim(),
      newFieldType.value,
      newFieldIcon.value || 'circle',
      newFieldLabel.value.trim() || undefined
    )
    toast.success('字段已添加')
    newFieldName.value = ''
    newFieldIcon.value = ''
    newFieldLabel.value = ''
    showNewField.value = false
  } catch (e) {
    toast.error('添加失败: ' + e)
  }
}

// ── Drag to reorder ──
function onDragStart(e: DragEvent, fieldId: number) {
  dragId.value = fieldId
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', String(fieldId))
  }
}

function onDragOver(_e: DragEvent, fieldId: number) {
  if (dragId.value !== null && dragId.value !== fieldId) {
    dragOverId.value = fieldId
  }
}

async function onDrop(_e: DragEvent, _fieldId: number) {
  if (!currentType.value || dragId.value === null || dragOverId.value === null) return
  const fields = [...currentType.value.fields]
  const fromIdx = fields.findIndex(f => f.id === dragId.value)
  const toIdx = fields.findIndex(f => f.id === dragOverId.value)
  if (fromIdx === -1 || toIdx === -1 || fromIdx === toIdx) return

  const [moved] = fields.splice(fromIdx, 1)
  fields.splice(toIdx, 0, moved)
  const reordered = fields

  try {
    await typeStore.reorderFields(
      currentType.value.id,
      reordered.map(f => f.id)
    )
    // Apply to store only after server confirms
    currentType.value.fields = reordered
  } catch (e) {
    toast.error('排序失败: ' + e)
    // Local state unchanged — order snaps back
  }
  dragId.value = null
  dragOverId.value = null
}

function onDragEnd() {
  dragId.value = null
  dragOverId.value = null
}
</script>

<style scoped>
.tm {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}

.tm-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.tm-header h3 {
  font-size: var(--fs-lg);
  font-weight: var(--fw-semibold);
  margin: 0;
  flex: 1;
}

.back-btn {
  width: 28px; height: 28px; padding: 0;
  border: none; background: transparent;
  color: var(--text-secondary); cursor: pointer;
  border-radius: var(--r-sm);
  display: flex; align-items: center; justify-content: center;
}
.back-btn:hover { background: var(--surface-hover); color: var(--text); }

/* ── Type List ── */
.type-list { padding: 8px; display: flex; flex-direction: column; gap: 2px; }

.type-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 8px 12px; border-radius: var(--r-md);
  transition: background var(--fast) var(--ease);
}
.type-row:hover { background: var(--surface-hover); }
.type-row.editing { background: var(--surface-active); }

.type-row-main {
  display: flex; align-items: center; gap: 8px;
  flex: 1; min-width: 0; cursor: pointer;
}
.type-name { font-size: var(--fs-sm); font-weight: var(--fw-medium); }

.type-row-actions {
  display: flex; align-items: center; gap: 2px;
  opacity: 0; transition: opacity var(--fast) var(--ease);
}
.type-row:hover .type-row-actions { opacity: 1; }

.preset-badge {
  font-size: 10px; color: var(--text-muted);
  background: var(--bg); padding: 1px 6px;
  border-radius: var(--r-sm); border: 1px solid var(--border);
}

/* ── Edit Row ── */
.edit-row {
  display: flex; align-items: center; gap: 4px; width: 100%;
}
.icon-input {
  width: 80px; font-size: var(--fs-sm);
}
.name-input {
  flex: 1; font-size: var(--fs-sm);
}

.sm { height: 28px; }
.icon-btn { width: 28px; height: 28px; padding: 0; }
.icon-btn.primary { background: var(--accent); color: var(--accent-fg); border-color: var(--accent); }
.icon-btn.primary:hover { background: var(--accent-hover); }

/* ── Category Edit ── */
.cat-edit-section {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-light);
}
.section-label {
  font-size: var(--fs-xs); font-weight: var(--fw-semibold);
  color: var(--text-muted); text-transform: uppercase;
  letter-spacing: 0.05em; margin-bottom: 8px;
  display: flex; align-items: center; gap: 8px;
}
.count { font-weight: var(--fw-normal); text-transform: none; font-size: 11px; }

.cat-display {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 8px; border-radius: var(--r-md);
  cursor: pointer; transition: background var(--fast) var(--ease);
}
.cat-display:hover { background: var(--surface-hover); }
.cat-display span { font-size: var(--fs-sm); font-weight: var(--fw-medium); flex: 1; }

/* ── Fields ── */
.fields-section {
  padding: 8px 16px;
  flex: 1;
  overflow-y: auto;
}

.field-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 8px; border-radius: var(--r-md);
  transition: background var(--fast) var(--ease);
  margin-bottom: 2px;
}
.field-row:hover { background: var(--surface-hover); }
.field-row.editing { background: var(--surface-active); }

.field-main {
  display: flex; align-items: center; gap: 6px;
  flex: 1; min-width: 0;
}
.drag-handle {
  cursor: grab; color: var(--text-muted);
  display: flex; align-items: center;
}
.drag-handle:active { cursor: grabbing; }
.field-name { font-size: var(--fs-sm); }
.type-tag {
  font-size: 10px; color: var(--text-muted);
  background: var(--bg); padding: 0 6px;
  border-radius: var(--r-sm);
}
.label-hint {
  font-size: 10px; color: var(--text-secondary);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 80px;
}

.field-edit-row {
  display: flex; align-items: center; gap: 4px; width: 100%;
  flex-wrap: wrap;
}
.icon-input-sm { width: 60px; font-size: var(--fs-sm); }
.type-select { font-size: var(--fs-sm); width: 80px; }
.label-input { width: 140px; font-size: var(--fs-sm); }

.empty-hint {
  font-size: var(--fs-sm); color: var(--text-muted);
  text-align: center; padding: 24px 0;
}

/* ── New Row ── */
.new-row { padding: 8px 16px 16px; }
.add-btn {
  display: flex; align-items: center; gap: 4px;
  font-size: var(--fs-sm); color: var(--text-muted);
  background: transparent; border: 1px dashed var(--border);
  border-radius: var(--r-md); padding: 6px 12px; cursor: pointer;
  width: 100%; justify-content: center;
  transition: all var(--fast) var(--ease);
}
.add-btn:hover { color: var(--accent); border-color: var(--accent); }

/* ── Popover ── */
.popover-overlay {
  position: fixed; inset: 0; z-index: 300;
  background: transparent;
}
.popover {
  position: fixed; top: 50%; left: 50%;
  transform: translate(-50%, -50%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  padding: 16px 20px;
  box-shadow: var(--shadow-lg);
  min-width: 240px;
}
.popover-msg {
  font-size: var(--fs-sm); margin: 0 0 12px; color: var(--text);
}
.popover-acts {
  display: flex; justify-content: flex-end; gap: 8px;
}
.popover-acts .sm { font-size: var(--fs-xs); height: 28px; }
.danger { color: var(--danger); }
</style>
