<template>
  <div class="tm">
    <!-- ═══ Level 1: Type List ═══ -->
    <template v-if="view === 'list'">
      <div class="tm-header">
        <h3>{{ $t('typeManager.title') }}</h3>
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
              <span v-if="t.id <= 2" class="preset-badge">{{ $t('typeManager.preset') }}</span>
            </div>
            <div class="type-row-actions">
              <button
                class="icon-btn sm"
                :title="$t('typeManager.edit')"
                @click.stop="startEditType(t)"
              >
                <TablerIcon name="pencil" :size="14" />
              </button>
              <button
                v-if="t.id > 2"
                class="icon-btn sm danger"
                :title="$t('typeManager.delete')"
                @click.stop="deleteTarget = { kind: 'type', id: t.id, name: t.name }"
              >
                <TablerIcon name="dots" :size="14" />
              </button>
            </div>
          </template>

          <!-- Inline edit state -->
          <template v-else>
            <div class="edit-row">
              <IconPicker v-model="editIcon" />
              <input
                v-model="editName"
                class="name-input"
                :placeholder="$t('typeManager.typeName')"
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
            <TablerIcon name="plus" :size="14" /> {{ $t('typeManager.newType') }}
          </button>
        </template>
        <template v-else>
          <div class="edit-row">
            <IconPicker v-model="newIcon" />
            <input
              v-model="newName"
              class="name-input"
              :placeholder="$t('typeManager.typeName')"
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
        <div class="section-label">{{ $t('typeManager.typeInfo') }}</div>
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
            <IconPicker v-model="catIcon" />
            <input
              v-model="catName"
              class="name-input"
              :placeholder="$t('typeManager.typeName')"
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
          {{ $t('typeManager.fields') }}
          <span class="count">{{ currentType.fields.length }}{{ $t('typeManager.items') }}</span>
        </div>

        <div v-if="currentType.fields.length === 0" class="empty-hint">
          {{ $t('typeManager.noFields') }}
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
              <span class="type-tag">{{ $t('typeManager.' + f.field_type) }}</span>
              <span v-if="f.label" class="label-hint">{{ f.label }}</span>
            </div>
            <div class="field-row-actions">
              <button
                class="icon-btn sm"
                :title="$t('typeManager.edit')"
                @click.stop="startEditField(f)"
              >
                <TablerIcon name="pencil" :size="14" />
              </button>
              <button
                class="icon-btn sm danger"
                :title="$t('typeManager.delete')"
                @click.stop="deleteTarget = { kind: 'field', id: f.id, name: f.name }"
              >
                <TablerIcon name="dots" :size="14" />
              </button>
            </div>
          </template>

          <!-- Inline edit state -->
          <template v-else>
            <div class="field-edit-row">
              <IconPicker v-model="fieldEditIcon" />
              <input
                v-model="fieldEditName"
                class="name-input"
                :placeholder="$t('typeManager.fieldName')"
              />
              <select v-model="fieldEditType" class="type-select">
                <option value="text">{{ $t('typeManager.text') }}</option>
                <option value="checkbox">{{ $t('typeManager.checkbox') }}</option>
                <option value="date">{{ $t('typeManager.date') }}</option>
                <option value="number">{{ $t('typeManager.number') }}</option>
                <option value="dropdown">{{ $t('typeManager.dropdown') }}</option>
              </select>
              <input
                v-if="fieldEditType !== 'text' && fieldEditType !== 'dropdown'"
                v-model="fieldEditLabel"
                class="label-input"
                :placeholder="$t('typeManager.displayText')"
              />
              <input
                v-if="fieldEditType === 'dropdown'"
                v-model="fieldEditOptions"
                class="label-input"
                :placeholder="$t('typeManager.optionsHint')"
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
            <TablerIcon name="plus" :size="14" /> {{ $t('typeManager.addField') }}
          </button>
        </template>
        <template v-else>
          <div class="field-edit-row">
            <IconPicker v-model="newFieldIcon" />
            <input
              v-model="newFieldName"
              class="name-input"
              :placeholder="$t('typeManager.fieldName')"
              @keydown.enter="addField"
              @keydown.escape="showNewField = false"
            />
              <select v-model="newFieldType" class="type-select">
                <option value="text">{{ $t('typeManager.text') }}</option>
                <option value="checkbox">{{ $t('typeManager.checkbox') }}</option>
                <option value="date">{{ $t('typeManager.date') }}</option>
                <option value="number">{{ $t('typeManager.number') }}</option>
                <option value="dropdown">{{ $t('typeManager.dropdown') }}</option>
              </select>
              <input
                v-if="newFieldType !== 'text' && newFieldType !== 'dropdown'"
                v-model="newFieldLabel"
                class="label-input"
                :placeholder="$t('typeManager.displayText')"
              />
              <input
                v-if="newFieldType === 'dropdown'"
                v-model="newFieldOptions"
                class="label-input"
                :placeholder="$t('typeManager.optionsHint')"
              />
            <button class="icon-btn sm primary" @click="addField">
              <TablerIcon name="check" :size="14" />
            </button>
            <button class="icon-btn sm" @click="showNewField = false; newFieldName = ''; newFieldIcon = ''; newFieldLabel = ''; newFieldOptions = ''">
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
          <p class="popover-msg">{{ deleteMessage }}</p>
          <div class="popover-acts">
            <button class="sm" @click="deleteTarget = null">{{ $t('common.no') }}</button>
            <button class="sm danger" @click="confirmDelete">{{ $t('common.confirm') }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTypeStore } from '@/stores/types'
import { useToast } from '@/composables/toast'
import TablerIcon from './TablerIcon.vue'
import IconPicker from './IconPicker.vue'
import type { ItemType, Field } from '@/types/bindings'

const { t } = useI18n()
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
const fieldEditType = ref<'text' | 'checkbox' | 'date' | 'number' | 'dropdown'>('text')
const fieldEditLabel = ref('')
const fieldEditOptions = ref('')

// ── New field ──
const showNewField = ref(false)
const newFieldName = ref('')
const newFieldIcon = ref('')
const newFieldType = ref<'text' | 'checkbox' | 'date' | 'number' | 'dropdown'>('text')
const newFieldLabel = ref('')
const newFieldOptions = ref('')

// ── Delete popover ──
const deleteTarget = ref<{ kind: 'type' | 'field'; id: number; name: string } | null>(null)

// ── Drag reorder ──
const dragId = ref<number | null>(null)
const dragOverId = ref<number | null>(null)

// ── Computed ──
const currentType = computed(() =>
  typeStore.types.find(t => t.id === editingTypeId.value) ?? null
)
const deleteMessage = computed(() => {
  if (!deleteTarget.value) return ''
  const kind = deleteTarget.value.kind === 'type' ? t('typeManager.type') : t('typeManager.field')
  return t('typeManager.confirmDelete', { kind, name: deleteTarget.value.name })
})

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
    toast.success(t('typeManager.updated'))
    inlineEditId.value = null
  } catch (e) {
    toast.error(t('typeManager.updateFailed') + ': ' + e)
  }
}

async function createType() {
  const name = newName.value.trim()
  if (!name) return
  try {
    await typeStore.create(name, newIcon.value || 'file')
    toast.success(t('typeManager.created'))
    newName.value = ''
    newIcon.value = ''
    showNewType.value = false
  } catch (e) {
    toast.error(t('typeManager.createFailed') + ': ' + e)
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const { kind, id } = deleteTarget.value
  try {
    if (kind === 'type') {
      await typeStore.remove(id)
      toast.success(t('typeManager.deleted'))
    } else {
      await typeStore.removeField(id)
      toast.success(t('typeManager.fieldDeleted'))
    }
  } catch (e) {
    toast.error(t('typeManager.deleteFailed') + ': ' + e)
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
    toast.success(t('typeManager.updated'))
    editingCategory.value = false
  } catch (e) {
    toast.error(t('typeManager.updateFailed') + ': ' + e)
  }
}

// ── Field CRUD ──
function startEditField(f: Field) {
  editingFieldId.value = f.id
  fieldEditName.value = f.name
  fieldEditIcon.value = f.icon
  fieldEditType.value = f.field_type as 'text' | 'checkbox' | 'date' | 'number' | 'dropdown'
  fieldEditLabel.value = f.label || ''
  fieldEditOptions.value = (f.options || []).join(', ')
}

function cancelEditField() {
  editingFieldId.value = null
  fieldEditOptions.value = ''
}

async function saveEditField(fieldId: number) {
  const name = fieldEditName.value.trim()
  if (!name) return
  try {
    const opts = fieldEditType.value === 'dropdown'
      ? fieldEditOptions.value.split(',').map(s => s.trim()).filter(s => s)
      : undefined
    await typeStore.updateField(
      fieldId,
      name,
      fieldEditType.value,
      fieldEditIcon.value || 'circle',
      fieldEditLabel.value || '',
      opts
    )
    toast.success(t('typeManager.fieldUpdated'))
    editingFieldId.value = null
    fieldEditOptions.value = ''
  } catch (e) {
    toast.error(t('typeManager.updateFailed') + ': ' + e)
  }
}

async function addField() {
  if (!currentType.value || !newFieldName.value.trim()) return
  try {
    const opts = newFieldType.value === 'dropdown'
      ? newFieldOptions.value.split(',').map(s => s.trim()).filter(s => s)
      : undefined
    await typeStore.addField(
      currentType.value.id,
      newFieldName.value.trim(),
      newFieldType.value,
      newFieldIcon.value || 'circle',
      newFieldLabel.value.trim() || undefined,
      opts
    )
    toast.success(t('typeManager.fieldAdded'))
    newFieldName.value = ''
    newFieldIcon.value = ''
    newFieldLabel.value = ''
    newFieldOptions.value = ''
    showNewField.value = false
  } catch (e) {
    toast.error(t('typeManager.addFieldFailed') + ': ' + e)
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
    toast.error(t('typeManager.reorderFailed') + ': ' + e)
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

.field-row-actions {
  display: flex; align-items: center; gap: 2px;
  opacity: 0; transition: opacity var(--fast) var(--ease);
}
.field-row:hover .field-row-actions { opacity: 1; }

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
