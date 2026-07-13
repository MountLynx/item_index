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
  bgColor: string
  textColor: string
  textColorAuto: boolean
  fontSize: 'small' | 'medium' | 'large'
  presets: ThemePreset[]
}

interface GeneralSection {
  locale: string
}

interface GlobalSettings {
  theme: ThemeSection
  general: GeneralSection
}

const STORAGE_KEY = 'index-settings'

function defaultSettings(): GlobalSettings {
  return {
    general: { locale: 'zh-CN' },
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

function loadFromStorage(): GlobalSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaultSettings()
    const parsed = JSON.parse(raw) as GlobalSettings
    // Ensure nested defaults exist
    return {
      general: {
        locale: parsed.general?.locale ?? 'zh-CN',
      },
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

export const useSettingsStore = defineStore('settings', () => {
  // ── State ──
  const themeMode = ref<'light' | 'dark'>('light')
  const accentColor = ref('#1A1C1E')
  const fontSize = ref<'small' | 'medium' | 'large'>('medium')
  const presets = ref<ThemePreset[]>([])
  const activePresetId = ref<string | null>(null)
  const presetCSS = ref('')
  const bgColor = ref('#FFFFFF')
  const textColor = ref('#333333')
  const textColorAuto = ref(true)
  const locale = ref('zh-CN')

  // ── Actions ──
  function load(): void {
    const settings = loadFromStorage()
    themeMode.value = settings.theme.mode
    accentColor.value = settings.theme.accentColor
    fontSize.value = settings.theme.fontSize
    bgColor.value = settings.theme.bgColor
    textColor.value = settings.theme.textColor
    textColorAuto.value = settings.theme.textColorAuto
    presets.value = settings.theme.presets
    locale.value = settings.general.locale

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
      general: { locale: locale.value },
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

  function setLocale(loc: string): void {
    locale.value = loc
    save()
  }

  return {
    themeMode, accentColor, bgColor, textColor, textColorAuto, fontSize, locale,
    presets, activePresetId, presetCSS,
    load, loadActivePresetFromRepo, save, applyTheme,
    createPreset, updatePreset, deletePreset, setActivePreset,
    applyPresetCSS, setLocale,
  }
})
