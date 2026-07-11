# 条目类别管理 — 界面重设计

| Field | Value |
|---|---|
| **Status** | Draft |
| **Date** | 2026-07-11 |
| **Scope** | 条目类别管理面板重设计 + 编辑/排序/删除确认完备 UX |
| **Dependencies** | 现有 ItemType/Field 数据模型、Pinia typeStore、SQLite backend |

---

## 1. 设计目标

将当前简陋的 `SettingsPanel.vue`（"设置"弹窗）重新设计为完备的**条目类别管理**面板，具备：

- 类别和字段的**完整 CRUD**（当前只有 Create + Delete，无 Update）
- **行内编辑**（无需弹窗，页面内完成所有操作）
- **字段拖拽排序**（后端已有 `reorder_fields`，前端补充 UI）
- **非原生删除确认**（用 Popover 替代 `confirm()`）
- Checkbox 字段支持**配置显示文字**（不再硬编码"已完成/未完成"）

同时将入口从 Topbar 齿轮图标改为专用的条目类别管理入口，为后续扩展（AI、插件等）建立 ActivityBar 架构。

---

## 2. 架构变更

### 2.1 右侧面板改造：ActivityBar

当前 RightPanel 直接渲染条目详情。改造后，右侧面板区域分为两层：

```
┌──────────────────────┬──────┐
│                      │      │
│   面板内容区          │  A   │
│   (360px)            │  c   │
│                      │  t   │
│                      │  i   │
│                      │  v   │
│                      │  i   │
│                      │  t   │
│                      │  y   │
│                      │  B   │
│                      │  a   │
│                      │  r   │
│                      │      │
│                      │ 48px │
└──────────────────────┴──────┘
```

- **ActivityBar**（48px 宽）：竖排图标 Tab 列，固定在面板最右侧
  - Tab 1: 条目详情（`file-description` 图标）
  - Tab 2: 类别管理（`category` 图标）
  - 后续扩展：AI 助手、插件、设置等
- **面板内容区**（360px）：根据 ActivityBar 选中的 Tab 动态渲染对应组件

### 2.2 组件重命名 & 新增

| 旧文件 | 新文件 | 说明 |
|--------|--------|------|
| `SettingsPanel.vue` | 删除 | 不再使用 |
| `RightPanel.vue` | 改造 | 作为面板容器，管理 Tab 切换 |
| — | `ActivityBar.vue` (新增) | 最右侧竖排图标 Tab |
| — | `TypeManager.vue` (新增) | 类别管理面板 |
| — | `TypeListView` (内嵌) | 第一级：类别列表 |
| — | `TypeFieldView` (内嵌) | 第二级：字段管理 |

### 2.3 Topbar 变更

- 移除齿轮图标 `settings` 按钮
- 新增类别管理按钮（图标 `category`），点击后 ActivityBar 切换到 Tab 2 并打开 TypeManager

---

## 3. TypeManager 交互设计

### 3.1 第一级：类别列表视图

```
┌──────────────────────────────────┐
│ 类别管理                         │
│                                  │
│ ┌──────────────────────────────┐ │
│ │ 📄  通用              预设   │ │  ← 预设，不可删除
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ ✅  任务              预设   │ │
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ 📚  书籍                    │ │  ← 点击行 → 进入字段视图
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ 🎵  音乐                    │ │
│ └──────────────────────────────┘ │
│                                  │
│ [+ 新建类别]                     │  ← 点击展开内联创建行
└──────────────────────────────────┘
```

**交互规则：**

1. **点击行** → 进入字段管理视图（第二级）
2. **双击行 / 点击编辑按钮** → 行内编辑模式：名称变为输入框、图标变为输入框，回车确认、Esc 取消
3. **新建类别**：点击 `[+]` → 展开内联创建行（图标输入 + 名称输入），回车确认
4. **删除**：行右侧 `···` 菜单按钮 → Popover 确认。预设类型（id 1, 2）不显示删除选项
5. **空状态**：仅 2 个预设类型时显示引导文字 "通过新建类别来创建自定义条目模板"
6. **预设标签**：灰色小标签 `预设`，区别于自定义类型
7. **Toast 反馈**：所有操作结果通过 Toast 通知

### 3.2 第二级：字段管理视图

```
┌──────────────────────────────────┐
│ ← 书籍                           │  ← 面包屑，点击返回列表
│                                  │
│ 类别信息                         │
│ ┌──────────────────────────────┐ │
│ │ 📚  [书籍              ]  ✗  │ │  ← 行内编辑
│ └──────────────────────────────┘ │
│                                  │
│ 字段                    3 项     │
│ ┌──────────────────────────────┐ │
│ │ ≡ 📝 作者            [text] │ │  ← ≡ 拖拽手柄
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ ≡ 📅 出版日期        [text] │ │
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ ≡ ☑ 已读       [checkbox]  │ │
│ │   显示文字: "已全部读完"      │ │  ← checkbox 专属
│ └──────────────────────────────┘ │
│                                  │
│ [+ 添加字段]                     │
└──────────────────────────────────┘
```

**交互规则：**

1. **类别编辑区**（顶部固定）：点击名称或图标进入行内编辑
2. **字段行常态**：显示图标、名称、类型标签
3. **字段行编辑态**（点击展开）：

```
常态:
  ≡ 📝 作者                    text    [···]

编辑态:
  ≡ [📝 ] [作者    ] [文本 ▾] [显示文字...]
                              [✓] [✗]
```

   - 图标、名称、类型（text/checkbox 下拉）均可编辑
   - checkbox 类型多一个"显示文字"输入框
   - ✓ 确认保存，✗ 取消恢复原值

4. **字段拖拽排序**：拖拽 `≡` 手柄调整顺序，松开即保存（调用 `reorder_fields`）
5. **字段删除**：`···` → Popover 确认
6. **空状态**：无自定义字段时显示 "暂无字段，点击下方添加"
7. **添加字段**：底部展开输入行，先选类型再输入名称

### 3.3 删除确认 Popover

替代 `window.confirm()`，设计为附着在 `···` 按钮旁的小弹出框：

```
┌──────────────────────┐
│ ⚠ 删除"音乐"类别     │
│ 已有 12 个条目使用    │  ← 显示影响范围（如可统计）
│                      │
│ [取消]  [确认删除]    │
└──────────────────────┘
```

---

## 4. PropertiesForm 修复

当前 `PropertiesForm.vue` 第 10 行硬编码了 checkbox 的显示文字：

```ts
// 当前（错误）
<span>{{ getValue(field.name) ? '已完成' : '未完成' }}</span>
```

修复为使用 Field 的 `label` 属性：

```ts
// 修复后
<span>{{ field.label || field.name }}</span>
```

label 为空时 fallback 到字段名称。

---

## 5. 数据模型 & 后端变更

### 5.1 数据库迁移

```sql
-- migration: 20260711_add_field_label.sql
ALTER TABLE fields ADD COLUMN label TEXT NOT NULL DEFAULT '';
```

### 5.2 Field 模型（Rust + TypeScript 同步）

```rust
pub struct Field {
    pub id: i64,
    pub type_id: i64,
    pub name: String,
    pub field_type: String,
    pub icon: String,
    pub position: i32,
    pub label: String,        // 新增：checkbox 显示文字
}
```

### 5.3 新增后端命令

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `update_item_type` | `id: i64, name: String, icon: String` | `ItemType` | 更新类别名称/图标 |
| `update_field` | `id: i64, name: String, field_type: String, icon: String, label: String` | `Field` | 更新字段属性 |

### 5.4 Store 新增方法

```ts
// typeStore
update(id: number, name: string, icon: string): Promise<ItemType>
updateField(id: number, name: string, fieldType: string, icon: string, label: string): Promise<Field>
```

---

## 6. 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/components/SettingsPanel.vue` | 删除 | 被 TypeManager 替代 |
| `src/components/ActivityBar.vue` | 新增 | 竖排图标 Tab 栏 |
| `src/components/TypeManager.vue` | 新增 | 类别管理面板（含两级视图） |
| `src/components/RightPanel.vue` | 修改 | 作为面板容器，集成 ActivityBar |
| `src/components/PropertiesForm.vue` | 修改 | 修复 checkbox 硬编码文字 |
| `src/components/Topbar.vue` | 修改 | 入口改为类别管理按钮 |
| `src/App.vue` | 修改 | 适配新面板架构 |
| `src/stores/types.ts` | 修改 | 新增 update/updateField 方法 |
| `src/types/bindings.ts` | 修改 | Field 加 label 字段，新增命令类型 |
| `src-tauri/src/models.rs` | 修改 | Field 加 label 字段 |
| `src-tauri/src/commands/types.rs` | 修改 | 新增 update_item_type, update_field |
| `src-tauri/src/lib.rs` | 修改 | 注册新命令 |
| `src-tauri/migrations/20260711_add_field_label.sql` | 新增 | 数据库迁移 |

---

## 7. ActivityBar 扩展性

ActivityBar 设计为通用 Tab 注册机制，方便后续扩展：

```ts
// 未来可扩展的结构
interface ActivityTab {
  id: string
  icon: string
  title: string
  component: Component
  badge?: number        // 未读数等
}
```

当前版本只用 2 个 Tab（条目详情 + 类别管理），后续 AI 助手、工作台插件、设置等均注册到这里。
