import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

type ThemeMode = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>('light')

  function toggle(): void {
    mode.value = mode.value === 'light' ? 'dark' : 'light'
  }

  function apply(): void {
    document.documentElement.classList.toggle('dark', mode.value === 'dark')
  }

  // Watch and apply theme changes
  watch(mode, apply, { immediate: true })

  return { mode, toggle, apply }
})
