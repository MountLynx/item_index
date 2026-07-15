# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

When using subagents, avoid the deepseek-v4-pro model as it is known to cause errors (e.g., "deepseek-v4-pro is temporarily unavailable...") due to an internal parameter conflict in subagent requests. Instead, use the deepseek-v4-flash model for all subagent operations. If you encounter the error, switch to flash or a supported Anthropic model. This ensures subagent actions (like file edits, command execution, or other non‑read‑only operations) work without interruption.

**Note:** `CLAUDE.md` is in `.gitignore` — it's a developer-local guide, not tracked in git.

## Project Overview

**Index** — a local-first, plugin-extensible object manager built on Tauri 2. Think "personal wiki meets Zotero": items with dynamic types, multi-level groups, flat tags, and file attachments stored in hash-named folders. The vision is a general-purpose data manager that can become a calendar, library tracker, or knowledge base through plugins and workspace configurations.

The project now also features a **repo dashboard** (grid-card landing screen), **workspace system** (named configs that filter types + add plugin tabs), **plugin system** (user-installed JS plugins with Vue component support), **i18n** (zh-CN + en), and **custom theme presets**.

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
pnpm test             # (no test script in package.json yet — vitest is a devDep)
```

The frontend dev server runs on **port 1420** (strict, configured in `vite.config.ts`). Tauri auto-opens the devtools window in debug mode.

For Rust-side changes:
```bash
cd src-tauri && cargo build   # Build Rust backend
cd src-tauri && cargo test    # Run Rust tests (db_test.rs, safe_path.rs)
```

Rust tests are in `src-tauri/tests/` (one integration test: `db_test.rs`). `safe_path.rs` has `#[cfg(test)]` unit tests inline.

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 (`@tauri-apps/api` v2) |
| Frontend framework | Vue 3.5 + TypeScript (strict mode) |
| State management | Pinia (composition API stores) |
| Icons | `@tabler/icons-vue` 3.x (+ `@tabler/icons-webfont`) |
| i18n | `vue-i18n` v10 (zh-CN + en) |
| Backend | Rust, `sqlx` (SQLite, `runtime-tokio-rustls`), `serde`, `tokio`, `chrono` |
| DB migrations | `sqlx::migrate!` (embedded, `src-tauri/migrations/`) |
| Tests | Rust: `cargo test` (sqlx in-memory tests); Frontend: `vitest` (devDep, no tests yet) |

### App Shell Layout

```
main.ts — createApp(App) + Pinia + i18n
App.vue
├── RepoDashboard          — shown when no repo is open (grid-card: managed repos + create tile)
└── [when repo open]
    ├── Titlebar           — workspace switcher, repo name, "+ new item", theme/settings gear, manage repos, window controls (frameless)
    ├── .main (flex row)
    │   ├── Sidebar             — left panel
    │   │   ├── GroupTree       — multi-level groups, drag items onto groups
    │   │   └── TagList         — flat tag filter
    │   ├── CenterPanel         — tabbed center area
    │   │   ├── [list tab]     — CenterList (item list, drag-to-group, right-click delete)
    │   │   └── [plugin tab]   — plugin-loaded component via usePluginLoader
    │   ├── RightPanel          — item detail / type manager (tabbed)
    │   │   ├── PropertiesForm  — dynamic form from ItemType.fields
    │   │   ├── Group/Tag chips — add/remove groups and tags
    │   │   ├── FileTree        — attached files tree
    │   │   └── TypeManager     — type CRUD (when rightTab='types')
    │   └── ActivityBar         — slim right-side tab selector (48px)
    ├── StatusBar           — repo path, item count
    ├── NewItemDialog       — modal: select type + enter name
    ├── SettingsModal       — modal: theme presets, accent color, locale, type/field management
    └── Toast               — notification stack (shared composable `useToast`, auto-dismiss 3.5s)
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
| `useSettingsStore` | `stores/settings.ts` | `theme`, `general` | Global settings: theme presets, accent color, locale. Persisted via `localStorage`. Loads active preset from repo on open. |
| `useWorkspaceStore` | `stores/workspace.ts` | `workspaces[]`, `activeName`, `activeConfig` | Workspace CRUD + activation. Filters item types, manages center tabs (list + plugin). |
| `useDashboardStore` | `stores/dashboard.ts` | `repos[]` | Managed repo list (grid-card dashboard). |

**Critical pattern**: the `repo` store's `isOpen` computed drives whether `App.vue` shows `RepoDashboard` or the main layout. On repo open, `onRepoOpened()` parallel-fetches `types`, `groups`, `tags`, `items`, then loads active workspace + settings preset.

### Rust Backend Structure

```
src-tauri/src/
├── lib.rs           — Tauri builder: registers all commands, AppState, deploy bundled resources, open devtools
├── main.rs          — Entry point: calls index_lib::run()
├── state.rs         — AppState { db: Mutex<Option<DbPool>>, repo_path, theme }
├── db.rs            — create_pool() + run_migrations() via sqlx
├── models.rs        — Shared data structures (Serialize + Deserialize) — MUST stay in sync with types/bindings.ts
├── safe_path.rs     — Path traversal prevention (unit tested)
└── commands/
    ├── mod.rs       — pub mod declarations (10 modules)
    ├── repo.rs      — create/open/close repo, get_repo_info, get_state, save_state
    ├── dashboard.rs — list_managed_repos, add/remove/update_repo_icon, open_dashboard_window
    ├── types.rs     — list/create/delete item types, add/remove/reorder fields, update_item_type, update_field
    ├── items.rs     — CRUD + list (with optional group_id/tag_id filters)
    ├── groups.rs    — tree/list CRUD + move + item-to-group junction
    ├── tags.rs      — CRUD + item-to-tag junction
    ├── files.rs     — list/create_folder/delete/rename/add_attachment/open_file
    ├── workspace.rs — list/read/write/delete workspaces
    ├── plugin.rs    — list_installed_plugins, read_plugin_file (loads JS from app_data_dir/plugin-store/)
    └── presets.rs   — list/install/export workspace presets, list_global_plugins, install_plugin
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
- Custom fields support `text`, `checkbox`, `date`, and `number`. `position` field controls ordering.
- Groups form a tree via `parent_id`; tags are flat.
- `item_groups` and `item_tags` are many-to-many junction tables.
- File paths are resolved through `safe_path.rs` which canonicalizes and checks for traversal attacks (`../../etc/passwd` → error).
- All entities (item_types, items, groups, tags) carry a `namespace` column (default `'default'`). Currently hardcoded — workspace-level namespace switching is a future feature.
- Managed repos are tracked in `app_data_dir/repos.json` (dashboard table).
- Plugins live in `app_data_dir/plugin-store/<name>/` (manifest.json + index.js).
- Workspace presets live in `app_data_dir/workspace-presets/` (JSON files).

### Design System

A minimal token system in `assets/theme.css` using CSS custom properties:

- All colors defined as tokens (`--bg`, `--surface`, `--text`, `--accent`, `--border`, etc.)
- Light mode on `:root`, dark mode on `.dark` class
- Typography: Inter font, CJK fallback to Microsoft YaHei
- Layout constants: `--sidebar-w: 240px`, `--right-w: 360px`, `--topbar-h: 40px`, `--status-h: 28px`
- Motion: `--fast: 120ms`, `--normal: 180ms`, `--slow: 280ms` with spring-like easing
- Border radius: `--r-sm: 4px`, `--r-md: 6px`, `--r-lg: 10px`
- Settings store supports user-defined theme presets (CSS variable overrides stored in localStorage, with `loadActivePresetFromRepo()` on repo open)

Components style directly with scoped `<style>` blocks referencing these tokens.

### TablerIcon Component

`TablerIcon.vue` wraps `@tabler/icons-vue` with a static import map of ~25 icons (plus a separate `assets/icon-names.ts` list of 2000+ names). If a name isn't in the map, it falls back to rendering the name directly as an emoji (if the name is emoji characters) or a `◆` placeholder. This allows user-specified type icons to be either Tabler icon names or raw emoji.

### Plugin System

Plugins are user-installed JS files loaded at runtime from `app_data_dir/plugin-store/<name>/`:
- `manifest.json` — declares `name`, `version`, `title`, `icon`, `extends` (center-panel|right-panel|sidebar), `requiresFields`, `config`
- `index.js` — CommonJS-like module executed via `new Function('exports', jsCode)`. Can export a factory function that receives `{ h, ref, computed, watch, onMounted }` from Vue to create render functions.

Plugin loading is handled by `usePluginLoader` composable. Active plugins receive a `PluginContext` (built by `usePluginContext`) exposing stores and actions.

### Workspace System

Named workspace configs stored as JSON in the repo (`.index/workspaces/<name>.json`). Each workspace defines:
- `itemTypes` — which item types to show (empty = all)
- `centerTabs` — ordered tabs for the center panel (`list` or `plugin` type)
- `defaultTab` — which tab is active by default
- `rightPanelAddons`, `sidebarAddons` — plugin slots in other panels

Activated via `WorkspaceSwitcher` in the Titlebar or programmatically. The `useWorkspaceStore` loads the default workspace on repo open.

### i18n

Two locale files: `locales/zh-CN.ts` (default) and `locales/en.ts`. Loaded via `vue-i18n` with `createI18n()` in `plugins/i18n.ts`. Locale is configurable in Settings modal. Components use `$t('key')` syntax.

### Key Implementation Details

- **`list_items` filtering**: Supports `group_id` and `tag_id` independently or combined (both filters → `DISTINCT` join through both junction tables).
- **Group tree building**: Rust `build_tree()` recursively builds the nested `Vec<Group>` from flat rows in `list_groups`.
- **File tree rendering**: `FileTree.vue` only triggers `list_files` when `itemId` changes (dedup via `last` ref); `FileTreeNode.vue` lazily expands directories on click.
- **Item drag-to-group**: `CenterList.vue` sets `text/plain` data on dragstart; `GroupTreeNode.vue` checks `dataTransfer.types` on dragover and calls `addItemToGroup` on drop.
- **Properties auto-save**: `PropertiesForm.vue` mutates `detail.item.properties` directly and calls `saveProperties()` which debounces 500ms before invoking `update_item`.
- **Repo dashboard**: `RepoDashboard.vue` shows grid cards (RepoCard, RepoCreateTile) for managed repos. Opening a repo calls `onRepoOpened()` which loads all stores.
- **Frameless window**: `tauri.conf.json` sets `decorations: false`. Window controls (minimize/maximize/close) are in `Titlebar.vue` using `@tauri-apps/api/window` `getCurrentWindow()`.
- **Bundled resources**: On first run, `lib.rs::deploy_bundled_resources()` copies bundled plugins and presets from the Tauri resource dir to `app_data_dir`.
- **Cargo.toml key deps**: `tauri` (with devtools), `tauri-plugin-shell`, `tauri-plugin-dialog`, `sqlx` (sqlite + migrate + chrono + runtime-tokio-rustls), `sha2`, `rand`, `chrono`, `thiserror`, `hex`, `open`.
- **package.json key deps**: `@tauri-apps/api` ^2.2, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-shell`, `pinia`, `vue` ^3.5, `vue-i18n` ^10, `@tabler/icons-vue` ^3.44.
- **package.json devDeps**: `@tauri-apps/cli`, `@vitejs/plugin-vue`, `typescript` ~5.7, `vite` ^6, `vitest` ^2.1, `vue-tsc` ^2.2.

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
- Features not yet built (full plugin SDK, vault/backup, AI integration) — design them before implementing

## Design Docs

- `docs/design.md` — original vision and full functional requirements spec (Chinese). Covers the 6-phase roadmap: core engine, UI layout, file system, plugin system, AI integration, workspace configs.
- `docs/superpowers/specs/` — design specs for recent features (vault phase 1, type manager redesign, icon picker, i18n, repo dashboard, theme fixes/redesign, titlebar redesign, workspace plugin system).
- `docs/superpowers/plans/` — implementation plans corresponding to each spec.
