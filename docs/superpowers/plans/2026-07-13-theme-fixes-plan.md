# Theme Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix danger button ghost styling, add bgColor/textColor to settings, implement form > CSS priority with sync button.

**Architecture:** Three files modified. Task 1 (danger CSS) is independent. Task 2 (settings store) adds utility functions and expanded state that Task 3 (SettingsModal) consumes. Dark mode verification is a manual check, no code change.

**Tech Stack:** Vue 3 + TypeScript, Pinia, CSS custom properties

## Global Constraints

- `button.danger` becomes ghost style: transparent bg, red text, subtle red hover
- `bgColor` default `#FFFFFF`, `textColor` default `#333333`, `textColorAuto` default `true`
- `accent-fg` is computed purely from accentColor luminance, never exposed in UI
- `textColor` auto-computes from bgColor luminance when `textColorAuto === true`
- `--surface` auto-computes from bgColor (luminance × 0.97)
- Form values MUST always override CSS preset values — `#theme-override` style tag must be AFTER `#theme-preset` in DOM
- "与自定义主题保持一致" button parses CSS → fills form → sets `textColorAuto = false`
- "应用" button now also backfills form from CSS in addition to injecting preset
- All existing `var(--xxx)` references must continue to work unchanged

---

### Task 1: Fix danger button to ghost style

**Files:**
- Modify: `src/assets/theme.css:153-154`

**Interfaces:**
- Produces: Updated `button.danger` style consumed by TypeManager, CenterList, etc.

- [ ] **Step 1: Replace button.danger styles**

In `src/assets/theme.css`, replace lines 153-154:

```css
button.danger { background: var(--danger); color: #fff; border-color: var(--danger); }
button.danger:hover { background: var(--danger-hover); }
```

with:

```css
button.danger { background: transparent; color: var(--danger); border-color: transparent; }
button.danger:hover { background: var(--danger-subtle); color: var(--danger-hover); }
```

- [ ] **Step 2: Verify existing danger button references**

These elements use `.danger` class and need no changes — they inherit the new ghost style automatically:
- TypeManager.vue line 33: `<button class="icon-btn sm danger">` (delete type)
- TypeManager.vue line 168: `<button class="icon-btn sm danger">` (delete field)
- TypeManager.vue line 253: `<button class="sm danger">` (confirm delete)
- CenterList.vue: `.menu-item { color: var(--danger); }` (delete item, already ghost)

- [ ] **Step 3: Commit**

```bash
git add src/assets/theme.css
git commit -m "fix: change button.danger to ghost style — transparent bg, red text"
```

---

### Task 2: Extend SettingsStore — utility functions, new state, applyTheme update

**Files:**
- Modify: `src/stores/settings.ts`

**Interfaces:**
- Consumes: None new (uses existing `FONT_SIZE_MAP`, `generateId`)
- Produces:
  - New state: `bgColor: Ref<string>`, `textColor: Ref<string>`, `textColorAuto: Ref<boolean>`
  - New exports: `hexToRgb(hex: string): {r:number,g:number,b:number}`, `computeLuminance(r,g,b): number`, `computeAccentFg(hex): string`, `computeTextColor(bgHex): string`, `computeSurface(bgHex): string`, `parseCSSVariables(css: string): Partial<ThemeOverrides>`
  - Updated: `applyTheme()` injects extended variables; `save()`/`load()` handle new fields
  - Interface: `ThemeOverrides { accentColor?, bgColor?, textColor?, fontSize? }`

- [ ] **Step 1: Add utility functions before the store definition**

Insert after `FONT_SIZE_MAP` (line 70) and before `export const useSettingsStore` (line 72):

```typescript
function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const h = hex.replace('#', '')
  return {
    r: parseInt(h.substring(0, 2), 16),
    g: parseInt(h.substring(2, 4), 16),
    b: parseInt(h.substring(4, 6), 16),
  }
}

function computeLuminance(r: number, g: number, b: number): number {
  // W3C relative luminance, 0-255 input → 0-1 output
  const rs = r / 255, gs = g / 255, bs = b / 255
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs
}

function computeAccentFg(accentHex: string): string {
  const { r, g, b } = hexToRgb(accentHex)
  return computeLuminance(r, g, b) > 0.5 ? '#1E1E1E' : '#FFFFFF'
}

function computeTextColor(bgHex: string): string {
  const { r, g, b } = hexToRgb(bgHex)
  return computeLuminance(r, g, b) > 0.5 ? '#333333' : '#F4F4F5'
}

function computeSurface(bgHex: string): string {
  const { r, g, b } = hexToRgb(bgHex)
  const factor = 0.97
  const toHex = (v: number) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0')
  return `#${toHex(r * factor)}${toHex(g * factor)}${toHex(b * factor)}`
}

export interface ThemeOverrides {
  accentColor?: string
  bgColor?: string
  textColor?: string
  fontSize?: 'small' | 'medium' | 'large'
}

export function parseCSSVariables(css: string): ThemeOverrides {
  const result: ThemeOverrides = {}
  const extract = (name: string): string | null => {
    const m = css.match(new RegExp(`${name}\\s*:\\s*([^;}\\n]+)`))
    return m ? m[1].trim() : null
  }
  const a = extract('--accent')
  if (a) result.accentColor = a
  const b = extract('--bg')
  if (b) result.bgColor = b
  const t = extract('--text')
  if (t) result.textColor = t
  const fs = extract('--fs-base')
  if (fs) {
    if (fs === '0.75rem' || fs === '12px') result.fontSize = 'small'
    else if (fs === '0.875rem' || fs === '14px') result.fontSize = 'medium'
    else if (fs === '0.9375rem' || fs === '15px') result.fontSize = 'large'
  }
  return result
}
```

- [ ] **Step 2: Add new state fields and update ThemeSection/GlobalSettings/defaultSettings**

Update `ThemeSection` interface (line 13-18):

```typescript
interface ThemeSection {
  mode: 'light' | 'dark'
  accentColor: string
  bgColor: string
  textColor: string
  textColorAuto: boolean
  fontSize: 'small' | 'medium' | 'large'
  presets: ThemePreset[]
}
```

Update `defaultSettings()` (line 26-34):

```typescript
function defaultSettings(): GlobalSettings {
  return {
    theme: {
      mode: 'light',
      accentColor: '#1A1C1E',
      bgColor: '#FFFFFF',
      textColor: '#333333',
      textColorAuto: true,
      fontSize: 'medium',
      presets: [],
    },
  }
}
```

Update `loadFromStorage()` (line 43-49):

```typescript
return {
  theme: {
    mode: parsed.theme?.mode ?? 'light',
    accentColor: parsed.theme?.accentColor ?? '#1A1C1E',
    bgColor: parsed.theme?.bgColor ?? '#FFFFFF',
    textColor: parsed.theme?.textColor ?? '#333333',
    textColorAuto: parsed.theme?.textColorAuto ?? true,
    fontSize: parsed.theme?.fontSize ?? 'medium',
    presets: Array.isArray(parsed.theme?.presets) ? parsed.theme.presets : [],
  },
}
```

- [ ] **Step 3: Add new state refs inside the store**

After line 78 (`const activePresetId`):

```typescript
const bgColor = ref('#FFFFFF')
const textColor = ref('#333333')
const textColorAuto = ref(true)
```

- [ ] **Step 4: Update load() to populate new fields**

In `load()` (after line 86), add:

```typescript
bgColor.value = settings.theme.bgColor
textColor.value = settings.theme.textColor
textColorAuto.value = settings.theme.textColorAuto
```

- [ ] **Step 5: Update save() to include new fields**

In `save()`, update the settings object (lines 110-118):

```typescript
function save(): void {
  const settings: GlobalSettings = {
    theme: {
      mode: themeMode.value,
      accentColor: accentColor.value,
      bgColor: bgColor.value,
      textColor: textColor.value,
      textColorAuto: textColorAuto.value,
      fontSize: fontSize.value,
      presets: presets.value,
    },
  }
  saveToStorage(settings)
}
```

- [ ] **Step 6: Replace applyTheme() to inject all variables + ensure priority**

Replace `applyTheme()` (lines 121-139):

```typescript
function applyTheme(): void {
  // Compute derived values
  const accentFg = computeAccentFg(accentColor.value)
  const txtColor = textColorAuto.value
    ? computeTextColor(bgColor.value)
    : textColor.value
  const surColor = computeSurface(bgColor.value)
  const fsSm = fontSize.value === 'small' ? '0.75rem' : '0.8125rem'

  // Create/update theme-override style tag
  let el = document.getElementById('theme-override') as HTMLStyleElement | null
  if (!el) {
    el = document.createElement('style')
    el.id = 'theme-override'
    document.head.appendChild(el)
  }
  el.textContent = `:root {
  --accent: ${accentColor.value};
  --accent-fg: ${accentFg};
  --bg: ${bgColor.value};
  --surface: ${surColor};
  --text: ${txtColor};
  --text-heading: ${txtColor};
  --fs-base: ${FONT_SIZE_MAP[fontSize.value]};
  --fs-sm: ${fsSm};
}`

  // Ensure theme-override is AFTER theme-preset in DOM (form > CSS)
  const presetEl = document.getElementById('theme-preset')
  if (presetEl && el.nextSibling !== presetEl) {
    // Move override to be right after preset (or at end if no preset)
    if (presetEl.nextSibling) {
      document.head.insertBefore(el, presetEl.nextSibling)
    } else {
      document.head.appendChild(el)
    }
  }

  applyPresetCSS()
}
```

- [ ] **Step 7: Update store return to expose new state**

Update return (line 203-209):

```typescript
return {
  themeMode, accentColor, bgColor, textColor, textColorAuto, fontSize,
  presets, activePresetId, presetCSS,
  load, loadActivePresetFromRepo, save, applyTheme,
  createPreset, updatePreset, deletePreset, setActivePreset,
  applyPresetCSS,
}
```

- [ ] **Step 8: Type-check and commit**

```bash
pnpm vue-tsc --noEmit
```

Expected: No errors.

```bash
git add src/stores/settings.ts
git commit -m "feat: add bgColor/textColor/textColorAuto to settings, auto-compute accent-fg and surface"
```

---

### Task 3: Update SettingsModal — new form rows, sync button, CSS parsing

**Files:**
- Modify: `src/components/SettingsModal.vue`

**Interfaces:**
- Consumes: `useSettingsStore()` new state: `bgColor`, `textColor`, `textColorAuto`; new function `parseCSSVariables`
- Produces: Updated SettingsModal with expanded form, sync button, backfill logic

- [ ] **Step 1: Add new local state variables**

In `<script setup>`, after line 140 (`const localCSS`):

```typescript
const localBgColor = ref('#FFFFFF')
const localTextColor = ref('#333333')
const localTextColorAuto = ref(true)
```

- [ ] **Step 2: Add new fields to snapshot type and open()**

Update snapshot type (line 144-150):

```typescript
let snapshot: {
  mode: 'light' | 'dark'
  accentColor: string
  bgColor: string
  textColor: string
  textColorAuto: boolean
  fontSize: 'small' | 'medium' | 'large'
  presetCSS: string
  activePresetId: string | null
} | null = null
```

Update `open()` (lines 152-170):

```typescript
function open(): void {
  snapshot = {
    mode: settingsStore.themeMode,
    accentColor: settingsStore.accentColor,
    bgColor: settingsStore.bgColor,
    textColor: settingsStore.textColor,
    textColorAuto: settingsStore.textColorAuto,
    fontSize: settingsStore.fontSize,
    presetCSS: settingsStore.presetCSS,
    activePresetId: settingsStore.activePresetId,
  }

  localMode.value = settingsStore.themeMode
  localAccentColor.value = settingsStore.accentColor
  localBgColor.value = settingsStore.bgColor
  localTextColor.value = settingsStore.textColor
  localTextColorAuto.value = settingsStore.textColorAuto
  localFontSize.value = settingsStore.fontSize
  localCSS.value = settingsStore.presetCSS
  selectedPresetId.value = settingsStore.activePresetId

  visible.value = true
}
```

- [ ] **Step 3: Add "应用CSS值回填表单" helper function**

After `applyPreset()` function (line 203), add:

```typescript
function backfillFormFromCSS(css: string): void {
  const overrides = parseCSSVariables(css)
  if (overrides.accentColor) localAccentColor.value = overrides.accentColor
  if (overrides.bgColor) localBgColor.value = overrides.bgColor
  if (overrides.textColor) {
    localTextColor.value = overrides.textColor
    localTextColorAuto.value = false
  }
  if (overrides.fontSize) localFontSize.value = overrides.fontSize
}
```

- [ ] **Step 4: Update live preview watch**

Replace the watch (lines 178-185):

```typescript
watch([localMode, localAccentColor, localBgColor, localTextColor, localTextColorAuto, localFontSize], () => {
  themeStore.mode = localMode.value
  themeStore.apply()
  settingsStore.accentColor = localAccentColor.value
  settingsStore.bgColor = localBgColor.value
  settingsStore.textColor = localTextColor.value
  settingsStore.textColorAuto = localTextColorAuto.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.applyTheme()
})
```

- [ ] **Step 5: Update applyPreset() to also backfill form**

Replace `applyPreset()` (lines 187-203):

```typescript
function applyPreset(): void {
  if (selectedPresetId.value) {
    const preset = settingsStore.presets.find(p => p.id === selectedPresetId.value)
    localCSS.value = preset?.css ?? ''
    // Backfill form from CSS
    if (localCSS.value) {
      backfillFormFromCSS(localCSS.value)
    }
    // Inject the CSS as theme-preset
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
```

- [ ] **Step 6: Add "与自定义主题保持一致" function**

After `applyPreset()`, add:

```typescript
function onSyncFromCSS(): void {
  if (!localCSS.value.trim()) return
  backfillFormFromCSS(localCSS.value)
  // Apply the synced values immediately
  settingsStore.accentColor = localAccentColor.value
  settingsStore.bgColor = localBgColor.value
  settingsStore.textColor = localTextColor.value
  settingsStore.textColorAuto = localTextColorAuto.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.applyTheme()
}
```

- [ ] **Step 7: Update onSave() and onCancel() snapshots**

Update `onSave()` (lines 222-237) to include new fields:

```typescript
async function onSave(): Promise<void> {
  settingsStore.themeMode = localMode.value
  settingsStore.accentColor = localAccentColor.value
  settingsStore.bgColor = localBgColor.value
  settingsStore.textColor = localTextColor.value
  settingsStore.textColorAuto = localTextColorAuto.value
  settingsStore.fontSize = localFontSize.value
  settingsStore.save()

  if (selectedPresetId.value !== settingsStore.activePresetId) {
    await settingsStore.setActivePreset(selectedPresetId.value)
  }
  settingsStore.presetCSS = localCSS.value
  settingsStore.applyTheme()

  close()
}
```

Update `onCancel()` (lines 239-252) to restore new fields:

```typescript
function onCancel(): void {
  if (snapshot) {
    themeStore.mode = snapshot.mode
    themeStore.apply()
    settingsStore.themeMode = snapshot.mode
    settingsStore.accentColor = snapshot.accentColor
    settingsStore.bgColor = snapshot.bgColor
    settingsStore.textColor = snapshot.textColor
    settingsStore.textColorAuto = snapshot.textColorAuto
    settingsStore.fontSize = snapshot.fontSize
    settingsStore.presetCSS = snapshot.presetCSS
    settingsStore.activePresetId = snapshot.activePresetId
    settingsStore.applyTheme()
  }
  close()
}
```

- [ ] **Step 8: Update template — add bgColor, textColor rows + sync button**

After the `强调色` row (line 54), insert before the `字号` row:

```html
                <div class="setting-row">
                  <label>背景色</label>
                  <input type="color" v-model="localBgColor" class="color-input" />
                </div>
                <div class="setting-row">
                  <label>
                    字体色
                    <span v-if="localTextColorAuto" class="auto-tag">自动 ✨</span>
                  </label>
                  <div class="text-color-group">
                    <input
                      type="color"
                      v-model="localTextColor"
                      class="color-input"
                      :disabled="localTextColorAuto"
                    />
                    <label class="checkbox-label">
                      <input type="checkbox" v-model="localTextColorAuto" />
                      <span class="text-xs">自动</span>
                    </label>
                  </div>
                </div>
```

Replace the CSS editor footer (line 101) with three buttons:

```html
                <div class="css-actions">
                  <button class="sm primary" @click="applyPreset">应用</button>
                  <button class="sm" @click="onSaveAs">保存为预设</button>
                  <button class="sm" @click="onSyncFromCSS">与自定义主题保持一致</button>
                </div>
```

- [ ] **Step 9: Add supporting styles**

Add to `<style scoped>` section:

```css
.auto-tag {
  font-size: var(--fs-xs); color: var(--text-muted);
  font-weight: var(--fw-normal); margin-left: 4px;
}
.text-color-group {
  display: flex; align-items: center; gap: 8px;
}
.checkbox-label {
  display: flex; align-items: center; gap: 4px; cursor: pointer;
  font-size: var(--fs-xs); color: var(--text-secondary);
}
.css-actions {
  display: flex; gap: 6px; flex-wrap: wrap;
}
```

- [ ] **Step 10: Type-check and commit**

```bash
pnpm vue-tsc --noEmit
```

Expected: No errors.

```bash
git add src/components/SettingsModal.vue
git commit -m "feat: add bgColor/textColor form rows, sync button, CSS backfill to SettingsModal"
```

---

### Verification Checklist

After all tasks:

- [ ] `pnpm vue-tsc --noEmit` — no errors
- [ ] `cargo build` (src-tauri) — no errors
- [ ] Delete buttons (TypeManager) show red ghost style (transparent bg, red icon/text)
- [ ] Hover on delete buttons shows subtle red background
- [ ] Dark mode: accent buttons show near-white (#F4F4F5) with dark text (#1E1E1E)
- [ ] SettingsModal → bgColor picker changes page background in real time
- [ ] SettingsModal → textColor auto checkbox enabled: text color follows bgColor luminance
- [ ] SettingsModal → uncheck auto, manually pick textColor → text changes
- [ ] Write `:root { --accent: #E53935; }` in CSS editor → click "应用" → accent color picker shows red
- [ ] After "应用", change accent color in form → form value wins (visible in UI)
- [ ] Click "与自定义主题保持一致" → form values reset to match CSS editor content
- [ ] Save → close → reopen SettingsModal → settings persist
- [ ] Cancel → all changes revert to pre-open state
