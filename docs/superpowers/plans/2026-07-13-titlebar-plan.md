# Custom Titlebar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace native titlebar with a 40px custom HTML titlebar integrating Topbar functionality, using CSS tokens for theme adaptation.

**Architecture:** `Titlebar.vue` combines the old Topbar buttons with Tauri window controls (minimize/maximize/close). Tauri config `decorations: false` removes the native frame. `App.vue` swaps `<Topbar>` for `<Titlebar>`.

**Tech Stack:** Vue 3 + TypeScript, @tauri-apps/api/window, CSS custom properties

## Global Constraints

- Titlebar height: 40px, background `var(--surface)`, bottom border `var(--border)`
- Entire bar is a drag region (`data-tauri-drag-region`)
- Colors must use CSS tokens ONLY — no hardcoded hex except close button hover (#C42B1C)
- Window API: `getCurrentWindow().minimize()`, `.toggleMaximize()`, `.close()`
- Tauri config: `"decorations": false` in `tauri.conf.json → app.windows[0]`
- All existing Topbar functionality preserved: new item button, settings gear, category button

---

### Task 1: Tauri config + create Titlebar.vue

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src/components/Titlebar.vue`

- [ ] **Step 1: Set decorations: false in Tauri config**

In `src-tauri/tauri.conf.json`, add to the window config (after `"dragDropEnabled": false`):

```json
"decorations": false
```

- [ ] **Step 2: Create Titlebar.vue**

Create `src/components/Titlebar.vue`:

```vue
<template>
  <header class="titlebar" data-tauri-drag-region>
    <!-- Left: Logo + repo name -->
    <div class="tb-left">
      <TablerIcon name="database" :size="18" :stroke="1.5" />
      <span class="tb-repo">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Index' }}</span>
    </div>

    <!-- Center spacer (drag region) -->
    <div class="tb-spacer" />

    <!-- Right: action buttons + window controls -->
    <div class="tb-right">
      <button class="primary sm" @click.stop="$emit('newItem')">
        <TablerIcon name="plus" :size="15" /> {{ $t('topbar.newItem') }}
      </button>
      <button class="icon-btn tb-icon" @click.stop="openSettings" :title="$t('common.settings')">
        <TablerIcon name="settings" :size="17" />
      </button>
      <button class="icon-btn tb-icon" @click.stop="$emit('openTypeManager')" :title="$t('common.category')">
        <TablerIcon name="category" :size="17" />
      </button>

      <span class="tb-sep" />

      <!-- Window controls -->
      <button class="icon-btn tb-ctrl" @click.stop="winMinimize" title="Minimize">
        <TablerIcon name="minus" :size="16" />
      </button>
      <button class="icon-btn tb-ctrl" @click.stop="winToggleMaximize" title="Maximize">
        <TablerIcon name="square" :size="14" />
      </button>
      <button class="icon-btn tb-ctrl tb-close" @click.stop="winClose" title="Close">
        <TablerIcon name="x" :size="17" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { getCurrentWindow } from '@tauri-apps/api/window'
import TablerIcon from './TablerIcon.vue'

const repoStore = useRepoStore()
const settingsRef = ref<InstanceType<typeof import('./SettingsModal.vue')['default']> | null>(null)

const emit = defineEmits<{ newItem: []; openTypeManager: [] }>()

function basename(p: string): string { return p.split(/[/\\]/).pop() || p }

const win = getCurrentWindow()

function winMinimize() { win.minimize() }
function winToggleMaximize() { win.toggleMaximize() }
function winClose() { win.close() }

// SettingsModal is mounted in App.vue — we reach it via a provide/inject or by passing a ref.
// For now, emit an event that App.vue handles.
function openSettings() {
  // App.vue provides a way to open settings — we emit a custom event
  const event = new CustomEvent('open-settings')
  window.dispatchEvent(event)
}

defineExpose({ settingsRef })
</script>

<style scoped>
.titlebar {
  height: 40px; flex-shrink: 0;
  display: flex; align-items: center;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  user-select: none; -webkit-app-region: drag;
}

.tb-left {
  display: flex; align-items: center; gap: 8px;
  padding-left: 12px;
  color: var(--accent);
  min-width: 0; -webkit-app-region: no-drag;
}
.tb-repo {
  font-weight: 620; font-size: var(--fs-sm);
  color: var(--text);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

.tb-spacer { flex: 1; }

.tb-right {
  display: flex; align-items: center; gap: 2px;
  padding-right: 6px;
  -webkit-app-region: no-drag;
}

.tb-icon {
  width: 30px; height: 30px; color: var(--text-secondary);
}
.tb-icon:hover { color: var(--text); background: var(--surface-hover); }

.sm { font-size: var(--fs-xs); height: 28px; padding: 4px 10px; }

.tb-sep {
  width: 1px; height: 20px;
  background: var(--border-strong);
  margin: 0 6px;
}

/* Window control buttons */
.tb-ctrl {
  width: 34px; height: 28px;
  color: var(--text-secondary);
  border-radius: var(--r-sm);
}
.tb-ctrl:hover { background: var(--surface-hover); color: var(--text); }
.tb-close:hover { background: #C42B1C; color: #fff; }
</style>
```

- [ ] **Step 3: Handle SettingsModal access**

The issue: SettingsModal is mounted in App.vue and Titlebar needs to call `settingsRef?.open()`. The cleanest approach is a `provide/inject` pattern or a simple custom event.

Since we're already refactoring App.vue in Task 2, use `provide`:

Titlebar uses `inject`:
```typescript
import { inject } from 'vue'
const openSettings = inject<() => void>('openSettings', () => {})
```

Then replace the `openSettings` function with the injected one.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json src/components/Titlebar.vue
git commit -m "feat: add custom Titlebar with frameless window support"
```

---

### Task 2: Update App.vue — Topbar → Titlebar + provide settings

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Replace Topbar with Titlebar in App.vue template**

```vue
<!-- Replace <Topbar @new-item="showNewItem = true" @open-type-manager="rightTab = 'types'" /> -->
<Titlebar @new-item="showNewItem = true" @open-type-manager="rightTab = 'types'" />
```

- [ ] **Step 2: Add provide for openSettings**

In App.vue `<script setup>`:
```typescript
import { provide } from 'vue'
import Titlebar from '@/components/Titlebar.vue'
// Remove Topbar import

const settingsRef = ref<InstanceType<typeof import('@/components/SettingsModal.vue')['default']> | null>(null)
provide('openSettings', () => settingsRef.value?.open())
```

The SettingsModal already has a `ref` — we just need to add it. Actually, looking at the current App.vue, SettingsModal is not mounted. It was added in Task 6 of the theme plan via Topbar. Since we're replacing Topbar, we need to mount SettingsModal in App.vue now.

Add to App.vue template (after `</template>` main content):
```vue
<SettingsModal ref="settingsRef" />
```

- [ ] **Step 3: Update imports**

```typescript
// Remove
// import Topbar from '@/components/Topbar.vue'
// Add
import Titlebar from '@/components/Titlebar.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import { provide, ref, onMounted } from 'vue'
```

- [ ] **Step 4: Run type-check**

```bash
pnpm vue-tsc --noEmit
```

- [ ] **Step 5: Commit**

```bash
git add src/App.vue
git commit -m "feat: replace Topbar with Titlebar, provide settings access, mount SettingsModal"
```

---

### Verification Checklist

- [ ] `pnpm vue-tsc --noEmit` — no errors
- [ ] `cargo build` — no errors
- [ ] `pnpm tauri dev` — window opens without native titlebar
- [ ] Drag the custom titlebar area → window moves
- [ ] Minimize button → window minimizes
- [ ] Maximize button → window maximizes/restores
- [ ] Close button → window closes
- [ ] Close button hover → red background
- [ ] "新建条目" button works
- [ ] Gear icon opens SettingsModal
- [ ] Category icon switches to TypeManager
- [ ] Dark mode → titlebar colors adapt
- [ ] Repo name displays correctly
