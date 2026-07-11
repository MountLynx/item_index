# Index Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a local-first object/project manager with Tauri 2 + Vue 3 + Rust + SQLite — three-pane layout, custom types, groups, tags, self-built file tree, theme switch.

**Architecture:** Rust backend (Tauri 2) handles all data & filesystem ops via IPC. Vue 3 frontend consumes through Pinia stores with self-built components. SQLite for metadata; filesystem for item folders & attachments. Windows only, NSIS installer.

**Tech Stack:** Tauri 2, Vue 3 + TypeScript, Pinia, Rust, SQLite (sqlx), pnpm + Cargo workspaces, Vite.

## Global Constraints

- Windows only. All file writes pass `safe_path()` — must not escape repo root.
- All IPC JSON. Frontend never touches SQLite directly.
- No third-party UI library (no Element Plus, Naive UI). No ORM. No Markdown parser.
- ID: 12 hex chars, `hex(sha256(rand_16_bytes)[..6])`. Pure random, not content-based.
- Item creation auto-generates `<name>.md` with content `# <name>\n`.
- Properties auto-save on blur, 500ms debounce. No save button.
- Double-click file → `open::that()` system default program.
- Theme persisted to `.index/state.json`. Timestamps: ISO 8601.

---

### Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `pnpm-workspace.yaml`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`
- Create: `src/main.ts`, `src/App.vue`, `src/vite-env.d.ts`, `index.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

**Dependencies — package.json:** vue 3.5, pinia 2.3, @tauri-apps/api 2.2, @tauri-apps/plugin-shell 2.2. Dev: @tauri-apps/cli 2.2, @vitejs/plugin-vue 5.2, typescript 5.7, vue-tsc 2.2, vite 6, vitest 2.1.

**Dependencies — Cargo.toml:** tauri 2, serde + derive, serde_json, sqlx (sqlite + migrate + chrono), sha2, rand, chrono (serde), thiserror, open. Dev: tempfile.

- [ ] **Step 1: Create all scaffold files**

Write every file. `App.vue` renders `<div>Index</div>`. `lib.rs` exposes one `greet` command. `tauri.conf.json` sets window 1200x800 min 900x600, identifier com.index.app, frontendDist ../dist, devUrl localhost:1420.

- [ ] **Step 2: Install & verify**

Run: `pnpm install && cd src-tauri && cargo check`
Expected: both pass. Then: `pnpm tauri dev` → window opens with "Index". Close.

- [ ] **Step 3: Commit**

```bash
git init && git add -A
git commit -m "feat: scaffold Tauri 2 + Vue 3 + pnpm + Cargo monorepo"
```

---

### Task 2: Database — Migrations + Models + Pool

**Files:**
- Create: `src-tauri/migrations/20260710000001_init.sql`
- Create: `src-tauri/src/models.rs`, `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs` (add modules)
- Create: `src-tauri/tests/db_test.rs`

**Schema** (7 tables):
- `item_types` (id, name, icon). Preset: 通用📄, 任务✅.
- `fields` (id, type_id FK CASCADE, name, field_type='text'|'checkbox', position). Preset: 通用.备注(text), 任务.描述(text), 任务.完成(checkbox).
- `items` (id TEXT PK 12hex, name, type_id FK, properties TEXT='{}', namespace='default', created_at TEXT, updated_at TEXT)
- `groups` (id, parent_id FK self CASCADE, name, position, namespace)
- `tags` (id, name UNIQUE)
- `item_groups` (item_id FK, group_id FK, PK)
- `item_tags` (item_id FK, tag_id FK, PK)

**Models:** ItemType, Field, Item, ItemDetail (item + item_type + groups + tags + files: FileNode), Group (with children: Vec<Group>), Tag, FileNode, RepoInfo.

**db.rs:** `create_pool(path)` → SqlitePool with `sqlite:{path}?mode=rwc` max 5 connections. `run_migrations(pool)` → `sqlx::migrate!("./migrations").run(pool)`.

- [ ] **Step 1: Write migration SQL with preset data**
- [ ] **Step 2: Write models.rs with all structs (Serialize, Deserialize)**
- [ ] **Step 3: Write db.rs (pool + migration runner)**
- [ ] **Step 4: Write test — in-memory pool, run migrations, assert 2 types + 3 fields**
- [ ] **Step 5: Run `cargo test` → pass → commit**

---

### Task 3: safe_path Utility

**Files:**
- Create: `src-tauri/src/safe_path.rs`
- Modify: `src-tauri/src/lib.rs` (add mod)

**Interface:** `safe_path(repo_root, item_id, rel_path) -> Result<PathBuf, String>`

**Logic:** canonicalize repo_root → build `repo_root/item_id/rel_path` → if exists, canonicalize; if not, walk up to nearest existing ancestor, canonicalize, append tail → verify `starts_with(repo_root)` → return path or "Path traversal detected".

- [ ] **Step 1: Implement + 3 tests (valid, traversal blocked, nonexistent valid)**
- [ ] **Step 2: Run `cargo test` → all pass → commit**

---

### Task 4: AppState + Repository Commands

**Files:**
- Create: `src-tauri/src/state.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/repo.rs`
- Modify: `src-tauri/src/lib.rs`

**state.rs:** `AppState { db: Mutex<Option<DbPool>>, repo_path: Mutex<Option<String>>, theme: Mutex<String> }` plus `fn new()`.

**repo.rs helper:** `fn get_pool(state) -> Result<SqlitePool, String>` — `state.db.lock().unwrap().clone().ok_or("No repo open")`.

**Commands:**
- `create_repo(path)`: mkdir `.index/`, create pool, run migrations, write `state.json={"theme":"light"}`, set state, return RepoInfo
- `open_repo(path)`: verify `.index/` exists, create pool, run migrations, set state, return RepoInfo
- `close_repo()`: take then close pool, clear path
- `get_repo_info()`: COUNT items, return RepoInfo

- [ ] **Step 1: Write state.rs**
- [ ] **Step 2: Write commands/repo.rs with all 4 commands**
- [ ] **Step 3: Wire in lib.rs — manage(AppState), register 4 commands**
- [ ] **Step 4: `cargo check` → clean → commit**

---

### Task 5: Type & Field Commands

**Files:**
- Create: `src-tauri/src/commands/types.rs`
- Modify: commands/mod.rs, lib.rs

**Commands:**
- `list_item_types`: query types + fields, assemble Vec<ItemType>
- `create_item_type(name, icon?)`: INSERT, RETURNING id
- `delete_item_type(id)`: block id 1,2 (preset), DELETE CASCADE
- `add_field(type_id, name, field_type)`: MAX(position)+1, INSERT RETURNING id
- `remove_field(field_id)`: DELETE
- `reorder_fields(type_id, field_ids[])`: loop UPDATE position=i

- [ ] **Step 1: Write commands/types.rs**
- [ ] **Step 2: Register → `cargo check` → commit**

---

### Task 6: Item Commands

**Files:**
- Create: `src-tauri/src/commands/items.rs`
- Modify: commands/mod.rs, lib.rs
- Add `hex` crate to Cargo.toml

**ID generation:** `sha256(rand_16_bytes)` → first 6 bytes → hex → 12 chars.

**Commands:**
- `create_item(type_id, name)`: generate id, chrono::now to_rfc3339, INSERT, mkdir `<repo>/<id>/`, write `<repo>/<id>/<name>.md` = `# <name>\n`, return Item
- `get_item(id)`: query item + type + fields + groups + tags, call list_files, return ItemDetail
- `list_items(group_id?, tag_id?)`: dynamic JOIN query, return Vec<Item>
- `update_item(id, name?, properties?)`: dynamic UPDATE, update updated_at, RETURNING *
- `delete_item(id)`: safe_path + remove_dir_all hash folder, DELETE item (CASCADE)

- [ ] **Step 1: Implement id generation + all 5 commands**
- [ ] **Step 2: Register → `cargo check` → commit**

---

### Task 7: Group & Tag Commands

**Files:**
- Create: `src-tauri/src/commands/groups.rs`, `src-tauri/src/commands/tags.rs`
- Modify: commands/mod.rs, lib.rs

**Groups:**
- `list_groups`: SELECT all → build tree (root nodes parent_id IS NULL, recursively attach children)
- `create_group(name, parent_id?)`: position = MAX+1 among siblings, INSERT
- `update_group(id, name?)`: UPDATE
- `delete_group(id)`: DELETE CASCADE
- `move_group(id, parent_id?, pos)`: UPDATE parent_id + position
- `add_item_to_group` / `remove_item_from_group`: INSERT/DELETE on item_groups

**Tags:**
- `list_tags`: SELECT all ORDER BY name
- `create_tag(name)`: INSERT (UNIQUE → error on dupe)
- `delete_tag(id)`: DELETE CASCADE
- `add_tag_to_item` / `remove_tag_from_item`: INSERT/DELETE on item_tags

- [ ] **Step 1: Write groups.rs with tree assembly**
- [ ] **Step 2: Write tags.rs**
- [ ] **Step 3: Register → `cargo check` → commit**

---

### Task 8: File Commands

**Files:**
- Create: `src-tauri/src/commands/files.rs`
- Modify: commands/mod.rs, lib.rs

**`read_dir_recursive(path) -> Vec<FileNode>`:** read_dir, for each entry get name + is_dir, recursively read subdirectories, sort dirs-first then alpha.

**Commands:**
- `list_files(item_id)`: build `<repo>/<item_id>/`, call read_dir_recursive, return FileNode with children
- `create_folder(item_id, rel_path)`: safe_path → `std::fs::create_dir`
- `delete_file(item_id, rel_path)`: safe_path → if dir: `remove_dir_all`, else: `remove_file`
- `rename_file(item_id, old, new)`: safe_path both → `std::fs::rename`
- `add_attachment(item_id, source_path)`: safe_path target dir, detect collision → auto-rename (`x.jpg` → `x (2).jpg`), `std::fs::copy`
- `open_file(item_id, rel_path)`: safe_path → `open::that(path)` system default

- [ ] **Step 1: Implement read_dir_recursive + 6 commands**
- [ ] **Step 2: Register → `cargo check` → commit**

---

### Task 9: TypeScript Bindings & Pinia Stores

**Files:**
- Create: `src/types/bindings.ts` (manual, mirroring Rust models)
- Create: `src/stores/repo.ts`, `src/stores/types.ts`, `src/stores/items.ts`
- Create: `src/stores/groups.ts`, `src/stores/tags.ts`, `src/stores/theme.ts`

**bindings.ts:** TypeScript interfaces matching Rust structs — `ItemType`, `Field`, `Item`, `ItemDetail`, `Group`, `Tag`, `FileNode`, `RepoInfo`.

**stores/repo.ts:** `useRepoStore` — state: repoPath, itemCount, isOpen. Actions: `open(path)`, `create(path)`, `close()`, `refresh()`. All call `invoke()`.

**stores/types.ts:** `useTypeStore` — state: types[], loading. Actions: `fetchAll()`, `create(name, icon?)`, `remove(id)`, `addField(typeId, name, fieldType)`, `removeField(id)`, `reorderFields(typeId, ids[])`. Getter: `getTypeById(id)`.

**stores/items.ts:** `useItemStore` — state: items[], selectedId, detail: ItemDetail|null. Actions: `fetchList(groupId?, tagId?)`, `select(id)` → fetches detail, `create(typeId, name)`, `update(id, data)`, `remove(id)`. Auto-save helper: `saveProperties(id, properties)` — debounced 500ms, calls `invoke('update_item', {id, properties})`.

**stores/groups.ts:** `useGroupStore` — state: tree[]. Actions: `fetchAll()`, `create(name, parentId?)`, `update(id, name)`, `remove(id)`, `move(id, parentId, pos)`, `addItem(itemId, groupId)`, `removeItem(itemId, groupId)`.

**stores/tags.ts:** `useTagStore` — state: tags[]. Actions: `fetchAll()`, `create(name)`, `remove(id)`, `addToItem(itemId, tagId)`, `removeFromItem(itemId, tagId)`.

**stores/theme.ts:** `useThemeStore` — state: mode ('light'|'dark'). Actions: `toggle()`, `load()` (reads from state.json via invoke), `persist()` (writes to state.json).

- [ ] **Step 1: Write bindings.ts**
- [ ] **Step 2: Write all 6 Pinia stores**
- [ ] **Step 3: `pnpm build` → no TS errors → commit**

---

### Task 10: Shell Layout — App.vue, Topbar, Sidebar, CenterList, StatusBar

**Files:**
- Create: `src/components/Topbar.vue`, `src/components/Sidebar.vue`
- Create: `src/components/CenterList.vue`, `src/components/StatusBar.vue`
- Create: `src/components/EmptyState.vue`
- Modify: `src/App.vue`
- Create: `src/assets/theme.css`

**theme.css:** CSS variables for light/dark — `--bg`, `--text`, `--accent`, `--border`, `--surface`, `--text-secondary`. Light: `#FFF / #111 / #2563EB`. Dark: `#0A0A0A / #E5E5E5 / #60A5FA`. `body` uses these vars. `.dark` class overrides.

**App.vue:** Three-column flex layout. On mount: check if repo open (try get_repo_info). If not → show EmptyState (two big buttons: "打开仓库" / "创建仓库"). If yes → show full layout. Wrap everything with `:class="{ dark: theme.mode === 'dark' }"`.

**Topbar.vue:** Flex row. Left: repo name (from store). Right buttons: `+ 新建条目` (opens NewItemDialog), `🌓` (toggles theme), `⚙` (opens SettingsPanel). Emits events or directly uses stores.

**Sidebar.vue:** 220px fixed width, flex column. Top section: GroupTree. Bottom section: TagList. Divider between them.

**CenterList.vue:** Flex-1. Renders filtered items from itemStore. Each row: type icon + name + type name + time ago. Click row → itemStore.select(id). Right-click context menu: "删除条目" (with confirm), "添加到分组..." (submenu). Empty state when no items: "创建第一个条目".

**StatusBar.vue:** Bottom bar. Left: repo path. Center: item count. Right: save status indicator.

- [ ] **Step 1: Write theme.css with CSS variables**
- [ ] **Step 2: Write App.vue with conditional layout (repo/no-repo)**
- [ ] **Step 3: Write Topbar, StatusBar**
- [ ] **Step 4: Write EmptyState (two buttons, calls repoStore.create/open)**
- [ ] **Step 5: Write CenterList with right-click menu**
- [ ] **Step 6: Write Sidebar shell (container for GroupTree + TagList)**
- [ ] **Step 7: `pnpm tauri dev` → layout visible → commit**

---

### Task 11: GroupTree & TagList Components

**Files:**
- Create: `src/components/GroupTree.vue`, `src/components/GroupTreeNode.vue`
- Create: `src/components/TagList.vue`

**GroupTreeNode.vue (recursive):**
Props: `group: Group`, `depth: number`. Template: indent by depth*16px, expand/collapse arrow (if has children), group name (editable on double-click → inline input, Enter to save). Right-click menu: "新建子分组" (shows inline input), "重命名" (inline edit), "删除" (confirm), "上移"/"下移" (reorder). Drag handle for reorder. Click selects filter. Emits: `select(groupId)`, `update`, `delete`, `create-child`, `move`.

**GroupTree.vue:**
Fetches groups on mount. Renders root nodes as GroupTreeNode list. Below the tree: "+ 新建根分组" button → inline input. Handles all CRUD via groupStore. Selected group highlighted. Emits `select(groupId)` for CenterList filtering.

**TagList.vue:**
Fetches tags on mount. Renders tag chips/list. At bottom: inline input "+ 新建标签" → Enter creates tag via tagStore. Each tag: click to filter, right-click to delete. Selected tag highlighted. Emits `select(tagId)`.

- [ ] **Step 1: Write GroupTreeNode.vue (recursive, inline edit, context menu)**
- [ ] **Step 2: Write GroupTree.vue (fetch, render roots, new group input)**
- [ ] **Step 3: Write TagList.vue (chips, new tag input, right-click delete)**
- [ ] **Step 4: Wire into Sidebar.vue → `pnpm tauri dev` test → commit**

---

### Task 12: RightPanel — PropertiesForm & FileTree

**Files:**
- Create: `src/components/RightPanel.vue`, `src/components/PropertiesForm.vue`
- Create: `src/components/FileTree.vue`, `src/components/FileTreeNode.vue`
- Create: `src/composables/useFileTree.ts`, `src/composables/useContextMenu.ts`

**RightPanel.vue:** 360px, flex column. Top: item name + ID + type. Middle: PropertiesForm. Then: group membership (chips, removable). Then: tags (chips, removable). Bottom: FileTree. Shows only when itemStore.selectedId is set. Empty state: "选择一个条目查看详情".

**PropertiesForm.vue:**
Props: `item: ItemDetail`. Renders dynamic form from `item.item_type.fields`. For each field:
- field_type='text': `<input>` bound to `properties[field.name]`, @blur → auto-save
- field_type='checkbox': `<input type="checkbox">` bound to `properties[field.name]`, @change → auto-save
Auto-save calls `itemStore.saveProperties(id, properties)` with 500ms debounce.

**useContextMenu.ts:**
`useContextMenu()` returns `{ show, x, y, items, open(event, menuItems), close() }`. Renders absolute-positioned `<ul>` at (x,y) with menu items. Each item: `{ label, action, danger? }`. Closes on click outside.

**useFileTree.ts:**
`useFileTree(itemId: Ref<string>)` returns `{ files, refresh, addAttachment, createFolder, deleteFile, renameFile, openFile }`. `refresh()` calls `invoke('list_files', {itemId})`. `addAttachment` calls invoke. `createFolder(relPath)` calls invoke then refresh. `deleteFile` confirms then invoke + refresh. `renameFile` inline then invoke. `openFile` calls invoke('open_file').

**FileTreeNode.vue (recursive):**
Props: `node: FileNode`, `itemId: string`, `depth: number`. Template: indent, folder icon 📁 or file icon 📄, name. Double-click: if dir → toggle expand, if file → `openFile()`. Right-click: "新建文件夹" (dirs only), "重命名" (inline edit), "删除" (confirm). Drag-over highlight zone. Props for expanded state.

**FileTree.vue:**
Uses `useFileTree`. Watches selected item → refresh. Renders FileTreeNode root. Drop zone: @drop.prevent → get file path from dataTransfer → `addAttachment`. @dragover.prevent → visual feedback. Context menu on empty area: "新建文件夹". File count badge in header.

- [ ] **Step 1: Write useContextMenu composable**
- [ ] **Step 2: Write useFileTree composable**
- [ ] **Step 3: Write PropertiesForm.vue (dynamic form, auto-save)**
- [ ] **Step 4: Write FileTreeNode.vue (recursive, icons, expand/collapse)**
- [ ] **Step 5: Write FileTree.vue (drop zone, context menu, refresh)**
- [ ] **Step 6: Write RightPanel.vue (assemble all sections)**
- [ ] **Step 7: `pnpm tauri dev` → full interaction test → commit**

---

### Task 13: Dialogs — NewItem, SettingsPanel, ConfirmDialog, Toast

**Files:**
- Create: `src/components/NewItemDialog.vue`, `src/components/SettingsPanel.vue`
- Create: `src/components/ConfirmDialog.vue`, `src/components/Toast.vue`
- Create: `src/composables/useToast.ts`

**NewItemDialog.vue:**
Modal. Step 1: select type from dropdown (list from typeStore). Step 2: enter item name. Bottom: "创建" button → `itemStore.create(typeId, name)` → close. "取消" button. Emits `close`.

**SettingsPanel.vue:**
Slide-out panel from right (or modal). Section "条目类型": list existing types with delete button (disabled for preset). "新建类型" → inline name + icon picker. Click type → expand: shows fields list with remove button, "+ 添加字段" → inline name + type dropdown (text/checkbox). Reorder fields via drag handle or up/down buttons.

**ConfirmDialog.vue:**
Props: `title`, `message`, `confirmText?`, `danger?`. Emits: `confirm`, `cancel`. Simple modal with two buttons.

**useToast.ts:**
`useToast()` returns `{ toasts, success(msg), error(msg), info(msg), remove(id) }`. Manages reactive array. Auto-remove after 3s.

**Toast.vue:**
Positioned bottom-right. Renders toast array from composable. Color-coded: success green, error red, info blue. Each toast has close button.

- [ ] **Step 1: Write useToast + Toast.vue**
- [ ] **Step 2: Write ConfirmDialog.vue**
- [ ] **Step 3: Write NewItemDialog.vue**
- [ ] **Step 4: Write SettingsPanel.vue (types list + fields manager)**
- [ ] **Step 5: Wire all dialogs into App.vue (teleport or conditional render)**
- [ ] **Step 6: `pnpm tauri dev` → test all dialogs → commit**

---

### Task 14: Integration — Filtering, Theme, Polish

**Files:**
- Modify: `src/App.vue`, `src/components/Sidebar.vue`
- Modify: `src/stores/items.ts` (add filter logic)

**Filtering logic in Sidebar:**
When user clicks group in GroupTree → `selectedGroupId` ref. When user clicks tag in TagList → `selectedTagId` ref. When either changes → `itemStore.fetchList(selectedGroupId, selectedTagId)`. Both can be selected simultaneously (AND filter — items in that group AND with that tag). Click again to deselect (set to null). "Clear filter" when no items match.

**Theme persistence:**
On app mount → `themeStore.load()` reads `.index/state.json` via custom invoke or Tauri path API. On toggle → `themeStore.toggle()` → `persist()` writes state.json. Apply CSS class `dark` to `document.documentElement`.

**Error handling:**
Wrap all `invoke()` calls in try-catch. On error → `toast.error(message)`. DB-locked errors show "数据库忙碌，重试中...". File operation errors show specific messages.

**Performance:** Ensure list_files called once when selecting item, cached. Debounce auto-save 500ms.

- [ ] **Step 1: Implement group+tag AND filtering in Sidebar**
- [ ] **Step 2: Implement theme load/persist on app mount**
- [ ] **Step 3: Add try-catch + toast to all invoke calls**
- [ ] **Step 4: Full manual smoke test → commit**

---

### Task 15: Rust Integration Tests

**Files:**
- Create: `src-tauri/tests/integration_test.rs`

**Tests use tempdir for each test case:**

1. `test_create_and_open_repo`: create temp dir → invoke create_repo logic → verify `.index/index.db` exists → verify `.index/state.json` exists
2. `test_create_item`: open repo → create item → verify folder exists → verify `<name>.md` exists with correct content → verify DB row matches
3. `test_item_crud`: create → read (get_item returns ItemDetail with files) → update name → verify updated → delete → verify folder gone + DB row gone
4. `test_group_tree`: create root group → create child group → verify tree structure → delete parent → verify CASCADE removes child
5. `test_tags`: create tag → add to item → list item → verify tag present → remove tag → verify gone
6. `test_file_operations`: create item → create_folder → verify on disk → rename file → verify → delete file → verify gone
7. `test_safe_path_blocks_escape`: feed `../../` path → verify error
8. `test_cascade_delete`: create item with groups + tags + files → delete item → verify folder gone, junction rows gone

Run: `cd src-tauri && cargo test --test integration_test` → all pass.

- [ ] **Step 1: Write all 8 integration tests**
- [ ] **Step 2: Run tests → all pass → commit**

---

### Task 16: Frontend Unit Tests (Vitest)

**Files:**
- Create: `src/stores/__tests__/theme.test.ts`, `src/stores/__tests__/items.test.ts`
- Create: `src/components/__tests__/FileTreeNode.test.ts`, `src/components/__tests__/PropertiesForm.test.ts`
- Modify: `vite.config.ts` (add vitest config)

**Setup:** Vitest with jsdom, `@vue/test-utils`. Mock `@tauri-apps/api` invoke globally.

**theme.test.ts:**
- Test: `toggle()` switches light→dark→light
- Test: initial state is 'light'

**items.test.ts:**
- Test: `fetchList` calls invoke with correct params
- Test: `select` fetches detail and sets selectedId
- Test: `create` calls invoke and refreshes list
- Test: `saveProperties` debounces — only one invoke call within 500ms window

**FileTreeNode.test.ts:**
- Test: renders file name, emits dblclick
- Test: renders folder with expand/collapse arrow
- Test: shows children when expanded, hides when collapsed

**PropertiesForm.test.ts:**
- Test: renders text input for 'text' field type
- Test: renders checkbox for 'checkbox' field type
- Test: emits save on blur (debounced)

Run: `pnpm vitest run` → all pass.

- [ ] **Step 1: Configure vitest + mock Tauri invoke**
- [ ] **Step 2: Write theme store tests**
- [ ] **Step 3: Write items store tests**
- [ ] **Step 4: Write FileTreeNode component test**
- [ ] **Step 5: Write PropertiesForm component test**
- [ ] **Step 6: Run `pnpm vitest run` → all pass → commit**

---

### Task 17: E2E Smoke Test

**Files:**
- Create: `tests/e2e/smoke.spec.ts`
- Modify: `package.json` (add e2e script)

**Setup:** Playwright + `@tauri-apps/api` mock or use Tauri driver.

Smoke test flow:
1. App launches → "打开仓库" / "创建仓库" buttons visible
2. Click "创建仓库" → pick temp dir → repo opens → preset types loaded
3. Create custom type "书籍" → add field "作者" (text) + "已读" (checkbox)
4. Click "+" → select "书籍" → enter "鲁迅" → item appears in center list
5. Click "鲁迅" → right pane shows properties form with 作者 + 已读 fields
6. Edit 作者 → blur → verify save called (no error toast)
7. Right pane file tree shows `鲁迅.md`
8. Create group "文学" → child group "近代" → drag item to "近代"
9. Create tag "中文" → apply to item
10. Click tag "中文" → center list filters
11. Delete item → confirm → item removed from list
12. Switch theme → dark mode applied
13. Close app → reopen repo → all data persists

- [ ] **Step 1: Set up Playwright with Tauri**
- [ ] **Step 2: Write smoke test with 13 steps**
- [ ] **Step 3: Run E2E → pass → commit**

---

### Task 18: Documentation

**Files:**
- Create: `README.md`, `README_zh.md`, `LICENSE`, `CONTRIBUTING.md`, `CHANGELOG.md`
- Create: `docs/architecture.md`, `docs/data-model.md`, `docs/user-guide.md`, `docs/developer-guide.md`

**README.md:** Project name, one-liner, screenshot placeholder, quick start (install → open → create repo), features list, roadmap, license badge.

**docs/architecture.md:** Overview diagram (Tauri ↔ Vue ↔ Rust ↔ SQLite ↔ Filesystem), module map, IPC flow, data flow.

**docs/data-model.md:** Entity descriptions, full SQL schema, Rust struct mapping, repo directory layout, properties JSON conventions.

**docs/user-guide.md:** Concepts (repo, item, type, field, group, tag, attachment), common tasks (create repo, create type, create item, add attachment, organize groups), troubleshooting.

**docs/developer-guide.md:** Dev setup (prerequisites, pnpm install, cargo build), project structure, how to add a new IPC command, how to add a new component, testing, building installer.

- [ ] **Step 1: Write README.md + README_zh.md**
- [ ] **Step 2: Write LICENSE (MIT) + CONTRIBUTING.md + CHANGELOG.md**
- [ ] **Step 3: Write docs/architecture.md + docs/data-model.md**
- [ ] **Step 4: Write docs/user-guide.md + docs/developer-guide.md**
- [ ] **Step 5: Commit all docs**

---

### Task 19: NSIS Installer & Packaging

**Files:**
- Modify: `src-tauri/tauri.conf.json` (bundle config)
- Create: `src-tauri/icons/` (app icons)

**Steps:**
1. Generate icons (PNG 32x32, 128x128, 256x256, ICO) — use placeholder or simple tool
2. Configure `tauri.conf.json` bundle section:
   - identifier: `com.index.app`
   - windows: NSIS installer
   - NSIS: install mode currentUser, license file path (LICENSE)
3. Run `pnpm tauri build` → produces `.msi` and `.nsis.exe` installers
4. Test install on fresh Windows machine — verify app launches, icon shows
5. Verify uninstall cleans up

- [ ] **Step 1: Create app icons (placeholder OK for Phase 1)**
- [ ] **Step 2: Configure bundle in tauri.conf.json**
- [ ] **Step 3: Run `pnpm tauri build` → verify installer produced**
- [ ] **Step 4: Install/uninstall test → commit**

---

## Execution Order

```
1  → Scaffolding
2  → Database
3  → safe_path
4  → Repo Commands
5  → Type Commands    ┐
6  → Item Commands    ├─ Can parallelize 5-8
7  → Group/Tag Cmds   │
8  → File Commands    ┘
9  → TS Bindings + Pinia Stores
10 → Shell Layout
11 → GroupTree + TagList
12 → RightPanel (PropertiesForm + FileTree)
13 → Dialogs + Toast
14 → Integration (Filtering, Theme, Polish)
15 → Rust Integration Tests
16 → Vitest Tests      ┐
17 → E2E Smoke Test    ├─ Can parallelize 15-17
18 → Documentation     ┘
19 → NSIS Packaging (after all tests pass)
```

---

## Self-Review Notes

- ✅ All 12 spec sections have corresponding tasks
- ✅ All Rust IPC commands from spec §7 mapped to Tasks 4-8
- ✅ All Vue components from spec §8 mapped to Tasks 10-13
- ✅ All acceptance criteria from spec §9 covered by integration/E2E tests
- ✅ No placeholders — every task has specific files, functions, and test names
- ✅ Types consistent: ItemDetail.files: FileNode used in both Rust and Vue
- ✅ Global constraints enforced: safe_path, no UI lib, no ORM, JSON IPC, 12 hex ID
