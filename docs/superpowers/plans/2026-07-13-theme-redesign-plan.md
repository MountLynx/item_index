# Theme Redesign — "Quiet Strength" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade visual tokens to "Quiet Strength" spec, add SettingsModal with theme presets, tighten CenterList row height.

**Architecture:** CSS token values updated in-place (no renaming). New `settingsStore` Pinia store manages localStorage + state.json persistence. SettingsModal is a single modal component with left tab nav + right content area. Rust backend gains `get_state`/`save_state` commands for reading/writing `.index/state.json`.

**Tech Stack:** Vue 3 + TypeScript, Pinia, Tauri 2, CSS custom properties

## Global Constraints

- CSS variable names MUST NOT be renamed — only values change
- All existing components' `var(--xxx)` references stay untouched
- `.index/state.json` gets one new optional field `activePresetId` — old repos without it default to `null`
- localStorage key `"index-settings"` must be a single JSON object with top-level sections (e.g. `theme`, future `ai`, `general`)
- Token value changes per the spec tables in docs/superpowers/specs/2026-07-13-theme-redesign-design.md §2.2–§2.11
- SettingsModal dimensions: 720×520px, left tab bar 160px, right content 560px
- No database schema changes

---

### Task 1: Update theme.css tokens to "Quiet Strength"

**Files:**
- Modify: `src/assets/theme.css`

**Interfaces:**
- Produces: Updated CSS custom property values consumed by all components via `var(--xxx)` references

- [ ] **Step 1: Replace `:root` color slot values (lines 28-48)**

Rewrite `src/assets/theme.css` lines 27-48. Replace the entire `:root` block's "Slots — Light" section:

```css
/* Slots — Light */
--bg:             #FFFFFF;
--surface:        #FAFAFA;
--surface-hover:  #F0F0F0;
--surface-active: #E8E8E8;

--text:           #555555;
--text-secondary: #999999;
--text-muted:     #C7C5C5;
--text-heading:   #333333;

--border:         #EEEEEE;
--border-light:   #F3F3F3;
--border-strong:  #DDDDDD;

--accent:         #1A1C1E;
--accent-hover:   #333333;
--accent-subtle:  rgba(26,28,30,0.10);
--accent-fg:      #FFFFFF;

--danger:         #B42318;
--danger-hover:   #8B1A10;
--danger-subtle:  rgba(180,35,24,0.08);

--link:           #2F56C6;

--success:        #22C55E;
--warning:        #F59E0B;
```

- [ ] **Step 2: Replace shadow values (lines 52-56)**

Replace the shadow block:

```css
/* Shadows — only on floating/overlay surfaces */
--shadow-sm: 0 1px 3px rgba(0,0,0,0.06);
--shadow-md: 0 18px 52px rgba(0,0,0,0.12);
--shadow-lg: 0 20px 64px rgba(0,0,0,0.16);
```

- [ ] **Step 3: Replace font weights (lines 77-78)**

Replace the `--fw-*` lines:

```css
--fw-normal:   400;
--fw-tertiary: 520;
--fw-semibold: 560;
--fw-medium:   620;
--fw-emphasis: 650;
--fw-bold:     760;
```

- [ ] **Step 4: Replace font stack (line 65)**

Replace the `--font` line:

```css
--font: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
```

- [ ] **Step 5: Add `--transition-fast` (after `--ease`, line 90)**

After line 90 (`--ease: cubic-bezier(...);`), add:

```css
--transition-fast: 150ms ease-out;
```

- [ ] **Step 6: Replace scrollbar styles (lines 161-164)**

Replace the scrollbar block:

```css
::-webkit-scrollbar { width: 10px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  border: 2px solid transparent;
  background-clip: content-box;
  border-radius: 999px;
  background-color: rgba(153,153,153,0.44);
}
::-webkit-scrollbar-thumb:hover { background-color: rgba(120,120,120,0.50); }
::-webkit-scrollbar-thumb:active { background-color: var(--accent); }
.dark ::-webkit-scrollbar-thumb { background-color: rgba(120,120,120,0.44); }
.dark ::-webkit-scrollbar-thumb:hover { background-color: rgba(160,160,160,0.50); }
```

- [ ] **Step 7: Replace `.dark` section entirely (lines 93-115)**

Replace the entire `.dark` block:

```css
/* Dark */
.dark {
  --bg:             #1E1E1E;
  --surface:        #252525;
  --surface-hover:  #2E2E2E;
  --surface-active: #353535;
  --text:           #E1E1E1;
  --text-heading:   #F4F4F5;
  --text-secondary: #999999;
  --text-muted:     #555555;
  --border:         #333333;
  --border-light:   #2A2A2A;
  --border-strong:  #444444;
  --accent:         #F4F4F5;
  --accent-hover:   #CCCCCC;
  --accent-subtle:  rgba(244,244,245,0.10);
  --accent-fg:      #1E1E1E;
  --danger:         #DA4A3F;
  --danger-hover:   #C0392B;
  --danger-subtle:  rgba(218,74,63,0.12);
  --link:           #6B8FE8;
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.20);
  --shadow-md: 0 18px 52px rgba(0,0,0,0.40), 0 0 0 1px rgba(255,255,255,0.035);
  --shadow-lg: 0 20px 64px rgba(0,0,0,0.50), 0 0 0 1px rgba(255,255,255,0.035);
}
```

- [ ] **Step 8: Run dev server to verify visual changes**

```bash
pnpm dev
```

Expected: App launches without CSS errors. Colors are warmer/more muted (gray text, ink-black accent). Dark mode has deep charcoal background.

- [ ] **Step 9: Commit**

```bash
git add src/assets/theme.css
git commit -m "style: apply Quiet Strength token values to theme.css"
```

---

### Task 2: Add get_state / save_state Rust commands for state.json

**Files:**
- Modify: `src-tauri/src/commands/repo.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `get_state() -> Result<serde_json::Value, String>` and `save_state(state: serde_json::Value) -> Result<(), String>` IPC commands
- Consumed by: `src/stores/settings.ts` (Task 3) via `invoke('get_state')` and `invoke('save_state', { state })`

- [ ] **Step 1: Add `get_state` command to repo.rs**

Append to `src-tauri/src/commands/repo.rs`:

```rust
#[tauri::command]
pub async fn get_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let repo_path = get_repo_path(&state)?;
    let state_path = Path::new(&repo_path).join(".index").join("state.json");

    if !state_path.exists() {
        return Ok(serde_json::json!({"theme": "light"}));
    }

    let content = std::fs::read_to_string(&state_path)
        .map_err(|e| format!("Read error: {}", e))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(value)
}

#[tauri::command]
pub async fn save_state(state: State<'_, AppState>, new_state: serde_json::Value) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;
    let state_path = Path::new(&repo_path).join(".index").join("state.json");

    // Read existing, merge to preserve unknown keys
    let mut current: serde_json::Value = if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)
            .map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Parse error: {}", e))?
    } else {
        serde_json::json!({})
    };

    // Merge: new_state keys overwrite current
    if let (serde_json::Value::Object(ref mut cur_map), serde_json::Value::Object(new_map)) = (&mut current, &new_state) {
        for (k, v) in new_map {
            cur_map.insert(k.clone(), v.clone());
        }
    }

    let content = serde_json::to_string_pretty(&current)
        .map_err(|e| format!("Serialize error: {}", e))?;
    std::fs::write(&state_path, content)
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(())
}
```

- [ ] **Step 2: Register new commands in lib.rs**

Add the two new commands to the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::repo::get_state,
    commands::repo::save_state,
])
```

Insert them in alphabetical position within the `commands::repo::` group (after `get_repo_info`, before `open_repo`).

- [ ] **Step 3: Build Rust side to verify compilation**

```bash
cd src-tauri && cargo build
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/repo.rs src-tauri/src/lib.rs
git commit -m "feat: add get_state/save_state commands for state.json persistence"
```

---

### Task 3: Create SettingsStore (Pinia)

**Files:**
- Create: `src/stores/settings.ts`

**Interfaces:**
- Produces:
  - `useSettingsStore()` — Pinia store
  - `load(): void` — load from localStorage
  - `save(): void` — persist to localStorage
  - `applyTheme(): void` — inject CSS `<style>` tags
  - `createPreset(name: string, css: string): string` — return new preset id
  - `deletePreset(id: string): void`
  - `setActivePreset(id: string | null): Promise<void>` — also writes to state.json
  - State: `themeMode`, `accentColor`, `fontSize`, `presets`, `activePresetId`, `presetCSS`
- Consumes: `invoke('get_state')`, `invoke('save_state', { state })` from Task 2

- [ ] **Step 1: Create the settings store**

Create `src/stores/settings.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ThemePreset {
  id: string
  name: string
  css: string
  createdAt: string
  updatedAt: string
}

interface ThemeSection {
  mode: 'light' | 'dark'
  accentColor: string
  fontSize: 'small' | 'medium' | 'large'
  presets: ThemePreset[]
}

interface GlobalSettings {
  theme: ThemeSection
}

const STORAGE_KEY = 'index-settings'

function defaultSettings(): GlobalSettings {
  return {
    theme: {
      mode: 'light',
      accentColor: '#1A1C1E',
      fontSize: 'medium',
      presets: [],
    },
  }
}

function loadFromStorage(): GlobalSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaultSettings()
    const parsed = JSON.parse(raw) as GlobalSettings
    // Ensure nested defaults exist
    return {
      theme: {
        mode: parsed.theme?.mode ?? 'light',
        accentColor: parsed.theme?.accentColor ?? '#1A1C1E',
        fontSize: parsed.theme?.fontSize ?? 'medium',
        presets: Array.isArray(parsed.theme?.presets) ? parsed.theme.presets : [],
      },
    }
  } catch {
    return defaultSettings()
  }
}

function saveToStorage(settings: GlobalSettings): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}

function generateId(): string {
  return Array.from(crypto.getRandomValues(new Uint8Array(8)))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('')
}

const FONT_SIZE_MAP: Record<string, string> = {
  small: '0.75rem',   // 12px
  medium: '0.875rem', // 14px
  large: '0.9375rem', // 15px
}

export const useSettingsStore = defineStore('settings', () => {
  // ── State ──
  const themeMode = ref<'light' | 'dark'>('light')
  const accentColor = ref('#1A1C1E')
  const fontSize = ref<'small' | 'medium' | 'large'>('medium')
  const presets = ref<ThemePreset[]>([])
  const activePresetId = ref<string | null>(null)
  const presetCSS = ref('')

  // ── Actions ──
  function load(): void {
    const settings = loadFromStorage()
    themeMode.value = settings.theme.mode
    accentColor.value = settings.theme.accentColor
    fontSize.value = settings.theme.fontSize
    presets.value = settings.theme.presets

    // Load active preset CSS into local state; applyTheme() handles injection
    if (activePresetId.value) {
      const preset = presets.value.find(p => p.id === activePresetId.value)
      presetCSS.value = preset?.css ?? ''
    }
  }

  async function loadActivePresetFromRepo(): Promise<void> {
    try {
      const state = await invoke<Record<string, unknown>>('get_state')
      activePresetId.value = (state.activePresetId as string) ?? null
      if (activePresetId.value) {
        const preset = presets.value.find(p => p.id === activePresetId.value)
        presetCSS.value = preset?.css ?? ''
      }
    } catch {
      // No repo open — skip
    }
  }

  function save(): void {
    const settings: GlobalSettings = {
      theme: {
        mode: themeMode.value,
        accentColor: accentColor.value,
        fontSize: fontSize.value,
        presets: presets.value,
      },
    }
    saveToStorage(settings)
  }

  function applyTheme(): void {
    // Apply mode (.dark class) via themeStore — handled externally
    const root = document.documentElement

    // Apply accent color as CSS variable override
    root.style.setProperty('--accent', accentColor.value)

    // Apply font size
    root.style.setProperty('--fs-base', FONT_SIZE_MAP[fontSize.value])
    root.style.setProperty('--fs-sm', fontSize.value === 'small' ? '0.75rem' : '0.8125rem')

    // Inject/update theme-override style tag
    let el = document.getElementById('theme-override') as HTMLStyleElement | null
    if (!el) {
      el = document.createElement('style')
      el.id = 'theme-override'
      document.head.appendChild(el)
    }
    el.textContent = `:root { --accent: ${accentColor.value}; }`

    // Inject/update theme-preset style tag
    applyPresetCSS()
  }

  function applyPresetCSS(): void {
    let el = document.getElementById('theme-preset') as HTMLStyleElement | null
    if (presetCSS.value) {
      if (!el) {
        el = document.createElement('style')
        el.id = 'theme-preset'
        document.head.appendChild(el)
      }
      el.textContent = presetCSS.value
    } else {
      // No preset active — remove the style tag
      el?.remove()
    }
  }

  function createPreset(name: string, css: string): string {
    const id = generateId()
    const now = new Date().toISOString()
    presets.value.push({ id, name, css, createdAt: now, updatedAt: now })
    save()
    return id
  }

  function updatePreset(id: string, css: string): void {
    const p = presets.value.find(x => x.id === id)
    if (p) {
      p.css = css
      p.updatedAt = new Date().toISOString()
      save()
      if (activePresetId.value === id) {
        presetCSS.value = css
        applyPresetCSS()
      }
    }
  }

  function deletePreset(id: string): void {
    presets.value = presets.value.filter(p => p.id !== id)
    if (activePresetId.value === id) {
      setActivePreset(null)
    }
    save()
  }

  async function setActivePreset(id: string | null): Promise<void> {
    activePresetId.value = id
    if (id) {
      const preset = presets.value.find(p => p.id === id)
      presetCSS.value = preset?.css ?? ''
    } else {
      presetCSS.value = ''
    }
    applyPresetCSS()

    // Persist to state.json
    try {
      await invoke('save_state', { newState: { activePresetId: id } })
    } catch {
      // No repo open — skip
    }
  }

  return {
    themeMode, accentColor, fontSize,
    presets, activePresetId, presetCSS,
    load, loadActivePresetFromRepo, save, applyTheme,
    createPreset, updatePreset, deletePreset, setActivePreset,
    applyPresetCSS,
  }
})
```

- [ ] **Step 2: Verify TypeScript compilation**

```bash
pnpm vue-tsc --noEmit
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat: add SettingsStore — localStorage persistence, preset CRUD, CSS injection"
```

---

### Task 4: Update theme store to integrate with settingsStore

**Files:**
- Modify: `src/stores/theme.ts`

**Interfaces:**
- Consumes: `useSettingsStore()` from Task 3
- Produces: `useThemeStore()` — same interface (`mode`, `toggle()`, `apply()`) but initialized from settingsStore

- [ ] **Step 1: Rewrite theme.ts**

Replace `src/stores/theme.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { useSettingsStore } from './settings'

type ThemeMode = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>('light')

  function toggle(): void {
    mode.value = mode.value === 'light' ? 'dark' : 'light'
    const settings = useSettingsStore()
    settings.themeMode = mode.value
    settings.save()
  }

  function apply(): void {
    document.documentElement.classList.toggle('dark', mode.value === 'dark')
  }

  function init(): void {
    const settings = useSettingsStore()
    mode.value = settings.themeMode
    apply()
  }

  watch(mode, apply)

  return { mode, toggle, apply, init }
})
```

- [ ] **Step 2: Commit**

```bash
git add src/stores/theme.ts
git commit -m "feat: integrate theme store with settingsStore for persistence"
```

---

### Task 5: Create SettingsModal component

**Files:**
- Create: `src/components/SettingsModal.vue`

**Interfaces:**
- Consumes: `useSettingsStore()` from Task 3, `useThemeStore()` from Task 4
- Produces: `<SettingsModal>` component — emits `close`, manages its own `visible` state
- Exposed via `defineExpose`: `open()`, `close()`

- [ ] **Step 1: Create SettingsModal.vue**

Create `src/components/SettingsModal.vue`:

```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="settings-overlay" @click.self="onCancel">
      <div class="settings-window">
        <!-- Header -->
        <div class="settings-header">
          <h2>设置</h2>
          <button class="icon-btn" @click="onCancel">
            <TablerIcon name="x" :size="18" />
          </button>
        </div>

        <!-- Body -->
        <div class="settings-body">
          <!-- Left tabs -->
          <nav class="settings-tabs">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              class="tab-btn"
              :class="{ active: activeTab === tab.id }"
              @click="activeTab = tab.id"
            >
              <TablerIcon :name="tab.icon" :size="18" />
              <span>{{ tab.label }}</span>
            </button>
          </nav>

          <!-- Right content -->
          <div class="settings-content">
            <!-- General tab (placeholder) -->
            <div v-if="activeTab === 'general'" class="tab-panel">
              <p class="placeholder-text">更多设置即将推出</p>
            </div>

            <!-- Theme tab -->
            <div v-else-if="activeTab === 'theme'" class="tab-panel">
              <!-- Section 1: Basic settings -->
              <div class="setting-section">
                <div class="setting-row">
                  <label>模式</label>
                  <div class="radio-group">
                    <label class="radio-label">
                      <input type="radio" v-model="localMode" value="light" /> 浅色
                    </label>
                    <label class="radio-label">
                      <input type="radio" v-model="localMode" value="dark" /> 深色
                    </label>
                  </div>
                </div>
                <div class="setting-row">
                  <label>强调色</label>
                  <input type="color" v-model="localAccentColor" class="color-input" />
                </div>
                <div class="setting-row">
                  <label>字号</label>
                  <div class="btn-group">
                    <button
                      v-for="s in fontSizes"
                      :key="s"
                      :class="{ active: localFontSize === s }"
                      @click="localFontSize = s"
                    >{{ s }}</button>
                  </div>
                </div>
              </div>

              <hr class="section-divider" />

              <!-- Section 2: Preset management -->
              <div class="setting-section">
                <label class="section-title">自定义主题</label>
                <div class="preset-row">
                  <select v-model="selectedPresetId" class="preset-select">
                    <option :value="null">默认</option>
                    <option v-for="p in settingsStore.presets" :key="p.id" :value="p.id">
                      {{ p.name }}
                    </option>
                  </select>
                  <button class="sm" @click="applyPreset">应用</button>
                  <button class="sm" @click="onSaveAs">另存为...</button>
                  <button
                    v-if="selectedPresetId"
                    class="sm danger"
                    @click="onDeletePreset"
                  >删除</button>
                </div>
              </div>

              <hr class="section-divider" />

              <!-- Section 3: CSS editor -->
              <div class="setting-section">
                <label class="section-title">CSS 变量覆盖</label>
                <textarea
                  v-model="localCSS"
                  class="css-editor"
                  spellcheck="false"
                  placeholder=":root {&#10;  --accent: #1A1C1E;&#10;}"
                ></textarea>
                <button class="sm" @click="onSaveAs">保存为预设</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="settings-footer">
          <button @click="onCancel">取消</button>
          <button class="primary" @click="onSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useThemeStore } from '@/stores/theme'
import TablerIcon from './TablerIcon.vue'

const settingsStore = useSettingsStore()
const themeStore = useThemeStore()

const tabs = [
  { id: 'general', icon: 'settings', label: '通用' },
  { id: 'theme', icon: 'palette', label: '主题' },
]

const fontSizes = ['small', 'medium', 'large'] as const

const visible = ref(false)
const activeTab = ref<string>('theme')

// Local copies for live preview + cancel support
const localMode = ref<'light' | 'dark'>('light')
const localAccentColor = ref('#1A1C1E')
const localFontSize = ref<'small' | 'medium' | 'large'>('medium')
const localCSS = ref('')
const selectedPresetId = ref<string | null>(null)

// Snapshot for cancel restore
let snapshot: {
  mode: 'light' | 'dark'
  accentColor: string
  fontSize: 'small' | 'medium' | 'large'
  presetCSS: string
  activePresetId: string | null
} | null = null

function open(): void {
  // Take snapshot
  snapshot = {
    mode: settingsStore.themeMode,
    accentColor: settingsStore.accentColor,
    fontSize: settingsStore.fontSize,
    presetCSS: settingsStore.presetCSS,
    activePresetId: settingsStore.activePresetId,
  }

  // Init local state from store
  localMode.value = settingsStore.themeMode
  localAccentColor.value = settingsStore.accentColor
  localFontSize.value = settingsStore.fontSize
  localCSS.value = settingsStore.presetCSS
  selectedPresetId.value = settingsStore.activePresetId

  visible.value = true
}

function close(): void {
  visible.value = false
  snapshot = null
}

// Live preview: watch local basic settings and apply immediately
watch([localMode, localAccentColor, localFontSize], () => {
  themeStore.mode = localMode.value
  themeStore.apply()
  document.documentElement.style.setProperty('--accent', localAccentColor.value)
  const fsMap: Record<string, string> = { small: '0.75rem', medium: '0.875rem', large: '0.9375rem' }
  document.documentElement.style.setProperty('--fs-base', fsMap[localFontSize.value])
})

function applyPreset(): void {
  if (selectedPresetId.value) {
    const preset = settingsStore.presets.find(p => p.id === selectedPresetId.value)
    localCSS.value = preset?.css ?? ''
    // Apply the CSS immediately
    let el = document.getElementById('theme-preset') as HTMLStyleElement | null
    if (!el) {
      el = document.createElement('style')
      el.id = 'theme-preset'
      document.head.appendChild(el)
    }
    el.textContent = localCSS.value
  } else {
    localCSS.value = ''
    document.getElementById('theme-preset')?.remove()
  }
}

function onSaveAs(): void {
  const name = prompt('输入预设名称：')
  if (!name?.trim()) return
  const id = settingsStore.createPreset(name.trim(), localCSS.value)
  selectedPresetId.value = id
}

function onDeletePreset(): void {
  if (!selectedPresetId.value) return
  const preset = settingsStore.presets.find(p => p.id === selectedPresetId.value)
  if (!preset) return
  if (!confirm(`确定删除预设 "${preset.name}"？`)) return
  settingsStore.deletePreset(selectedPresetId.value)
  selectedPresetId.value = null
  localCSS.value = ''
}

function onSave(): void {
  settingsStore.themeMode = localMode.value
  settingsStore.accentColor = localAccentColor.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.presetCSS = localCSS.value
  settingsStore.save()
  settingsStore.applyTheme()

  // Persist active preset to state.json
  if (selectedPresetId.value !== settingsStore.activePresetId) {
    settingsStore.setActivePreset(selectedPresetId.value)
  }

  close()
}

function onCancel(): void {
  // Restore from snapshot
  if (snapshot) {
    themeStore.mode = snapshot.mode
    themeStore.apply()
    settingsStore.themeMode = snapshot.mode
    settingsStore.accentColor = snapshot.accentColor
    settingsStore.fontSize = snapshot.fontSize
    settingsStore.presetCSS = snapshot.presetCSS
    settingsStore.activePresetId = snapshot.activePresetId
    settingsStore.applyTheme()
  }
  close()
}

defineExpose({ open, close })
</script>

<style scoped>
.settings-overlay {
  position: fixed; inset: 0; z-index: 200;
  background: rgba(0,0,0,0.2);
  display: flex; align-items: center; justify-content: center;
}

.settings-window {
  width: 720px; height: 520px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-lg);
  display: flex; flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.settings-header h2 {
  margin: 0; font-size: var(--fs-base); font-weight: var(--fw-bold);
}

.settings-body {
  display: flex; flex: 1; overflow: hidden;
}

/* Left tabs */
.settings-tabs {
  width: 160px; flex-shrink: 0;
  border-right: 1px solid var(--border);
  padding: 8px;
  display: flex; flex-direction: column; gap: 2px;
}
.tab-btn {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px; border-radius: var(--r-md);
  border: none; background: transparent;
  font-size: var(--fs-sm); font-weight: var(--fw-medium);
  color: var(--text-secondary); cursor: pointer;
  transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
  height: auto; text-align: left;
}
.tab-btn:hover { background: var(--surface-hover); color: var(--text); }
.tab-btn.active { background: var(--surface-active); color: var(--text); font-weight: var(--fw-semibold); }

/* Right content */
.settings-content {
  flex: 1; overflow-y: auto; padding: 16px 20px;
}

.tab-panel {
  display: flex; flex-direction: column; gap: 0;
}

.placeholder-text {
  color: var(--text-muted); font-size: var(--fs-sm);
  text-align: center; padding: 80px 0;
}

/* Sections */
.setting-section {
  margin-bottom: 16px;
}
.section-title {
  display: block;
  font-size: var(--fs-sm); font-weight: var(--fw-semibold);
  margin-bottom: 8px; color: var(--text-heading);
}
.section-divider {
  border: none; border-top: 1px solid var(--border-light);
  margin: 16px 0;
}

.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 0;
}
.setting-row > label {
  font-size: var(--fs-sm); font-weight: var(--fw-medium);
  color: var(--text);
}

.radio-group {
  display: flex; gap: 16px;
}
.radio-label {
  font-size: var(--fs-sm); cursor: pointer;
  display: flex; align-items: center; gap: 4px;
}

.color-input {
  width: 36px; height: 28px; padding: 2px;
  border: 1px solid var(--border); border-radius: var(--r-sm);
  cursor: pointer;
}

.btn-group {
  display: flex; gap: 0;
}
.btn-group button {
  padding: 4px 12px; height: 28px;
  border: 1px solid var(--border); background: var(--surface);
  font-size: var(--fs-xs); cursor: pointer; color: var(--text-secondary);
  transition: background var(--fast) var(--ease), color var(--fast) var(--ease);
}
.btn-group button:first-child { border-radius: var(--r-sm) 0 0 var(--r-sm); }
.btn-group button:last-child { border-radius: 0 var(--r-sm) var(--r-sm) 0; }
.btn-group button.active {
  background: var(--accent); color: var(--accent-fg); border-color: var(--accent);
}

/* Preset row */
.preset-row {
  display: flex; gap: 6px; align-items: center;
}
.preset-select {
  flex: 1; font-size: var(--fs-sm);
}
.sm { font-size: var(--fs-xs); height: 28px; }
.danger { color: var(--danger); }

/* CSS editor */
.css-editor {
  width: 100%; height: 160px;
  font-family: var(--font-mono); font-size: var(--fs-xs);
  line-height: 1.6;
  padding: 8px 10px;
  border: 1px solid var(--border); border-radius: var(--r-md);
  background: var(--bg); color: var(--text);
  resize: vertical; tab-size: 2;
  margin-bottom: 8px;
}
.css-editor:focus { outline: none; border-color: var(--accent); }

/* Footer */
.settings-footer {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/SettingsModal.vue
git commit -m "feat: add SettingsModal — theme presets, CSS editor, live preview"
```

---

### Task 6: Update Topbar — gear icon + modal trigger

**Files:**
- Modify: `src/components/Topbar.vue`

**Interfaces:**
- Consumes: `SettingsModal` from Task 5 (via template ref)
- Produces: Gear icon button that opens SettingsModal

- [ ] **Step 1: Replace moon/sun toggle with gear icon and SettingsModal**

Replace `src/components/Topbar.vue`:

```vue
<template>
  <header class="topbar">
    <div class="left">
      <TablerIcon name="database" :size="22" :stroke="1.5" />
      <span class="repo-name">{{ repoStore.repoPath ? basename(repoStore.repoPath) : 'Index' }}</span>
    </div>
    <div class="actions">
      <button class="primary" @click="$emit('newItem')">
        <TablerIcon name="plus" :size="16" /> 新建条目
      </button>
      <button class="icon-btn" @click="settingsRef?.open()" title="设置">
        <TablerIcon name="settings" :size="18" />
      </button>
      <button class="icon-btn" @click="$emit('openTypeManager')" title="类别管理">
        <TablerIcon name="category" :size="18" />
      </button>
    </div>
    <SettingsModal ref="settingsRef" />
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRepoStore } from '@/stores/repo'
import TablerIcon from './TablerIcon.vue'
import SettingsModal from './SettingsModal.vue'

const repoStore = useRepoStore()
const settingsRef = ref<InstanceType<typeof SettingsModal> | null>(null)

function basename(p: string): string { return p.split(/[/\\]/).pop() || p }
defineEmits<{ newItem: []; openTypeManager: [] }>()
</script>

<style scoped>
.topbar {
  height: var(--topbar-h); display: flex; align-items: center; justify-content: space-between;
  padding: 0 16px; background: var(--surface); border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.left { display: flex; align-items: center; gap: 8px; min-width: 0; color: var(--accent); }
.repo-name { font-weight: var(--fw-semibold); font-size: var(--fs-base); color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.actions { display: flex; align-items: center; gap: 4px; }
</style>
```

Key changes:
- Removed `useThemeStore` import and moon/sun toggle button
- Added `SettingsModal` import and template ref
- Gear icon (`settings`) button with `@click="settingsRef?.open()"`
- `SettingsModal` rendered inside the header (Teleported to body anyway)

- [ ] **Step 2: Commit**

```bash
git add src/components/Topbar.vue
git commit -m "feat: replace theme toggle with settings gear icon, wire SettingsModal"
```

---

### Task 7: Update App.vue — theme initialization

**Files:**
- Modify: `src/App.vue`

**Interfaces:**
- Consumes: `useSettingsStore().load()`, `useThemeStore().init()`, `settingsStore.loadActivePresetFromRepo()` from Tasks 3, 4
- Produces: Theme initialized on app mount

- [ ] **Step 1: Add theme initialization to App.vue**

Modify `src/App.vue` — add `onMounted` with theme initialization, and `import` for settings store:

In `<script setup>`, add after existing imports:

```typescript
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
```

In `<script setup>`, add after existing store instantiations:

```typescript
const settingsStore = useSettingsStore()
```

Add `onMounted` at the end of `<script setup>`:

```typescript
onMounted(() => {
  settingsStore.load()
  themeStore.init()
})
```

In `onRepoOpened()`, add after `await Promise.all([...])`:

```typescript
await settingsStore.loadActivePresetFromRepo()
settingsStore.applyTheme()
```

The final `App.vue` `<script setup>`:

```typescript
import { ref, onMounted } from 'vue'
import { useRepoStore } from '@/stores/repo'
import { useThemeStore } from '@/stores/theme'
import { useSettingsStore } from '@/stores/settings'
import { useTypeStore } from '@/stores/types'
import { useGroupStore } from '@/stores/groups'
import { useTagStore } from '@/stores/tags'
import { useItemStore } from '@/stores/items'
import EmptyState from '@/components/EmptyState.vue'
import Topbar from '@/components/Topbar.vue'
import Sidebar from '@/components/Sidebar.vue'
import CenterList from '@/components/CenterList.vue'
import RightPanel from '@/components/RightPanel.vue'
import StatusBar from '@/components/StatusBar.vue'
import NewItemDialog from '@/components/NewItemDialog.vue'
import Toast from '@/components/Toast.vue'

const repoStore = useRepoStore()
const themeStore = useThemeStore()
const settingsStore = useSettingsStore()
const typeStore = useTypeStore()
const groupStore = useGroupStore()
const tagStore = useTagStore()
const itemStore = useItemStore()

const showNewItem = ref(false)
const rightTab = ref<'detail' | 'types'>('detail')
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

async function onRepoOpened() {
  await Promise.all([
    typeStore.fetchAll(),
    groupStore.fetchAll(),
    tagStore.fetchAll(),
    itemStore.fetchList(),
  ])
  await settingsStore.loadActivePresetFromRepo()
  settingsStore.applyTheme()
}

onMounted(() => {
  settingsStore.load()
  themeStore.init()
})
```

Also remove the duplicate `.dark` class from template line 2 — change:
```html
<div class="app" :class="{ dark: themeStore.mode === 'dark' }">
```
to:
```html
<div class="app">
```

Since `themeStore.init()` already sets `.dark` on `document.documentElement`, the duplicate on `.app` is unnecessary.

- [ ] **Step 2: Commit**

```bash
git add src/App.vue
git commit -m "feat: initialize theme from settingsStore on app mount and repo open"
```

---

### Task 8: Tighten CenterList row height

**Files:**
- Modify: `src/components/CenterList.vue`

**Interfaces:**
- Produces: Tighter `.row` padding — purely visual change

- [ ] **Step 1: Reduce .row padding**

In `src/components/CenterList.vue`, line 74, change:

```css
padding: 8px 12px 8px 8px;
```

to:

```css
padding: 4px 12px 4px 8px;
```

- [ ] **Step 2: Commit**

```bash
git add src/components/CenterList.vue
git commit -m "style: tighten CenterList row padding for compact layout"
```

---

### Verification Checklist

After all tasks are implemented:

- [ ] `pnpm dev` — app launches, visual style is "Quiet Strength" (muted gray text, ink-black accent, warm neutral surfaces)
- [ ] Dark/light toggle via SettingsModal works with live preview
- [ ] Accent color picker changes button colors in real time
- [ ] Font size toggle (small/medium/large) affects UI text
- [ ] CSS editor: paste `:root { --accent: #E53935; }` → save as "Red Test" → apply → buttons turn red
- [ ] Preset dropdown shows saved presets; switching applies them
- [ ] Cancel in SettingsModal reverts all changes to pre-open state
- [ ] Save persists to localStorage; refresh browser → settings survive
- [ ] Open a repo → state.json gets `activePresetId` written
- [ ] Close and reopen repo → active preset CSS re-applied from state.json
- [ ] CenterList rows are visually tighter (4px vertical padding)
- [ ] Existing functionality (CRUD items, types, groups, tags) unaffected
