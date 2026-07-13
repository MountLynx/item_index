# Repo Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the bare-bones `EmptyState.vue` entry screen with a repo management dashboard featuring grid cards, create/import via native file dialog, and multi-window support.

**Architecture:** New Rust `commands/dashboard.rs` reads/writes `repos.json` in Tauri's app data directory. New `useDashboardStore` Pinia store wraps the 4 IPC commands. `RepoDashboard.vue` replaces `EmptyState.vue` in `App.vue`, composing `RepoCard.vue`, `RepoCreateTile.vue`, and `TemplateBanner.vue`. The Titlebar's category button becomes a "repo" button that opens a new dashboard window via `WebviewWindow`.

**Tech Stack:** Tauri 2, Vue 3 + TypeScript, Pinia, `tauri-plugin-dialog`, existing `theme.css` tokens.

## Global Constraints

- All Rust struct fields in `models.rs` are additive only — new structs are fine
- Command signatures in `lib.rs` may add new parameters but never remove or reorder existing ones
- TypeScript types in `types/bindings.ts` mirror Rust structs exactly
- Use existing `theme.css` design tokens — no hardcoded colors
- `dragDropEnabled: false` in tauri.conf.json (existing constraint)
- i18n: add keys to both `zh-CN.ts` and `en.ts`

---

## File Map

| Action | File | Purpose |
|--------|------|---------|
| CREATE | `src-tauri/src/commands/dashboard.rs` | 4 managed-repo IPC commands |
| MODIFY | `src-tauri/src/commands/mod.rs` | Add `pub mod dashboard;` |
| MODIFY | `src-tauri/src/models.rs` | Add `ManagedRepo` struct |
| MODIFY | `src-tauri/src/lib.rs` | Register 4 new commands |
| MODIFY | `src-tauri/Cargo.toml` | Add `tauri-plugin-dialog` dep |
| MODIFY | `src-tauri/tauri.conf.json` | Enable dialog plugin permission |
| CREATE | `src/stores/dashboard.ts` | `useDashboardStore` Pinia store |
| MODIFY | `src/types/bindings.ts` | Add `ManagedRepo` interface |
| CREATE | `src/components/RepoCard.vue` | Single repo grid card |
| CREATE | `src/components/RepoCreateTile.vue` | Create/import entry tile |
| CREATE | `src/components/TemplateBanner.vue` | "Coming soon" banner |
| CREATE | `src/components/RepoDashboard.vue` | Main dashboard layout |
| MODIFY | `src/App.vue` | `EmptyState` → `RepoDashboard` |
| MODIFY | `src/components/Titlebar.vue` | Category button → repo button |
| MODIFY | `src/stores/repo.ts` | Auto-add on open/create |
| MODIFY | `src/locales/zh-CN.ts` | New dashboard i18n keys |
| MODIFY | `src/locales/en.ts` | New dashboard i18n keys |
| MODIFY | `package.json` | Add `@tauri-apps/plugin-dialog` |
| NO CHANGE | `src/components/EmptyState.vue` | Keep as fallback (unreferenced after migration) |

---

### Task 1: Add `tauri-plugin-dialog` dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `@tauri-apps/plugin-dialog` → `open({ directory: true })` available for folder selection in Task 11

- [ ] **Step 1: Add Rust crate dependency**

In `src-tauri/Cargo.toml`, add under `[dependencies]`:

```toml
tauri-plugin-dialog = "2"
```

- [ ] **Step 2: Add npm package**

Run:
```bash
pnpm add @tauri-apps/plugin-dialog
```

Expected: package added to `package.json` with version compatible with Tauri 2.

- [ ] **Step 3: Register dialog plugin in Rust**

In `src-tauri/src/lib.rs`, add the plugin initialization. Change:
```rust
.plugin(tauri_plugin_shell::init())
```
to:
```rust
.plugin(tauri_plugin_shell::init())
.plugin(tauri_plugin_dialog::init())
```

- [ ] **Step 4: Add dialog plugin permission to Tauri config**

In `src-tauri/tauri.conf.json`, under `"app"`, add the plugins permission. The config should include:

```json
"plugins": {
  "dialog": {
    "open": true
  }
}
```

The full `"app"` section becomes:

```json
"app": {
  "windows": [
    {
      "title": "Index",
      "width": 1200,
      "height": 800,
      "minWidth": 900,
      "minHeight": 600,
      "resizable": true,
      "dragDropEnabled": false,
      "decorations": false
    }
  ],
  "security": {
    "csp": null
  },
  "plugins": {
    "dialog": {
      "open": true
    }
  }
}
```

- [ ] **Step 5: Verify build**

Run:
```bash
cd src-tauri && cargo check
```

Expected: compilation succeeds, no errors about `tauri-plugin-dialog`.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/tauri.conf.json package.json pnpm-lock.yaml
git commit -m "chore: add tauri-plugin-dialog for native folder selection"
```

---

### Task 2: Rust — ManagedRepo model

**Files:**
- Modify: `src-tauri/src/models.rs`

**Interfaces:**
- Produces: `ManagedRepo` struct with fields `path: String`, `icon: Option<String>`, `name: Option<String>`, `last_opened_at: String`, `item_count: Option<i32>`

- [ ] **Step 1: Add ManagedRepo struct**

In `src-tauri/src/models.rs`, append after the `RepoInfo` struct (line 71):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedRepo {
    pub path: String,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub last_opened_at: String,
    pub item_count: Option<i32>,
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: PASS (struct compiles, derives are valid).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat: add ManagedRepo model for repo dashboard"
```

---

### Task 3: Rust — dashboard commands

**Files:**
- Create: `src-tauri/src/commands/dashboard.rs`

**Interfaces:**
- Consumes: `ManagedRepo` from `crate::models`, `AppState` from `crate::state`
- Produces: `list_managed_repos`, `add_managed_repo`, `remove_managed_repo`, `update_repo_icon` commands

- [ ] **Step 1: Create dashboard.rs with all 4 commands**

Create `src-tauri/src/commands/dashboard.rs`:

```rust
use tauri::Manager;
use crate::models::ManagedRepo;

fn repos_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("repos.json"))
}

fn read(app: &tauri::AppHandle) -> Result<Vec<ManagedRepo>, String> {
    let p = repos_path(app)?;
    if !p.exists() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn write(app: &tauri::AppHandle, repos: &[ManagedRepo]) -> Result<(), String> {
    let p = repos_path(app)?;
    let json = serde_json::to_string_pretty(repos).map_err(|e| e.to_string())?;
    std::fs::write(&p, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_managed_repos(app: tauri::AppHandle) -> Result<Vec<ManagedRepo>, String> {
    read(&app)
}

#[tauri::command]
pub async fn add_managed_repo(
    app: tauri::AppHandle,
    path: String,
    icon: Option<String>,
    name: Option<String>,
    item_count: Option<i32>,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert: remove existing entry with same path
    repos.retain(|r| r.path != path);

    repos.push(ManagedRepo {
        path,
        icon,
        name,
        last_opened_at: now,
        item_count,
    });

    write(&app, &repos)?;
    Ok(repos)
}

#[tauri::command]
pub async fn remove_managed_repo(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    repos.retain(|r| r.path != path);
    write(&app, &repos)?;
    Ok(repos)
}

#[tauri::command]
pub async fn update_repo_icon(
    app: tauri::AppHandle,
    path: String,
    icon: String,
) -> Result<Vec<ManagedRepo>, String> {
    let mut repos = read(&app)?;
    if let Some(repo) = repos.iter_mut().find(|r| r.path == path) {
        repo.icon = Some(icon);
    }
    write(&app, &repos)?;
    Ok(repos)
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs
git commit -m "feat: add managed repo CRUD commands"
```

---

### Task 4: Register dashboard module and commands

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `commands/dashboard.rs` module
- Produces: 4 commands registered in Tauri invoke handler

- [ ] **Step 1: Register module in mod.rs**

In `src-tauri/src/commands/mod.rs`, append:

```rust
pub mod dashboard;
```

The file should look like:

```rust
pub mod repo;
pub mod types;
pub mod items;
pub mod groups;
pub mod tags;
pub mod files;
pub mod dashboard;
```

- [ ] **Step 2: Register 4 commands in lib.rs**

In `src-tauri/src/lib.rs`, add the four new commands to the `invoke_handler` macro. Add these lines after the `commands::files::open_file` entry (before the closing `]`):

```rust
            commands::dashboard::list_managed_repos,
            commands::dashboard::add_managed_repo,
            commands::dashboard::remove_managed_repo,
            commands::dashboard::update_repo_icon,
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: PASS. All commands registered.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: register dashboard commands in Tauri builder"
```

---

### Task 5: TypeScript bindings — ManagedRepo interface

**Files:**
- Modify: `src/types/bindings.ts`

**Interfaces:**
- Produces: `ManagedRepo` interface matching the Rust struct

- [ ] **Step 1: Add ManagedRepo type**

In `src/types/bindings.ts`, append after `RepoInfo` (line 61):

```ts
export interface ManagedRepo {
  path: string
  icon: string | null
  name: string | null
  last_opened_at: string
  item_count: number | null
}
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no new errors from this addition.

- [ ] **Step 3: Commit**

```bash
git add src/types/bindings.ts
git commit -m "feat: add ManagedRepo TypeScript binding"
```

---

### Task 6: i18n — dashboard strings

**Files:**
- Modify: `src/locales/zh-CN.ts`
- Modify: `src/locales/en.ts`

**Interfaces:**
- Produces: `dashboard` i18n namespace with keys for all dashboard UI text

- [ ] **Step 1: Add zh-CN dashboard strings**

In `src/locales/zh-CN.ts`, insert a `dashboard` block after the `emptyState` block (line 69). Add after the `emptyState` closing `}`:

```ts
  dashboard: {
    title: 'Index',
    tagline: '本地优先 · 插件扩展 · 对象管理',
    createOrImport: '创建或导入仓库',
    newRepo: '新建仓库',
    importRepo: '导入已有仓库',
    removeRepo: '从列表中移除',
    confirmRemove: '确定从列表中移除此仓库？文件和数据仍保留在本地。',
    pathUnavailable: '路径不可用',
    templateSoon: '从模板快速开始 · 即将推出',
    selectFolder: '选择文件夹',
    notAValidRepo: '所选目录不包含 Index 仓库',
    items: ' 项',
  },
```

- [ ] **Step 2: Add en dashboard strings**

In `src/locales/en.ts`, insert after the `emptyState` block (line 67):

```ts
  dashboard: {
    title: 'Index',
    tagline: 'Local-first · Plugin-extensible · Object Manager',
    createOrImport: 'Create or Import Repo',
    newRepo: 'New Repo',
    importRepo: 'Import Existing Repo',
    removeRepo: 'Remove from list',
    confirmRemove: 'Remove this repo from the list? Files and data remain on disk.',
    pathUnavailable: 'Path unavailable',
    templateSoon: 'Start from a template · Coming soon',
    selectFolder: 'Select Folder',
    notAValidRepo: 'Selected folder is not an Index repository',
    items: ' items',
  },
```

- [ ] **Step 3: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/locales/zh-CN.ts src/locales/en.ts
git commit -m "feat: add dashboard i18n strings (zh-CN + en)"
```

---

### Task 7: Pinia store — useDashboardStore

**Files:**
- Create: `src/stores/dashboard.ts`

**Interfaces:**
- Consumes: `ManagedRepo` from `@/types/bindings`, `invoke` from `@tauri-apps/api/core`
- Produces: `useDashboardStore` with `repos`, `loading`, `fetchAll()`, `addRepo(path, icon?, itemCount?)`, `removeRepo(path)`, `updateIcon(path, icon)`

- [ ] **Step 1: Create dashboard store**

Create `src/stores/dashboard.ts`:

```ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ManagedRepo } from '@/types/bindings'

export const useDashboardStore = defineStore('dashboard', () => {
  const repos = ref<ManagedRepo[]>([])
  const loading = ref(false)

  async function fetchAll() {
    loading.value = true
    try {
      repos.value = await invoke<ManagedRepo[]>('list_managed_repos')
    } catch {
      repos.value = []
    } finally {
      loading.value = false
    }
  }

  async function addRepo(path: string, icon?: string, itemCount?: number) {
    repos.value = await invoke<ManagedRepo[]>('add_managed_repo', {
      path,
      icon: icon ?? null,
      name: null,
      itemCount: itemCount ?? null,
    })
  }

  async function removeRepo(path: string) {
    repos.value = await invoke<ManagedRepo[]>('remove_managed_repo', { path })
  }

  async function updateIcon(path: string, icon: string) {
    repos.value = await invoke<ManagedRepo[]>('update_repo_icon', { path, icon })
  }

  return { repos, loading, fetchAll, addRepo, removeRepo, updateIcon }
})
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/stores/dashboard.ts
git commit -m "feat: add useDashboardStore Pinia store"
```

---

### Task 8: RepoCard component

**Files:**
- Create: `src/components/RepoCard.vue`

**Interfaces:**
- Props: `repo: ManagedRepo`
- Emits: `open` (click card), `delete` (menu → confirm delete)
- Consumes: `useI18n` for localized strings

- [ ] **Step 1: Create RepoCard.vue**

Create `src/components/RepoCard.vue`:

```vue
<template>
  <div class="repo-card" @click="$emit('open')" :title="repo.path">
    <div class="card-icon">{{ displayIcon }}</div>
    <div class="card-name truncate">{{ displayName }}</div>
    <div class="card-meta">
      <span class="meta-item" v-if="repo.item_count != null">
        <TablerIcon name="file" :size="11" />
        {{ repo.item_count }}{{ $t('dashboard.items') }}
      </span>
      <span class="meta-item">
        <TablerIcon name="clock" :size="11" />
        {{ timeAgo }}
      </span>
    </div>
    <button class="card-menu-btn" @click.stop="showMenu = !showMenu" :title="$t('dashboard.removeRepo')">
      <TablerIcon name="dots" :size="14" />
    </button>
    <div v-if="showMenu" class="card-menu" @click.stop>
      <button class="danger" @click="doDelete">{{ $t('dashboard.removeRepo') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ManagedRepo } from '@/types/bindings'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()

const props = defineProps<{ repo: ManagedRepo }>()
const emit = defineEmits<{ open: []; delete: [] }>()

const showMenu = ref(false)

const displayIcon = computed(() => props.repo.icon || '📁')
const displayName = computed(() => props.repo.name || basename(props.repo.path))

function basename(p: string): string {
  return p.split(/[/\\]/).pop() || p
}

const timeAgo = computed(() => {
  try {
    const then = new Date(props.repo.last_opened_at).getTime()
    const diff = Date.now() - then
    const mins = Math.floor(diff / 60000)
    if (mins < 1) return t('centerList.justNow')
    if (mins < 60) return `${mins}${t('centerList.minAgo')}`
    const hours = Math.floor(mins / 60)
    if (hours < 24) return `${hours}${t('centerList.hourAgo')}`
    const days = Math.floor(hours / 24)
    if (days === 1) return t('centerList.yesterday')
    if (days < 30) return `${days}${t('centerList.dayAgo')}`
    return `${Math.floor(days / 30)}${t('centerList.monthAgo')}`
  } catch {
    return ''
  }
})

function doDelete() {
  if (confirm(t('dashboard.confirmRemove'))) {
    showMenu.value = false
    emit('delete')
  }
}
</script>

<style scoped>
.repo-card {
  width: 180px; height: 140px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 4px;
  padding: 16px 12px;
  cursor: pointer;
  position: relative;
  transition: border-color var(--slow) var(--ease), transform var(--slow) var(--ease), box-shadow var(--slow) var(--ease);
  user-select: none;
}
.repo-card:hover {
  border-color: var(--border-strong);
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}
.card-icon { font-size: 40px; line-height: 1; margin-bottom: 2px; }
.card-name {
  font-weight: var(--fw-semibold);
  font-size: var(--fs-sm);
  color: var(--text-heading);
  max-width: 100%;
  text-align: center;
}
.card-meta {
  display: flex; gap: 10px;
  font-size: var(--fs-xs); color: var(--text-secondary);
  margin-top: 2px;
}
.meta-item {
  display: inline-flex; align-items: center; gap: 3px;
}
.card-menu-btn {
  position: absolute; top: 6px; right: 6px;
  width: 26px; height: 26px; padding: 0;
  border: none; background: transparent;
  color: var(--text-muted); border-radius: var(--r-sm);
  cursor: pointer;
  display: none;
  align-items: center; justify-content: center;
}
.repo-card:hover .card-menu-btn { display: flex; }
.card-menu-btn:hover { background: var(--surface-hover); color: var(--text); }
.card-menu {
  position: absolute; top: 34px; right: 6px;
  background: var(--bg);
  border: 1px solid var(--border-strong);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-md);
  padding: 4px;
  z-index: 10;
  min-width: 120px;
}
.card-menu button {
  display: block; width: 100%;
  text-align: left; padding: 6px 10px;
  border: none; background: transparent;
  font-size: var(--fs-sm); cursor: pointer;
  color: var(--text);
  border-radius: var(--r-sm);
  height: auto;
}
.card-menu button:hover { background: var(--surface-hover); }
.card-menu button.danger { color: var(--danger); }
.card-menu button.danger:hover { background: var(--danger-subtle); }
</style>
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/RepoCard.vue
git commit -m "feat: add RepoCard component"
```

---

### Task 9: RepoCreateTile component

**Files:**
- Create: `src/components/RepoCreateTile.vue`

**Interfaces:**
- Props: none
- Emits: `create` (start new repo flow), `import` (import existing repo)

- [ ] **Step 1: Create RepoCreateTile.vue**

Create `src/components/RepoCreateTile.vue`:

```vue
<template>
  <div class="create-tile" @click="showMenu = !showMenu">
    <div class="create-plus">+</div>
    <div class="create-label">{{ $t('dashboard.createOrImport') }}</div>
    <div v-if="showMenu" class="create-menu" @click.stop>
      <button @click="showMenu = false; $emit('create')">
        <TablerIcon name="plus" :size="15" /> {{ $t('dashboard.newRepo') }}
      </button>
      <button @click="showMenu = false; $emit('import')">
        <TablerIcon name="folder-open" :size="15" /> {{ $t('dashboard.importRepo') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TablerIcon from './TablerIcon.vue'

defineEmits<{ create: []; import: [] }>()
const showMenu = ref(false)
</script>

<style scoped>
.create-tile {
  width: 180px; height: 140px;
  border: 2px dashed var(--border-strong);
  border-radius: var(--r-lg);
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: 6px;
  cursor: pointer;
  position: relative;
  transition: border-color var(--fast) var(--ease), background var(--fast) var(--ease);
  user-select: none;
}
.create-tile:hover {
  border-color: var(--accent);
  background: var(--accent-subtle);
}
.create-plus {
  font-size: 32px; font-weight: 300;
  color: var(--text-muted); line-height: 1;
  transition: color var(--fast) var(--ease);
}
.create-tile:hover .create-plus { color: var(--accent); }
.create-label {
  font-size: var(--fs-sm);
  color: var(--text-secondary);
}
.create-menu {
  position: absolute; top: 100%; left: 50%;
  transform: translateX(-50%);
  margin-top: 8px;
  background: var(--bg);
  border: 1px solid var(--border-strong);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-md);
  padding: 4px;
  z-index: 10;
  min-width: 180px;
}
.create-menu button {
  display: flex; align-items: center; gap: 8px;
  width: 100%; text-align: left;
  padding: 8px 12px;
  border: none; background: transparent;
  font-size: var(--fs-sm); cursor: pointer;
  color: var(--text);
  border-radius: var(--r-sm);
  height: auto;
}
.create-menu button:hover { background: var(--surface-hover); }
</style>
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/RepoCreateTile.vue
git commit -m "feat: add RepoCreateTile component"
```

---

### Task 10: TemplateBanner component

**Files:**
- Create: `src/components/TemplateBanner.vue`

**Interfaces:**
- Props: none
- Emits: none (purely visual placeholder)

- [ ] **Step 1: Create TemplateBanner.vue**

Create `src/components/TemplateBanner.vue`:

```vue
<template>
  <div class="template-banner">
    💡 {{ $t('dashboard.templateSoon') }}
  </div>
</template>

<style scoped>
.template-banner {
  text-align: center;
  padding: 8px 16px;
  font-size: var(--fs-xs);
  color: var(--text-muted);
  background: var(--surface);
  border-radius: var(--r-full);
  display: inline-block;
}
</style>
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/TemplateBanner.vue
git commit -m "feat: add TemplateBanner placeholder component"
```

---

### Task 11: RepoDashboard component (main layout)

**Files:**
- Create: `src/components/RepoDashboard.vue`

**Interfaces:**
- Consumes: `useDashboardStore`, `useRepoStore`, `useI18n`, `@tauri-apps/plugin-dialog`
- Emits: `repoOpened` (same signature as current EmptyState)
- Produces: Full dashboard layout composing RepoCard, RepoCreateTile, TemplateBanner

- [ ] **Step 1: Create RepoDashboard.vue**

Create `src/components/RepoDashboard.vue`:

```vue
<template>
  <div class="dashboard">
    <div class="dash-content">
      <div class="dash-header">
        <TablerIcon name="database" :size="36" :stroke="1.5" class="dash-logo" />
        <h1>{{ $t('dashboard.title') }}</h1>
        <p>{{ $t('dashboard.tagline') }}</p>
      </div>

      <div class="dash-grid" v-if="!loading">
        <RepoCard
          v-for="repo in store.repos"
          :key="repo.path"
          :repo="repo"
          @open="openRepo(repo)"
          @delete="removeRepo(repo)"
        />
        <RepoCreateTile
          @create="doCreate"
          @import="doImport"
        />
      </div>
      <div class="dash-loading" v-else>
        <p class="text-muted">...</p>
      </div>

      <TemplateBanner />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { useDashboardStore } from '@/stores/dashboard'
import { useRepoStore } from '@/stores/repo'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import { useSettingsStore } from '@/stores/settings'
import type { ManagedRepo } from '@/types/bindings'
import RepoCard from './RepoCard.vue'
import RepoCreateTile from './RepoCreateTile.vue'
import TemplateBanner from './TemplateBanner.vue'
import TablerIcon from './TablerIcon.vue'

const { t } = useI18n()
const store = useDashboardStore()
const repoStore = useRepoStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()
const settingsStore = useSettingsStore()

const { repos, loading } = store
const emit = defineEmits<{ repoOpened: [] }>()

onMounted(() => {
  store.fetchAll()
})

async function openRepo(repo: ManagedRepo) {
  try {
    await repoStore.openRepo(repo.path)
    // Update last_opened_at and item_count in background
    await store.addRepo(repo.path, repo.icon ?? undefined)
    await loadStores()
    emit('repoOpened')
  } catch {
    alert(t('emptyState.openFailed'))
  }
}

async function removeRepo(repo: ManagedRepo) {
  await store.removeRepo(repo.path)
}

async function doCreate() {
  const selected = await open({ directory: true, multiple: false, title: t('dashboard.selectFolder') })
  if (!selected) return
  const p = typeof selected === 'string' ? selected : selected.path
  try {
    await repoStore.createRepo(p)
    await store.addRepo(p)
    await loadStores()
    emit('repoOpened')
  } catch {
    alert(t('emptyState.createFailed'))
  }
}

async function doImport() {
  const selected = await open({ directory: true, multiple: false, title: t('dashboard.selectFolder') })
  if (!selected) return
  const p = typeof selected === 'string' ? selected : selected.path
  // Verify it has .index/index.db
  try {
    await repoStore.openRepo(p)
    // It opened successfully — close it and add to managed list
    await repoStore.closeRepo()
    await store.addRepo(p)
  } catch {
    alert(t('dashboard.notAValidRepo'))
  }
}

async function loadStores() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
    itemStore.fetchList(),
  ])
  await settingsStore.loadActivePresetFromRepo()
  settingsStore.applyTheme()
}
</script>

<style scoped>
.dashboard {
  display: flex; align-items: center; justify-content: center;
  height: 100vh; background: var(--bg);
  user-select: none;
}
.dash-content {
  display: flex; flex-direction: column; align-items: center;
  gap: 32px;
  padding: 48px 32px;
  max-width: 880px;
  width: 100%;
}
.dash-header {
  text-align: center;
}
.dash-logo {
  color: var(--accent); margin-bottom: 8px;
}
.dash-header h1 {
  font-size: var(--fs-2xl); font-weight: var(--fw-bold);
  margin: 0 0 4px; color: var(--text-heading);
}
.dash-header p {
  font-size: var(--fs-base); color: var(--text-secondary);
  margin: 0;
}
.dash-grid {
  display: flex; flex-wrap: wrap; gap: 16px;
  justify-content: center;
}
.dash-loading {
  display: flex; align-items: center; justify-content: center;
  min-height: 172px;
}
</style>
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/RepoDashboard.vue
git commit -m "feat: add RepoDashboard main layout component"
```

---

### Task 12: Update App.vue — use RepoDashboard

**Files:**
- Modify: `src/App.vue`

**Interfaces:**
- Consumes: `RepoDashboard` component
- Produces: Dashboard shown when no repo open, replaces old EmptyState

- [ ] **Step 1: Replace EmptyState with RepoDashboard**

In `src/App.vue`, change the import and template usage.

Change the import line:
```ts
import EmptyState from '@/components/EmptyState.vue'
```
to:
```ts
import RepoDashboard from '@/components/RepoDashboard.vue'
```

Change the template line:
```html
<EmptyState v-if="!repoStore.isOpen" @repo-opened="onRepoOpened" />
```
to:
```html
<RepoDashboard v-if="!repoStore.isOpen" @repo-opened="onRepoOpened" />
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: replace EmptyState with RepoDashboard in App.vue"
```

---

### Task 13: Update Titlebar — category button → repo button

**Files:**
- Modify: `src/components/Titlebar.vue`

**Interfaces:**
- Consumes: `WebviewWindow` from `@tauri-apps/api/webviewWindow`
- Produces: New "仓库" button that opens a dashboard window
- Removes: `openTypeManager` emit

- [ ] **Step 1: Replace category button with repo button**

In `src/components/Titlebar.vue`, make the following changes:

**Template** — replace:
```html
<button class="icon-btn tb-icon" @click.stop="$emit('openTypeManager')" :title="$t('common.category')">
  <TablerIcon name="category" :size="17" />
</button>
```
with:
```html
<button class="icon-btn tb-icon" @click.stop="openDashboard" :title="$t('topbar.manageRepos')">
  <TablerIcon name="database" :size="17" />
</button>
```

**Script** — change the emit type from:
```ts
defineEmits<{ newItem: []; openTypeManager: [] }>()
```
to:
```ts
defineEmits<{ newItem: [] }>()
```

**Script** — add the `openDashboard` function and import. Add after `const openSettings = ...` line:

```ts
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

function openDashboard() {
  new WebviewWindow('dashboard-' + Date.now(), {
    url: '/',
    title: 'Index — 仓库管理',
    width: 900,
    height: 640,
    resizable: true,
    decorations: false,
  })
}
```

**Note:** The `WebviewWindow` import should be placed at the top with other imports.

- [ ] **Step 2: Add i18n key for the repo button tooltip**

In `src/locales/zh-CN.ts`, add to the `topbar` block (line 2):
```ts
manageRepos: '仓库管理',
```
Full `topbar` block becomes:
```ts
topbar: { newItem: '新建条目', manageRepos: '仓库管理' },
```

In `src/locales/en.ts`, add to the `topbar` block (line 2):
```ts
manageRepos: 'Manage Repos',
```

- [ ] **Step 3: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/components/Titlebar.vue src/locales/zh-CN.ts src/locales/en.ts
git commit -m "feat: replace category button with repo dashboard button in Titlebar"
```

---

### Task 14: Update repo store — auto-add on open/create

**Files:**
- Modify: `src/stores/repo.ts`

**Interfaces:**
- Consumes: `useDashboardStore`
- Produces: `openRepo()` and `createRepo()` automatically record in dashboard

- [ ] **Step 1: Auto-add repo on open**

In `src/stores/repo.ts`, add import:
```ts
import { useDashboardStore } from './dashboard'
```

Modify `openRepo` function:
```ts
async function openRepo(path: string): Promise<RepoInfo> {
  const info = await invoke<RepoInfo>('open_repo', { path })
  repoPath.value = info.path
  itemCount.value = info.item_count
  // Auto-record in dashboard with current item count
  try { await useDashboardStore().addRepo(info.path, undefined, info.item_count) } catch { /* ignore */ }
  return info
}
```

Modify `createRepo` function:
```ts
async function createRepo(path: string): Promise<RepoInfo> {
  const info = await invoke<RepoInfo>('create_repo', { path })
  repoPath.value = info.path
  itemCount.value = info.item_count
  // Auto-record in dashboard
  try { await useDashboardStore().addRepo(info.path) } catch { /* ignore */ }
  return info
}
```

- [ ] **Step 2: Type-check**

Run:
```bash
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/stores/repo.ts
git commit -m "feat: auto-record repo in dashboard on open/create"
```

---

### Task 15: Integration verification

- [ ] **Step 1: Build and check for errors**

Run:
```bash
cd src-tauri && cargo check
```

Expected: PASS.

Run:
```bash
npx vue-tsc --noEmit
```

Expected: PASS. No type errors.

- [ ] **Step 2: Full build check**

Run:
```bash
pnpm build
```

Expected: PASS.

- [ ] **Step 3: Test manual flow (if dev available)**

Run:
```bash
pnpm dev
```

In the dev app:
1. Dashboard shows with create tile + template banner (no repos yet)
2. Click create tile → "新建仓库" → native folder dialog → creates repo → enters main view
3. Close and reopen app → dashboard shows the created repo card
4. Click card → opens repo
5. Inside repo, click repo button in titlebar → new dashboard window opens
6. In new dashboard window, delete the repo card → confirm dialog → removed from list
7. Verify the actual repo folder and `.index/` still exist on disk

- [ ] **Step 4: Commit final verification**

```bash
git add -A
git diff --cached --stat
git commit -m "chore: final integration verification of repo dashboard"
```

---

## Execution Notes

- **Task order matters**: Tasks 1-7 build the data layer (Rust → TypeScript → i18n → store). Tasks 8-10 build leaf components. Task 11 composes them. Tasks 12-14 wire everything into the app. Task 15 verifies.
- **EmptyState.vue**: Not deleted — kept as an unreferenced file in case of rollback. Can be removed in a follow-up cleanup if desired.
- **Multi-window**: The new dashboard window loads the same `index.html`. Since `repoStore.isOpen` is `false` in a fresh session, it shows `RepoDashboard` automatically.
- **`app_data_dir`**: Platform-resolved by Tauri. On Windows: `%APPDATA%/com.index.app/`. On macOS: `~/Library/Application Support/com.index.app/`. On Linux: `~/.local/share/com.index.app/`.
