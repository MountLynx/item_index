# 条目类别管理重设计 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 SettingsPanel.vue 重建为 ActivityBar + TypeManager 架构，实现条目类别和字段的完整 CRUD、行内编辑、拖拽排序、Popover 删除确认、checkbox 可配置文字。

**Architecture:** 右侧面板区新增 ActivityBar（48px 竖排图标 Tab），RightPanel 改造为容器组件按 Tab 渲染 ItemDetail 或 TypeManager。TypeManager 内分两级视图（类别列表 → 字段管理），所有编辑操作通过行内展开完成。

**Tech Stack:** Vue 3 + TypeScript + Pinia + Tauri 2 + SQLite (sqlx) + Tabler Icons

## Global Constraints

- 保留现有设计令牌系统（`theme.css`），所有新 UI 使用 CSS 变量
- Tabler Icons 需注册到 `TablerIcon.vue` 的 import map
- 后端模型 Rust/TS 双端同步
- 预设类型（id 1, 2）不可删除但可编辑
- 删除确认使用 Popover，不使用 `window.confirm()`
- 操作反馈通过 `useToast()` composable

---

## File Structure

```
Create:
  src-tauri/migrations/20260711000004_add_field_label.sql
  src/components/ActivityBar.vue
  src/components/TypeManager.vue

Modify:
  src-tauri/src/models.rs          — Field + label
  src-tauri/src/commands/types.rs  — + update_item_type, update_field
  src-tauri/src/lib.rs             — register 2 new commands
  src/types/bindings.ts            — Field + label, new invoke signatures
  src/stores/types.ts              — + update, updateField
  src/components/RightPanel.vue    — container + ActivityBar
  src/components/PropertiesForm.vue — fix checkbox text
  src/components/Topbar.vue        — settings → category button
  src/components/TablerIcon.vue    — add new icons to map
  src/App.vue                      — activeRightTab state, wire props

Delete:
  src/components/SettingsPanel.vue
```

---

### Task 1: 数据库迁移 — fields 表添加 label 列

**Files:**
- Create: `src-tauri/migrations/20260711000004_add_field_label.sql`

**Interfaces:**
- Produces: `fields` 表新增 `label TEXT NOT NULL DEFAULT ''` 列，sqlx migrate 自动执行

- [ ] **Step 1: 创建迁移 SQL 文件**

```sql
-- Add label column to fields table for checkbox display text
ALTER TABLE fields ADD COLUMN label TEXT NOT NULL DEFAULT '';
```

Write to `src-tauri/migrations/20260711000004_add_field_label.sql`.

- [ ] **Step 2: 验证迁移文件存在**

Run: `ls src-tauri/migrations/`

Expected: 文件列表包含 `20260711000004_add_field_label.sql`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/migrations/20260711000004_add_field_label.sql
git commit -m "feat: add label column to fields table for checkbox display text"
```

---

### Task 2: 后端 — Field 模型更新 + 新增 update 命令

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands/types.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: Task 1 migration（运行时生效）
- Produces:
  - `Field` struct 新增 `pub label: String`
  - `update_item_type(state, id: i64, name: String, icon: String) -> Result<ItemType, String>`
  - `update_field(state, id: i64, name: String, field_type: String, icon: String, label: String) -> Result<Field, String>`

- [ ] **Step 1: 更新 Field 模型**

Edit `src-tauri/src/models.rs`，在 `Field` struct 末尾添加 `label` 字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: i64,
    pub type_id: i64,
    pub name: String,
    pub field_type: String,
    pub icon: String,
    pub position: i32,
    pub label: String,
}
```

- [ ] **Step 2: 更新 list_item_types 查询**

Edit `src-tauri/src/commands/types.rs`，更新 `list_item_types` 中的字段查询和映射。

在 `list_item_types` 函数中，将 field_rows 的类型从 `(i64, i64, String, String, String, i32)` 改为 `(i64, i64, String, String, String, i32, String)`，SQL 查询添加 `f.label`，Field 构造添加 `label`：

```rust
let field_rows: Vec<(i64, i64, String, String, String, i32, String)> = sqlx::query_as(
    "SELECT f.id, f.type_id, f.name, f.field_type, f.icon, f.position, f.label FROM fields f WHERE f.type_id = ? ORDER BY f.position",
)
.bind(id)
.fetch_all(&pool)
.await
.map_err(|e| e.to_string())?;

let fields = field_rows
    .into_iter()
    .map(|(fid, tid, fname, ftype, ficon, pos, label)| Field {
        id: fid,
        type_id: tid,
        name: fname,
        field_type: ftype,
        icon: ficon,
        position: pos,
        label,
    })
    .collect();
```

- [ ] **Step 3: 添加 update_item_type 命令**

在 `src-tauri/src/commands/types.rs` 文件末尾（`reorder_fields` 之后）追加：

```rust
#[tauri::command]
pub async fn update_item_type(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    icon: String,
) -> Result<ItemType, String> {
    let pool = get_pool(&state)?;

    sqlx::query("UPDATE item_types SET name = ?, icon = ? WHERE id = ?")
        .bind(&name)
        .bind(&icon)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // Return updated type with its fields
    let field_rows: Vec<(i64, i64, String, String, String, i32, String)> = sqlx::query_as(
        "SELECT f.id, f.type_id, f.name, f.field_type, f.icon, f.position, f.label FROM fields f WHERE f.type_id = ? ORDER BY f.position",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let fields = field_rows
        .into_iter()
        .map(|(fid, tid, fname, ftype, ficon, pos, label)| Field {
            id: fid,
            type_id: tid,
            name: fname,
            field_type: ftype,
            icon: ficon,
            position: pos,
            label,
        })
        .collect();

    Ok(ItemType { id, name, icon, fields })
}
```

- [ ] **Step 4: 添加 update_field 命令**

在 `update_item_type` 后继续追加：

```rust
#[tauri::command]
pub async fn update_field(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    field_type: String,
    icon: String,
    label: String,
) -> Result<Field, String> {
    let pool = get_pool(&state)?;

    sqlx::query("UPDATE fields SET name = ?, field_type = ?, icon = ?, label = ? WHERE id = ?")
        .bind(&name)
        .bind(&field_type)
        .bind(&icon)
        .bind(&label)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // Return updated field
    let row: (i64, i64, String, String, String, i32, String) = sqlx::query_as(
        "SELECT id, type_id, name, field_type, icon, position, label FROM fields WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Field {
        id: row.0,
        type_id: row.1,
        name: row.2,
        field_type: row.3,
        icon: row.4,
        position: row.5,
        label: row.6,
    })
}
```

- [ ] **Step 5: 更新 add_field 命令支持 label 参数**

在 `add_field` 函数签名中添加 `label` 参数，INSERT 语句加入 `label` 列：

```rust
#[tauri::command]
pub async fn add_field(
    state: State<'_, AppState>,
    type_id: i64,
    name: String,
    field_type: String,
    icon: Option<String>,
    label: Option<String>,
) -> Result<Field, String> {
    let pool = get_pool(&state)?;
    let icon = icon.unwrap_or_else(|| "circle".to_string());
    let label = label.unwrap_or_default();

    let max_pos: Option<i32> =
        sqlx::query_scalar("SELECT MAX(position) FROM fields WHERE type_id = ?")
            .bind(type_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
    let position = max_pos.unwrap_or(-1) + 1;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fields (type_id, name, field_type, icon, position, label) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(type_id)
    .bind(&name)
    .bind(&field_type)
    .bind(&icon)
    .bind(position)
    .bind(&label)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Field {
        id,
        type_id,
        name,
        field_type,
        icon,
        position,
        label,
    })
}
```

- [ ] **Step 6: 注册新命令**

Edit `src-tauri/src/lib.rs`，在 `invoke_handler` 宏中添加两个新命令：

```rust
commands::types::update_item_type,
commands::types::update_field,
```

添加到 `commands::types::reorder_fields,` 之后。

- [ ] **Step 7: 编译验证后端**

Run: `cd src-tauri && cargo build 2>&1`

Expected: 编译成功，无错误。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/commands/types.rs src-tauri/src/lib.rs
git commit -m "feat: add update_item_type and update_field commands, Field.label column"
```

---

### Task 3: 前端类型 + Store 更新

**Files:**
- Modify: `src/types/bindings.ts`
- Modify: `src/stores/types.ts`

**Interfaces:**
- Consumes: Task 2 后端命令
- Produces:
  - `Field` interface 新增 `label: string`
  - `typeStore.update(id, name, icon): Promise<ItemType>`
  - `typeStore.updateField(id, name, fieldType, icon, label): Promise<Field>`

- [ ] **Step 1: 更新 Field 类型定义**

Edit `src/types/bindings.ts`，在 `Field` interface 中添加 `label`：

```ts
export interface Field {
  id: number
  type_id: number
  name: string
  field_type: 'text' | 'checkbox'
  icon: string
  position: number
  label: string
}
```

- [ ] **Step 2: 添加 Store update 方法**

Edit `src/stores/types.ts`，在 `reorderFields` 方法后添加两个新方法。

`update` 方法（在 `remove` 方法后添加）：

```ts
async function update(id: number, name: string, icon: string): Promise<ItemType> {
  const t = await invoke<ItemType>('update_item_type', { id, name, icon })
  const idx = types.value.findIndex(t => t.id === id)
  if (idx !== -1) types.value[idx] = t
  return t
}
```

`updateField` 方法（在 `removeField` 方法后添加）：

```ts
async function updateField(id: number, name: string, fieldType: string, icon: string, label: string): Promise<Field> {
  const f = await invoke<Field>('update_field', { id, name, fieldType, icon, label })
  for (const t of types.value) {
    const idx = t.fields.findIndex(fi => fi.id === id)
    if (idx !== -1) {
      t.fields[idx] = f
      break
    }
  }
  return f
}
```

- [ ] **Step 3: 更新 addField 方法签名（支持 label 参数）**

Edit `src/stores/types.ts`，更新现有 `addField` 函数，添加 `label` 可选参数：

将：
```ts
async function addField(typeId: number, name: string, fieldType: string, icon?: string): Promise<Field> {
  const f = await invoke<Field>('add_field', { typeId, name, fieldType, icon: icon ?? null })
```
改为：
```ts
async function addField(typeId: number, name: string, fieldType: string, icon?: string, label?: string): Promise<Field> {
  const f = await invoke<Field>('add_field', { typeId, name, fieldType, icon: icon ?? null, label: label ?? null })
```

- [ ] **Step 4: 更新 return 导出**

在 `src/stores/types.ts` 的 return 对象中，添加新方法：

```ts
return { types, loading, getTypeById, fetchAll, create, remove, addField, removeField, reorderFields, update, updateField }
```

- [ ] **Step 4: 类型检查**

Run: `cd src-tauri && cargo build 2>&1` （确保 Rust 端无误）
Run: `npx vue-tsc --noEmit 2>&1` （检查 TypeScript 类型）

Expected: 编译通过。

- [ ] **Step 5: Commit**

```bash
git add src/types/bindings.ts src/stores/types.ts
git commit -m "feat: add Field.label type and store update/updateField methods"
```

---

### Task 4: ActivityBar 组件

**Files:**
- Create: `src/components/ActivityBar.vue`
- Modify: `src/components/TablerIcon.vue`（添加新图标）

**Interfaces:**
- Produces: `<ActivityBar>` 组件
  - Props: `tabs: { id: string, icon: string, title: string }[]`, `active: string`
  - Emits: `select(tabId: string)`

- [ ] **Step 1: 添加新图标到 TablerIcon 注册表**

Edit `src/components/TablerIcon.vue`，在 import 中添加 `IconCategory, IconFileDescription`：

```ts
import {
  IconDatabase, IconPlus, IconMoon, IconSun, IconSettings,
  IconFolder, IconFolderOpen, IconTag, IconHash, IconChevronRight,
  IconClipboard, IconTrash, IconFile, IconFileText, IconPhoto,
  IconBook, IconMusic, IconVideo, IconFileZip,
  IconPaperclip, IconArrowLeft, IconCheckbox, IconCircle, IconCheck, IconX,
  IconGripVertical, IconCategory, IconFileDescription, IconDots, IconPencil,
} from '@tabler/icons-vue'
```

在 map 中添加：
```ts
'category': IconCategory,
'file-description': IconFileDescription,
'dots': IconDots,
```

- [ ] **Step 2: 创建 ActivityBar.vue**

```vue
<template>
  <nav class="act-bar">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-btn"
      :class="{ active: active === tab.id }"
      :title="tab.title"
      @click="$emit('select', tab.id)"
    >
      <TablerIcon :name="tab.icon" :size="20" />
    </button>
  </nav>
</template>

<script setup lang="ts">
import TablerIcon from './TablerIcon.vue'

defineProps<{
  tabs: { id: string; icon: string; title: string }[]
  active: string
}>()

defineEmits<{
  select: [tabId: string]
}>()
</script>

<style scoped>
.act-bar {
  width: 48px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
  border-left: 1px solid var(--border);
  background: var(--surface);
}

.tab-btn {
  width: 36px;
  height: 36px;
  padding: 0;
  border: none;
  border-radius: var(--r-md);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--fast) var(--ease);
}

.tab-btn:hover {
  color: var(--text-secondary);
  background: var(--surface-hover);
}

.tab-btn.active {
  color: var(--accent);
  background: var(--accent-subtle);
}
</style>
```

Write to `src/components/ActivityBar.vue`.

- [ ] **Step 3: 验证文件**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误。

- [ ] **Step 4: Commit**

```bash
git add src/components/ActivityBar.vue src/components/TablerIcon.vue
git commit -m "feat: add ActivityBar component with vertical icon tabs"
```

---

### Task 5: TypeManager 组件（类别管理面板）

**Files:**
- Create: `src/components/TypeManager.vue`

**Interfaces:**
- Consumes: `useTypeStore`（Task 3 新增的 update/updateField）, `useToast`
- Produces: `<TypeManager>` 自包含组件，内部切换两级视图

- [ ] **Step 1: 创建组件骨架和脚本**

```vue
<template>
  <div class="tm">
    <!-- 视图内容在后续步骤填充 -->
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

// ── Computed ──
const currentType = computed(() =>
  typeStore.types.find(t => t.id === editingTypeId.value) ?? null
)

// ── Methods (后续步骤填充) ──
</script>
```

- [ ] **Step 2: 实现 TypeListView 模板和方法**

完整的 TypeListView 部分：

```vue
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

    <!-- Delete popover (rendered in both views) -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="popover-overlay" @click.self="deleteTarget = null">
        <div class="popover" :style="popoverStyle">
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
```

- [ ] **Step 3: 实现 TypeListView 脚本方法**

在 `<script setup>` 中添加方法：

```ts
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

// ── Popover position helper ──
const popoverStyle = computed(() => ({
  // Center of viewport (simple approach)
}))
```

- [ ] **Step 4: 实现 TypeFieldView 模板**

在 TypeListView 的 `</template>` 之后添加：

```vue
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
            <div class="field-main">
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
```

- [ ] **Step 5: 实现 TypeFieldView 脚本方法**

在 `<script setup>` 中添加：

```ts
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
const dragId = ref<number | null>(null)
const dragOverId = ref<number | null>(null)

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
  if (!currentType.value || dragId.value === null) return
  const fields = [...currentType.value.fields]
  const fromIdx = fields.findIndex(f => f.id === dragId.value)
  const toIdx = fields.findIndex(f => f.id === dragOverId.value)
  if (fromIdx === -1 || toIdx === -1 || fromIdx === toIdx) return

  // Reorder locally
  const [moved] = fields.splice(fromIdx, 1)
  fields.splice(toIdx, 0, moved)
  // Reflect in store immediately
  currentType.value.fields = fields

  try {
    await typeStore.reorderFields(
      currentType.value.id,
      fields.map(f => f.id)
    )
  } catch (e) {
    toast.error('排序失败: ' + e)
  }
  dragId.value = null
  dragOverId.value = null
}

function onDragEnd() {
  dragId.value = null
  dragOverId.value = null
}
```

- [ ] **Step 6: 添加样式**

在 `<script setup>` 后添加 `<style scoped>` 块：

```css
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
```

Write the complete file to `src/components/TypeManager.vue`.

- [ ] **Step 7: 类型检查**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误。

- [ ] **Step 8: Commit**

```bash
git add src/components/TypeManager.vue
git commit -m "feat: add TypeManager component with inline edit, drag reorder, and popover delete"
```

---

### Task 6: RightPanel 改造 + TablerIcon 补充

**Files:**
- Modify: `src/components/RightPanel.vue`
- Modify: `src/components/TablerIcon.vue`（补充 pencil 图标）

**Interfaces:**
- Consumes: ActivityBar, TypeManager（Task 4, 5）
- Props: `activeTab: 'detail' | 'types'`
- Emits: `update:activeTab(tab: string)`

- [ ] **Step 1: 确认 TablerIcon 已注册所需图标**

Task 4 已添加 `IconCategory, IconFileDescription, IconDots, IconPencil`。确认 `src/components/TablerIcon.vue` 中这些图标的 import 和 map 都已正确配置。

- [ ] **Step 2: 改造 RightPanel 为容器组件**

Edit `src/components/RightPanel.vue`，将整个文件替换为：

```vue
<template>
  <div class="rp-container">
    <div class="rp-content">
      <!-- Tab: Detail -->
      <aside v-if="activeTab === 'detail'" class="rp">
        <div v-if="!detail" class="empty">
          <TablerIcon name="arrow-left" :size="28" :stroke="1" />
          <p>选择条目查看详情</p>
        </div>
        <template v-else>
          <div class="hd">
            <div class="title"><TablerIcon :name="detail.item_type.icon" :size="20" /> {{ detail.item.name }}</div>
            <div class="id font-mono">{{ detail.item.id }}</div>
          </div>

          <div class="sec"><div class="lbl">属性</div><PropertiesForm :detail="detail" /></div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">分组</div>
            <div class="chips">
              <span v-for="g in detail.groups" :key="g.id" class="chip" @click="removeGroup(g.id)">
                <TablerIcon name="folder" :size="13" />{{ g.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.groups.length === 0 && !addingGroup" class="text-muted text-xs">未分组</span>
            </div>
            <div v-if="!addingGroup" class="add-btn" @click="addingGroup = true">+ 添加分组</div>
            <div v-else class="add-row">
              <select v-model="newGroupId">
                <option :value="null" disabled>选择分组...</option>
                <option v-for="g in availableGroups" :key="g.id" :value="g.id">{{ g.name }}</option>
              </select>
              <button class="primary sm" @click="addGroup" :disabled="!newGroupId">确定</button>
              <button class="ghost sm" @click="addingGroup = false">取消</button>
            </div>
          </div>
          <div class="sep" />

          <div class="sec">
            <div class="lbl">标签</div>
            <div class="chips">
              <span v-for="t in detail.tags" :key="t.id" class="chip tag" @click="removeTag(t.id)">
                <TablerIcon name="hash" :size="13" />{{ t.name }} <TablerIcon name="x" :size="11" />
              </span>
              <span v-if="detail.tags.length === 0 && !addingTag" class="text-muted text-xs">无标签</span>
            </div>
            <div v-if="!addingTag" class="add-btn" @click="addingTag = true">+ 添加标签</div>
            <div v-else class="add-row">
              <select v-model="newTagId">
                <option :value="null" disabled>选择标签...</option>
                <option v-for="t in availableTags" :key="t.id" :value="t.id">{{ t.name }}</option>
              </select>
              <button class="primary sm" @click="addTag" :disabled="!newTagId">确定</button>
              <button class="ghost sm" @click="addingTag = false">取消</button>
            </div>
          </div>
          <div class="sep" />

          <div class="sec file-sec">
            <FileTree :item-id="detail.item.id" />
          </div>
        </template>
      </aside>

      <!-- Tab: Type Manager -->
      <TypeManager v-else-if="activeTab === 'types'" />
    </div>

    <ActivityBar
      :tabs="tabs"
      :active="activeTab"
      @select="emit('update:activeTab', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useItemStore } from '@/stores/items'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import PropertiesForm from './PropertiesForm.vue'
import FileTree from './FileTree.vue'
import TablerIcon from './TablerIcon.vue'
import ActivityBar from './ActivityBar.vue'
import TypeManager from './TypeManager.vue'

const props = defineProps<{
  activeTab: 'detail' | 'types'
}>()

const emit = defineEmits<{
  'update:activeTab': [tab: string]
}>()

const tabs = [
  { id: 'detail', icon: 'file-description', title: '条目详情' },
  { id: 'types', icon: 'category', title: '类别管理' },
]

const itemStore = useItemStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()

const detail = computed(() => itemStore.detail)
const addingGroup = ref(false)
const addingTag = ref(false)
const newGroupId = ref<number | null>(null)
const newTagId = ref<number | null>(null)

const availableGroups = computed(() => {
  if (!detail.value) return []
  const ids = new Set(detail.value.groups.map(g => g.id))
  return flattenGroups(groupStore.tree).filter(g => !ids.has(g.id))
})

const availableTags = computed(() => {
  if (!detail.value) return []
  const ids = new Set(detail.value.tags.map(t => t.id))
  return tagStore.tags.filter(t => !ids.has(t.id))
})

function flattenGroups(groups: any[]): any[] {
  return groups.flatMap(g => [g, ...flattenGroups(g.children || [])])
}

async function addGroup() {
  if (!newGroupId.value || !detail.value) return
  await groupStore.addItemToGroup(detail.value.item.id, newGroupId.value)
  await itemStore.select(detail.value.item.id)
  addingGroup.value = false; newGroupId.value = null
}

async function removeGroup(groupId: number) {
  if (!detail.value) return
  await groupStore.removeItemFromGroup(detail.value.item.id, groupId)
  await itemStore.select(detail.value.item.id)
}

async function addTag() {
  if (!newTagId.value || !detail.value) return
  await tagStore.addToItem(detail.value.item.id, newTagId.value)
  await itemStore.select(detail.value.item.id)
  addingTag.value = false; newTagId.value = null
}

async function removeTag(tagId: number) {
  if (!detail.value) return
  await tagStore.removeFromItem(detail.value.item.id, tagId)
  await itemStore.select(detail.value.item.id)
}
</script>

<style scoped>
.rp-container {
  display: flex;
  flex-shrink: 0;
  height: 100%;
}

.rp-content {
  width: var(--right-w);
  flex-shrink: 0;
  overflow: hidden;
}

.rp {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  border-left: 1px solid var(--border);
  background: var(--surface);
}

.empty {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; height: 100%; gap: 8px; color: var(--text-muted);
}

.hd { padding: 16px 16px 12px; }
.title {
  display: flex; align-items: center; gap: 8px;
  font-size: var(--fs-lg); font-weight: var(--fw-semibold); margin-bottom: 4px;
}
.id {
  font-size: var(--fs-xs); color: var(--text-secondary);
  background: var(--surface-hover); padding: 1px 6px;
  border-radius: var(--r-sm); display: inline-block;
}

.sec { padding: 8px 16px; }
.lbl {
  font-size: var(--fs-xs); font-weight: var(--fw-semibold);
  color: var(--text-muted); text-transform: uppercase;
  letter-spacing: 0.05em; margin-bottom: 8px;
}
.sep { height: 1px; background: var(--border); margin: 0 16px; }
.file-sec { flex: 1; min-height: 100px; }

.chips { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 4px; }
.chip {
  display: inline-flex; align-items: center; gap: 4px; font-size: var(--fs-xs);
  padding: 2px 8px 2px 10px; border-radius: var(--r-full); cursor: pointer;
  background: var(--bg); color: var(--text-secondary);
  transition: all var(--fast) var(--ease);
}
.chip:hover { background: var(--danger-subtle); color: var(--danger); }
.chip.tag { color: var(--accent); background: var(--accent-subtle); }
.chip.tag:hover { background: var(--danger-subtle); color: var(--danger); }

.add-btn {
  font-size: var(--fs-xs); color: var(--text-muted);
  cursor: pointer; padding: 2px 0;
}
.add-btn:hover { color: var(--accent); }

.add-row { display: flex; gap: 4px; margin-top: 4px; align-items: center; }
.add-row select { font-size: var(--fs-xs); height: 28px; flex: 1; }
.sm { font-size: var(--fs-xs); height: 26px; }
</style>
```

- [ ] **Step 3: 类型检查**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误。

- [ ] **Step 4: Commit**

```bash
git add src/components/RightPanel.vue src/components/TablerIcon.vue
git commit -m "refactor: RightPanel as container with ActivityBar, add TypeManager tab"
```

---

### Task 7: PropertiesForm 修复 — checkbox 文字

**Files:**
- Modify: `src/components/PropertiesForm.vue`

**Interfaces:**
- Consumes: Field.label（Task 3 新增）

- [ ] **Step 1: 修复 checkbox 硬编码文字**

Edit `src/components/PropertiesForm.vue`，将第 10 行：

```vue
<span class="check-label">{{ getValue(field.name) ? '已完成' : '未完成' }}</span>
```

替换为：

```vue
<span class="check-label">{{ field.label || field.name }}</span>
```

即 checkbox 的显示文字始终使用 `field.label`，如果为空则 fallback 到字段名称。不管 checkbox 是选中还是未选中状态，显示的文字都一样（描述性的标签）。

- [ ] **Step 2: 验证修复**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误。

- [ ] **Step 3: Commit**

```bash
git add src/components/PropertiesForm.vue
git commit -m "fix: use field.label for checkbox display text instead of hardcoded string"
```

---

### Task 8: Topbar + App.vue 接入

**Files:**
- Modify: `src/components/Topbar.vue`
- Modify: `src/App.vue`

**Interfaces:**
- App.vue: `activeRightTab` state 传递给 RightPanel
- Topbar: `@open-type-manager` 事件替代 `@settings`

- [ ] **Step 1: 更新 Topbar 按钮**

Edit `src/components/Topbar.vue`，修改模板中设置按钮：

将：
```html
<button class="icon-btn" @click="$emit('settings')" title="设置">
  <TablerIcon name="settings" :size="18" />
</button>
```

替换为：
```html
<button class="icon-btn" @click="$emit('openTypeManager')" title="类别管理">
  <TablerIcon name="category" :size="18" />
</button>
```

更新 emits 定义：
```ts
defineEmits<{ newItem: []; openTypeManager: [] }>()
```

- [ ] **Step 2: 更新 App.vue 状态和模板**

Edit `src/App.vue`：

在 `<script setup>` 中添加：
```ts
const activeRightTab = ref<'detail' | 'types'>('detail')
```

修改模板中的 Topbar 和 RightPanel：
```html
<Topbar @new-item="showNewItem = true" @open-type-manager="activeRightTab = 'types'" />
```

```html
<RightPanel :active-tab="activeRightTab" @update:active-tab="activeRightTab = $event" />
```

移除 `showSettings` 相关代码：
- 删除 `const showSettings = ref(false)`
- 删除 `<SettingsPanel v-if="showSettings" @close="showSettings = false" />`
- 删除 `import SettingsPanel from '@/components/SettingsPanel.vue'`

- [ ] **Step 3: 检查所有引用**

Run: `grep -r "SettingsPanel" src/ --include="*.vue" --include="*.ts"`

Expected: 无匹配结果（除已删除的文件本身）。

- [ ] **Step 4: 类型检查**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误。

- [ ] **Step 5: Commit**

```bash
git add src/components/Topbar.vue src/App.vue
git commit -m "refactor: wire ActivityBar tab state, replace settings with type manager entry"
```

---

### Task 9: 清理 — 删除 SettingsPanel.vue

**Files:**
- Delete: `src/components/SettingsPanel.vue`

- [ ] **Step 1: 删除文件**

```bash
git rm src/components/SettingsPanel.vue
```

- [ ] **Step 2: 验证构建**

Run: `npx vue-tsc --noEmit 2>&1`

Expected: 无类型错误，无未解析的导入。

Run: `cd src-tauri && cargo build 2>&1`

Expected: Rust 编译成功。

- [ ] **Step 3: Commit**

```bash
git commit -m "refactor: remove old SettingsPanel, replaced by TypeManager + ActivityBar"
```

---

## Implementation Order

```
Task 1 (DB) ──→ Task 2 (Backend) ──→ Task 3 (Types+Store)
                                           │
              Task 4 (ActivityBar) ←───────┘
                   │
              Task 5 (TypeManager) ←───────┘
                   │
              Task 6 (RightPanel) ←──────── Task 4, 5
                   │
              Task 7 (PropertiesForm fix) ← Task 3
                   │
              Task 8 (Topbar + App.vue) ←── Task 6
                   │
              Task 9 (Cleanup) ←─────────── Task 8
```

Tasks 1-3 可以并行开发（DB → Backend → Types），Tasks 4-5 依赖 Task 3，Task 6 依赖 4+5。
