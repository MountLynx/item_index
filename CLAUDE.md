# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Index** — a local-first, plugin-extensible object manager built on Tauri 2. Think "personal wiki meets Zotero": items with dynamic types, multi-level groups, flat tags, and file attachments stored in hash-named folders. The vision is a general-purpose data manager that can become a calendar, library tracker, or knowledge base through plugins and workspace configurations.

## Reference Documentation

**Before implementing, consult the official docs first.** Many bugs stem from framework defaults rather than code errors.

| Resource | URL | When to check |
|----------|-----|---------------|
| Vue 3 | https://cn.vuejs.org/guide/introduction.html | Template syntax, reactivity, component API |
| Tauri 2 | https://v2.tauri.app/ | Window config, IPC, plugins, platform specifics |
| Tauri GitHub | https://github.com/tauri-apps/tauri | Issues, config schema, breaking changes |

**Known Tauri 2 Windows pitfall:** `dragDropEnabled` defaults to `true`, which makes Tauri intercept all HTML5 drag-and-drop at the native window level — `dragover`/`dragenter`/`dragleave`/`drop` events never reach the WebView JS layer. Must set `"dragDropEnabled": false` in `tauri.conf.json` → `app.windows[0]` to use HTML5 drag-and-drop on the frontend.

## Commands

```bash
pnpm dev              # Start Vite dev server (port 1420), then launch Tauri
pnpm build            # Type-check (vue-tsc --noEmit) + Vite build
pnpm preview          # Preview built frontend
pnpm tauri            # Tauri CLI (e.g., pnpm tauri build, pnpm tauri dev)
```

The frontend dev server runs on **port 1420** (strict, configured in `vite.config.ts`). Tauri auto-opens the devtools window in debug mode.

For Rust-side changes, `cargo build` / `cargo test` inside `src-tauri/` work as usual. There are unit tests in `safe_path.rs` for path traversal prevention.

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 (`@tauri-apps/api` v2) |
| Frontend framework | Vue 3.5 + TypeScript (strict mode) |
| State management | Pinia (composition API stores) |
| Icons | `@tabler/icons-vue` 3.x |
| Backend | Rust, `sqlx` (SQLite), `serde`, `tokio`, `chrono` |
| DB migrations | `sqlx::migrate!` (embedded, `src-tauri/migrations/`) |

### App Shell Layout

```
App.vue
├── EmptyState         — shown when no repo is open (create/open repo)
└── [when repo open]
    ├── Topbar         — repo name, "+ new item", theme toggle, settings gear
    ├── .main (flex row)
    │   ├── Sidebar        — left panel
    │   │   ├── GroupTree  — multi-level groups, drag items onto groups
    │   │   └── TagList    — flat tag filter
    │   ├── CenterList     — item list (drag-to-group, right-click delete)
    │   └── RightPanel     — item detail
    │       ├── PropertiesForm — dynamic form from ItemType.fields
    │       ├── Group/Tag chips — add/remove groups and tags
    │       └── FileTree       — attached files tree
    ├── StatusBar       — repo path, item count
    ├── NewItemDialog   — modal: select type + enter name
    ├── SettingsPanel   — modal: manage item types, fields, icons
    └── Toast           — notification stack (success/error/info, auto-dismiss 3.5s)
```

### Pinia Stores (Frontend State)

All stores are composition-style (`defineStore` with setup function):

| Store | File | Key State | Notes |
|-------|------|-----------|-------|
| `useRepoStore` | `stores/repo.ts` | `repoPath`, `itemCount`, `isOpen` | Wraps `create_repo`, `open_repo`, `close_repo` IPC |
| `useTypeStore` | `stores/types.ts` | `types[]`, `loading` | Item type CRUD + field management. `getTypeById` is a computed getter. |
| `useItemStore` | `stores/items.ts` | `items[]`, `selectedId`, `detail` | List + detail; `saveProperties()` auto-debounces 500ms |
| `useGroupStore` | `stores/groups.ts` | `tree[]` | Full tree from `list_groups`; always re-fetches after mutations |
| `useTagStore` | `stores/tags.ts` | `tags[]` | Flat list, optimistic push on create |
| `useThemeStore` | `stores/theme.ts` | `mode` | Light/dark toggle, watches and applies to `<html>` class |

**Critical pattern**: the `repo` store's `isOpen` computed drives whether `App.vue` shows `EmptyState` or the main layout. On repo open, `EmptyState` calls `onRepoOpened()` which parallel-fetches `types`, `groups`, `tags`, `items`.

### Rust Backend Structure

```
src-tauri/src/
├── lib.rs           — Tauri builder: registers all commands, manages AppState, opens devtools
├── state.rs         — AppState { db: Mutex<Option<DbPool>>, repo_path: Mutex<Option<String>>, theme }
├── db.rs            — create_pool() + run_migrations() via sqlx
├── models.rs        — Shared data structures (Serialize + Deserialize)
├── safe_path.rs     — Path traversal prevention (unit tested)
└── commands/
    ├── mod.rs       — pub mod declarations
    ├── repo.rs      — create/open/close repo, get_repo_info
    ├── types.rs     — list/create/delete item types, add/remove/reorder fields
    ├── items.rs     — CRUD + list (with optional group_id/tag_id filters)
    ├── groups.rs    — tree/list CRUD + move + item-to-group junction
    ├── tags.rs      — CRUD + item-to-tag junction
    └── files.rs     — list/create_folder/delete/rename/add_attachment/open_file
```

### IPC Flow

Frontend never touches the database directly. Every operation goes through `invoke<T>()`:

```ts
// Frontend
const items = await invoke<Item[]>('list_items', { groupId, tagId })
```

```rs
// Backend (tauri::command)
#[tauri::command]
pub async fn list_items(state: State<'_, AppState>, group_id: Option<i64>, ...) -> Result<Vec<Item>, String>
```

Tauri automatically serializes/deserializes between TypeScript types (in `types/bindings.ts`) and Rust structs (in `models.rs`). These must stay in sync.

### Data Model & Repository Layout

```
~/MyIndex/               ← repo root (user-chosen directory)
├── .index/
│   ├── index.db          ← SQLite database
│   └── state.json        ← {"theme":"light"}
├── a3f2c1b8e9d4/        ← item hash folder (12 hex chars, SHA256 of random 16 bytes → first 6 bytes)
│   ├── My Note.md        ← auto-generated on item creation
│   └── ...               ← arbitrary files/attachments
├── b7e1d5c3f2a8/
└── ...
```

**Key data model rules:**
- Item IDs are 12-char hex, not UUIDs. Generated by `rand + SHA256` in the Rust layer.
- Item types 1 and 2 are presets (通用=general, 任务=task) — `delete_item_type` refuses to delete them.
- Custom fields support `text` and `checkbox`. `position` field controls ordering.
- Groups form a tree via `parent_id`; tags are flat.
- `item_groups` and `item_tags` are many-to-many junction tables.
- File paths are resolved through `safe_path.rs` which canonicalizes and checks for traversal attacks (`../../etc/passwd` → error).
- All entities (item_types, items, groups, tags) carry a `namespace` column (default `'default'`). Currently hardcoded — workspace-level namespace switching is a future feature.

### Design System

A minimal token system in `assets/theme.css` using CSS custom properties:

- All colors defined as tokens (`--bg`, `--surface`, `--text`, `--accent`, `--border`, etc.)
- Light mode on `:root`, dark mode on `.dark` class
- Typography: Inter font, CJK fallback to Microsoft YaHei
- Layout constants: `--sidebar-w: 240px`, `--right-w: 360px`, `--topbar-h: 48px`, `--status-h: 28px`
- Motion: `--fast: 120ms`, `--normal: 180ms`, `--slow: 280ms` with spring-like easing

Components style directly with scoped `<style>` blocks referencing these tokens.

### TablerIcon Component

`TablerIcon.vue` wraps `@tabler/icons-vue` with a static import map of ~25 icons. If a name isn't in the map, it falls back to rendering the name directly as an emoji (if the name is emoji characters) or a `◆` placeholder. This allows user-specified type icons to be either Tabler icon names or raw emoji.

### Key Implementation Details

- **`list_items` filtering**: Supports `group_id` and `tag_id` independently or combined (both filters → `DISTINCT` join through both junction tables).
- **Group tree building**: Rust `build_tree()` recursively builds the nested `Vec<Group>` from flat rows in `list_groups`.
- **File tree rendering**: `FileTree.vue` only triggers `list_files` when `itemId` changes (dedup via `last` ref); `FileTreeNode.vue` lazily expands directories on click.
- **Item drag-to-group**: `CenterList.vue` sets `text/plain` data on dragstart; `GroupTreeNode.vue` checks `dataTransfer.types` on dragover and calls `addItemToGroup` on drop.
- **Properties auto-save**: `PropertiesForm.vue` mutates `detail.item.properties` directly and calls `saveProperties()` which debounces 500ms before invoking `update_item`.
- **Page reload protection**: `EmptyState.vue`'s `openRepo`/`doCreate` functions call `repoStore.openRepo()`/`createRepo()` then load all stores and emit `repoOpened` — no URL-encoded state.
- **Cargo.toml dependencies**: `tauri`, `sqlx` (sqlite, runtime-tokio), `serde`/`serde_json`, `sha2`, `hex`, `rand`, `chrono`, `open`, `tauri-plugin-shell`.

## Backward Compatibility Constraints

All changes MUST preserve compatibility with existing user data. The following are **locked** and must not be broken:

### Database Schema
- Evolve only through additive sqlx migrations — `ALTER TABLE ... ADD COLUMN` with `DEFAULT`
- Never rename or drop existing tables or columns
- Never change existing column types
- Preset item types (id=1, id=2) and their fields must not be deleted or renamed

### Repository Layout
- `.index/index.db` — database path is stable
- `.index/state.json` — top-level keys are additive only; never remove or rename existing keys
- `{12-char-hex}/` item folders — naming algorithm (SHA256 of 16 random bytes, first 6 bytes → 12 hex) is immutable
- `<item-name>.md` auto-generation on item creation is a stable contract

### IPC Interface
- Rust struct fields in `models.rs` are additive only — new fields must be `Option<T>` or have a `Default`
- Command signatures in `lib.rs` may add new optional parameters but never remove or reorder existing ones
- TypeScript types in `types/bindings.ts` mirror Rust structs exactly; keep them in sync

### Not Covered
- Vue component internals, CSS, Pinia store logic — free to refactor
- Features not yet built (plugin system, workspaces, etc.) — design them before implementing

## Design Docs

- `docs/design.md` — original vision and full functional requirements spec (Chinese). Covers the 6-phase roadmap: core engine, UI layout, file system, plugin system, AI integration, workspace configs.
- `docs/superpowers/` — design specs and implementation plans from the Superpowers workflow.
