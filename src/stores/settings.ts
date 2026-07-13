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

    // Inject/update theme-override style tag with all CSS variable overrides
    let el = document.getElementById('theme-override') as HTMLStyleElement | null
    if (!el) {
      el = document.createElement('style')
      el.id = 'theme-override'
      document.head.appendChild(el)
    }
    el.textContent = `:root {
  --accent: ${accentColor.value};
  --fs-base: ${FONT_SIZE_MAP[fontSize.value]};
  --fs-sm: ${fontSize.value === 'small' ? '0.75rem' : '0.8125rem'};
}`

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
