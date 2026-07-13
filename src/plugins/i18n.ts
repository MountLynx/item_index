import { createI18n } from 'vue-i18n'
import zhCN from '@/locales/zh-CN'
import en from '@/locales/en'

function getInitialLocale(): string {
  try {
    const raw = localStorage.getItem('index-settings')
    if (raw) {
      const parsed = JSON.parse(raw)
      return parsed?.general?.locale ?? 'zh-CN'
    }
  } catch {}
  return 'zh-CN'
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    en: en,
  },
})
