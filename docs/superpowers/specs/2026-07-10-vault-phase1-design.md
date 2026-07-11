# Index Phase 1 Design Spec

| Field | Value |
|---|---|
| **Status** | Draft |
| **Date** | 2026-07-10 |
| **Project code name** | Index (placeholder) |
| **Scope** | Phase 1 — local-first object/project manager |
| **Platform** | Windows only |
| **Quality bar** | Open-source product (polished UX, complete docs) |

---

## 1. Product Positioning

A **local-first object/project manager** built on Tauri 2 + Vue 3 + Rust + SQLite.

**What it IS:** Manage item structures, custom typed properties, multi-level groups, flat tags, and attachment file trees. A file-system-aware organizational tool — like a supercharged folder manager with typed metadata.

**What it is NOT:** A Markdown editor. A wiki. A Notion/Obsidian/Zotero clone. The app does **not** edit file contents — double-clicking any file opens it with the system default program. Editing happens outside the app.

### Core Principles

- **Local-first**: all data in a user-chosen folder, Git/syncthing-friendly
- **Object-first**: every item is a real folder on disk, not a database blob
- **No editing**: the app manages structure + properties; file contents are external
- **Open-source quality bar**: polished UX, complete docs, stable API from Phase 1

---

## 2. Phase 1 Scope

### 2.1 In Scope

- Tauri 2 + Vue 3 + Rust + SQLite project scaffolding
- Repository open / close / create
- Three-pane layout: sidebar (groups + tags) | center list | right detail (properties + file tree)
- **Custom item types**: user creates types, adds fields (text + checkbox)
- **Item CRUD**: create with 12-char random hex ID, auto-generate `<name>.md`, auto-create hash folder
- **Group management**: multi-level tree, inline CRUD in sidebar, drag-reorder
- **Tag management**: flat list, inline CRUD in sidebar
- **Attachment file tree**: recursive tree view, right-click delete/rename, drag-in from OS, new folder
- File double-click → system default program (`shell::open`)
- Theme switch: light / dark, persisted
- Empty states, error toasts, status bar
- Test suites: Rust unit + integration, Vue unit (Vitest), E2E (Playwright + Tauri)
- Documentation: README, architecture doc, data-model doc, user guide, developer guide, CONTRIBUTING, CHANGELOG, LICENSE

### 2.2 Explicitly Deferred

| Feature | Phase | Design Reservation |
|---|---|---|
| Item links (standard Markdown `[text](../hash/)`) | Phase 2 | `item_links` table in design.md, `properties` ready |
| Search / filter | Phase 2 | FTS5 index deferred |
| Plugin system | Phase 3 | `namespace` column, Web Components architecture |
| AI integration | Phase 4 | Rust proxy for LLM APIs |
| Workspace snapshots | Phase 4 | `namespace` column |
| Custom views (calendar / gallery / kanban) | Phase 5 | center pane slot architecture |
| Sync | Not planned | Folder layout naturally Git/syncthing-friendly |
| macOS / Linux | Not planned | No Windows-only APIs |

---

## 3. Tech Stack

| Layer | Choice | Reason |
|---|---|---|
| Desktop shell | **Tauri 2** | 5–10 MB installer, <500 ms startup, Rust native |
| Frontend | **Vue 3 + TypeScript** | Three-pane layout natural in Vue; Phase 2 plugins → Web Components |
| State management | **Pinia** | Vue 3 official recommendation |
| UI components | **Self-built** | Simple components (tree, list, form, context menu); avoid heavy deps |
| Backend | **Rust** | Tauri native, zero-cost IPC |
| Database | **SQLite via sqlx** | Single file, `sqlx::migrate!` versioned schema |
| Type sharing | **specta + specta-typescript** | Rust struct → TS types auto-generated |
| ID generation | **Pure random 12 hex chars** | `sha256(rand_16_bytes)` truncated to 12 chars; ~2.8×10¹⁴ space |
| File tree | **Self-built Vue 3 recursive component** | ~200 lines; no suitable embeddable web file manager exists |
| Workspace | **pnpm workspace + Cargo workspace** | Dual-language monorepo |
| Build | **Vite (frontend) + cargo tauri build** | Standard Tauri 2 pipeline |
| Installer | **NSIS** | Windows-native |

### Explicitly NOT Used

- ❌ No Element Plus / Naive UI — too heavy for our simple component set
- ❌ No markdown-it — not editing or rendering Markdown in-app
- ❌ No Monaco / CodeMirror — not editing text in-app
- ❌ No ORM — sqlx raw SQL is sufficient

---

## 4. Data Model

### 4.1 SQL Schema

```sql
-- Item types (user-defined)
CREATE TABLE item_types (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    icon TEXT NOT NULL DEFAULT '📄'
);

-- Type fields
CREATE TABLE fields (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    type_id    INTEGER NOT NULL REFERENCES item_types(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    field_type TEXT NOT NULL DEFAULT 'text',    -- 'text' | 'checkbox'
    position   INTEGER NOT NULL DEFAULT 0,
    UNIQUE(type_id, name)
);

-- Items
CREATE TABLE items (
    id         TEXT PRIMARY KEY,                -- 12 hex pure random
    name       TEXT NOT NULL,
    type_id    INTEGER NOT NULL REFERENCES item_types(id),
    properties TEXT NOT NULL DEFAULT '{}',      -- JSON: {"author":"鲁迅","read":true}
    namespace  TEXT NOT NULL DEFAULT 'default',
    created_at TEXT NOT NULL,                   -- ISO 8601
    updated_at TEXT NOT NULL
);

-- Groups (multi-level tree)
CREATE TABLE groups (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER REFERENCES groups(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    position  INTEGER NOT NULL DEFAULT 0,
    namespace TEXT NOT NULL DEFAULT 'default'
);

-- Tags (flat)
CREATE TABLE tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Junction tables
CREATE TABLE item_groups (
    item_id  TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    group_id INTEGER NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, group_id)
);

CREATE TABLE item_tags (
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);
```

### 4.2 Key Design Decisions

**No `attachments` table**: The file tree reads directly from `std::fs::read_dir`. No metadata storage, no MIME detection, no content hashing. The filesystem IS the source of truth.

**Properties as JSON blob**: Field definitions live in `fields` (controls UI rendering). Actual values live in `items.properties` (controls data). Deleting a field does not delete existing data — it just stops rendering that key.

**Preset types on first repo creation**:
- "通用" 📄: field `备注` (text)
- "任务" ✅: fields `描述` (text), `完成` (checkbox)

### 4.3 Rust Structs

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemType {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Field {
    pub id: i64,
    pub type_id: i64,
    pub name: String,
    pub field_type: String,     // "text" | "checkbox"
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Item {
    pub id: String,             // 12 hex pure random
    pub name: String,
    pub type_id: i64,
    pub properties: serde_json::Value,
    pub namespace: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemDetail {
    pub item: Item,
    pub item_type: ItemType,
    pub groups: Vec<Group>,
    pub tags: Vec<Tag>,
    pub files: FileNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Group {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub position: i32,
    pub children: Vec<Group>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}
```

---

## 5. Repository Layout

```
~/Documents/MyRepo/                    ← user-selected root
├── .index/                            ← app metadata (hidden)
│   ├── index.db
│   └── state.json                    ← theme preference
├── a3f2c1b8e9d4/                     ← item "鲁迅"
│   ├── 鲁迅.md                        ← auto-created on item creation
│   ├── cover.jpg                     ← user-dropped attachment
│   └── notes/                        ← user-created subfolder
│       └── draft.txt
├── b7d8e9f01234/                     ← item "呐喊"
│   └── 呐喊.md
└── c1d2e3f45678/                     ← item "朱自清"
    ├── 朱自清.md
    ├── 背影.txt
    └── 荷塘月色.txt
```

**Rules**:
- Item creation → auto-create hash folder + `<name>.md` (content: `# <name>\n`)
- User drags files into app → copy into hash folder
- User creates subfolders in file tree → `std::fs::create_dir`
- Delete item → recursive remove entire hash folder + cascade DB
- User can freely operate in file manager outside the app

---

## 6. UI Design

### 6.1 Three-Pane Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Topbar   仓库名  │  + 新建条目  │  🌓 主题  │  ⚙ 设置       │
├──────────────┬──────────────────────┬────────────────────────┤
│ Sidebar 220px│  Center flex-1       │  Right 360px           │
│              │                      │                        │
│ 📁 Groups    │ ┌──────────────────┐ │ ┌────────────────────┐ │
│  ├ 工作      │ │ 📄 鲁迅          │ │ │ 📄 鲁迅            │ │
│  │ ├ 笔记    │ │    书籍 · 3d ago │ │ │ a3f2c1b8  书籍     │ │
│  │ └ 项目    │ ├──────────────────┤ │ │                    │ │
│  └ 学习      │ │ 📄 呐喊          │ │ │ 📝 属性            │ │
│              │ │    书籍 · 1d ago │ │ │ 作者  [鲁迅      ] │ │
│ 🏷 Tags      │ ├──────────────────┤ │ │ 已读  [✓]          │ │
│  # 中文      │ │ 📄 朱自清        │ │ │ 出版  [1938      ] │ │
│  # 散文      │ │    作家 · 5d ago │ │ │                    │ │
│  + 新建标签  │ └──────────────────┘ │ │ 📁 Groups          │ │
│              │                      │ │  工作 > 笔记       │ │
│              │  右键 → 删除条目     │ │                    │ │
│              │  右键 → 移到分组     │ │ 🏷 Tags            │ │
│              │                      │ │  # 中文  # 散文    │ │
│              │                      │ │                    │ │
│              │                      │ │ 📎 附件 (4)        │ │
│              │                      │ │  ├ 📄 鲁迅.md      │ │
│              │                      │ │  ├ 🖼 cover.jpg    │ │
│              │                      │ │  └ 📁 notes/       │ │
│              │                      │ │     └ draft.txt    │ │
│              │                      │ └────────────────────┘ │
├──────────────┴──────────────────────┴────────────────────────┤
│ Statusbar   仓库路径  │  12 条目  │  ⚪ 已保存               │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 Key Interactions

| Action | Trigger | Result |
|---|---|---|
| Filter items | Click group or tag in sidebar | Center list shows items in that group AND with that tag |
| New item | Topbar `+` → select type → enter name | Create hash folder + `<name>.md` + DB row |
| Edit properties | Type in right panel form fields | Auto-save on blur (debounce 500ms), no save button |
| View file | Double-click file in file tree | `shell::open` → system default program |
| Add attachment | Drag file from Explorer onto file tree area | Copy to hash folder, tree refreshes |
| Delete attachment | Right-click file → Delete | File deleted, tree refreshes |
| Rename attachment | Right-click file → Rename | Inline rename |
| New folder | Right-click in file tree → New Folder | `std::fs::create_dir` |
| Manage groups | Inline in sidebar: type to create, right-click to rename/delete, drag to reorder | |
| Manage tags | Inline in sidebar: type in input box to create, right-click to delete | |
| Manage types | Settings panel (⚙) | Create/delete types, add/remove/reorder fields |
| Delete item | Right-click item in center → Delete (confirm dialog) | Remove folder + cascade DB |
| Switch theme | Topbar 🌓 | Global toggle, persisted to `state.json` |

### 6.3 Theme

- **Light**: bg `#FFFFFF` / text `#111111` / accent `#2563EB`
- **Dark**: bg `#0A0A0A` / text `#E5E5E5` / accent `#60A5FA`
- CSS variables driven, persisted to `.index/state.json`

### 6.4 Empty & Error States

| Scenario | Display |
|---|---|
| No repo open | Full-screen: "Open Repo" / "Create Repo" buttons |
| Empty repo | Center: "Create your first item" with type selector |
| No items match filter | Center: "No items match" with clear-filter link |
| File operation fails | Toast error with details |
| Path traversal detected | Silent block + log |
| DB locked | Toast: "Database busy, retrying..." |
| File name collision (drag-in) | Auto-rename: "cover.jpg" → "cover (2).jpg" |

---

## 7. IPC API (Rust Backend)

```
Repository
  create_repo(path)                   → { item_count }
  open_repo(path)                     → { item_count, db_version }
  close_repo()                        → void
  get_repo_info()                     → { path, item_count, db_version }

Items
  create_item(type_id, name)          → Item
  get_item(id)                        → ItemDetail
  list_items(group_id?, tag_id?)      → Item[]
  update_item(id, name?, properties?) → Item
  delete_item(id)                     → void

Types
  list_item_types()                   → ItemType[]
  create_item_type(name, icon?)       → ItemType
  delete_item_type(id)                → void
  add_field(type_id, name, field_type)→ Field
  remove_field(field_id)              → void
  reorder_fields(type_id, field_ids[])→ void

Groups
  list_groups()                       → Group[]
  create_group(name, parent_id?)      → Group
  update_group(id, name?)             → Group
  delete_group(id)                    → void
  move_group(id, parent_id?, position)→ void
  add_item_to_group(item_id, group_id)→ void
  remove_item_from_group(item_id, group_id) → void

Tags
  list_tags()                         → Tag[]
  create_tag(name)                    → Tag
  delete_tag(id)                      → void
  add_tag_to_item(item_id, tag_id)    → void
  remove_tag_from_item(item_id, tag_id) → void

Files
  list_files(item_id)                 → FileNode
  create_folder(item_id, rel_path)    → void
  delete_file(item_id, rel_path)      → void
  rename_file(item_id, old, new)      → void
  add_attachment(item_id, source_path)→ void
  open_file(item_id, rel_path)        → void     (system default)
```

All file paths validated through `safe_path()` — ensures operations stay inside repo root.

---

## 8. Component Tree (Vue 3)

```
App.vue
├── Topbar.vue              Repo name, +New, theme toggle, settings
├── Sidebar.vue
│   ├── GroupTree.vue       Recursive tree, inline CRUD, drag-reorder
│   └── TagList.vue         Flat list + input, inline CRUD
├── CenterList.vue          Filtered item list, right-click context menu
├── RightPanel.vue
│   ├── PropertiesForm.vue  Dynamic form by type fields
│   └── FileTree.vue        Recursive tree, drag-in, right-click CRUD
├── StatusBar.vue           Repo path | N items | save status
│
├── Dialogs
│   ├── NewItemDialog.vue   Select type → enter name → create
│   ├── SettingsPanel.vue   Manage item types & fields
│   └── ConfirmDialog.vue   Reusable delete confirmation
```

**Self-built components only.** No third-party UI library.
- `FileTree.vue` + `FileTreeNode.vue` + `useFileTree.ts` ≈ 200 lines total
- `GroupTree.vue` + `GroupTreeNode.vue` ≈ 150 lines

---

## 9. Testing Strategy

| Layer | Tool | Scope |
|---|---|---|
| Rust unit | `cargo test` | ID generation, `safe_path()`, JSON round-trip, CRUD via `sqlite::memory:` |
| Rust integration | `cargo test` + tempdir | Create item → folder + DB consistency, file copy, cascade delete |
| Vue unit | Vitest | Pinia stores, dynamic form rendering, FileTree recursion |
| E2E | Playwright + Tauri | Full smoke test (see acceptance criteria) |

### Acceptance Criteria (Phase 1 Done =)

A reviewer on a fresh Windows machine can:

1. Install via NSIS → app launches to empty state
2. Create repo → repo opens with preset types
3. Create custom type "书籍" → add field "作者" (text) + "已读" (checkbox)
4. Create item "鲁迅" → hash folder + `鲁迅.md` exist on disk
5. Edit properties → auto-save → close & reopen → data persists
6. Drag JPG from Explorer onto file tree → file copied, tree refreshes
7. Create folder in file tree → delete file via right-click → rename file
8. Double-click `鲁迅.md` → system default program opens
9. Create 3-level group tree → assign item to sub-group
10. Create tags → tag items → click tag/group in sidebar → list filters
11. Delete item → folder gone, all DB rows cascade-cleaned
12. Switch theme → persists across restart

**Plus:**
- All Rust unit + integration tests pass
- All Vitest tests pass
- E2E smoke test passes
- All docs present and accurate

---

## 10. Documentation Requirements

| File | Content |
|---|---|
| `README.md` | Project intro, quick start, screenshots |
| `README_zh.md` | Chinese version |
| `LICENSE` | MIT |
| `CONTRIBUTING.md` | Dev setup, code style, PR workflow |
| `CHANGELOG.md` | Version history |
| `docs/architecture.md` | Architecture overview |
| `docs/data-model.md` | DB schema + Rust structs + repo layout |
| `docs/user-guide.md` | Concepts, common tasks, troubleshooting |
| `docs/developer-guide.md` | Module structure, IPC API, extension points |

---

## 11. Phase Roadmap

| Phase | Content | Deliverable |
|---|---|---|
| **1** (current) | Object manager: 3-pane + custom types + groups + tags + file tree + theme | Windows NSIS installer |
| **2** | Item links (standard Markdown `[text](../hash/)` → `item_links` table, mutual-link dedup) | + upgrade |
| **3** | Plugin system (view slots, panel slots, command palette, Web Components) | + SDK docs |
| **4** | AI integration (Rust LLM proxy, auto-tag, summarization plugins) | + plugin examples |
| **Future** | Workspace snapshots, custom views (calendar/gallery/kanban), sync | TBD |

---

## 12. Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| WebView2 missing on user's Windows | Medium | NSIS installer detects + guides to download |
| User manually deletes item folder | Medium | Detect on next `list_files` call, show broken indicator |
| Symlink path traversal | Medium | All write ops pass `safe_path()` whitelist check |
| File tree perf with many files | Low | Recursive component with lazy-load; typical item folder <100 files |
| ID collision (12 hex = 48 bits) | Negligible | ~2.8×10¹⁴ values; collision probability < cosmic-ray bit-flip |
| Network drive IO | Medium | Document as unsupported in user guide |
