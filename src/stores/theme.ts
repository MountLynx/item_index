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
