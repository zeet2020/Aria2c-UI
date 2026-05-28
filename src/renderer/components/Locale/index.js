import { createI18n } from 'vue-i18n'
import enUS from '@shared/locales/en-US'

// English-only build. Legacy mode keeps the Options-API `$t` available.
export const i18n = createI18n({
  legacy: true,
  globalInjection: true,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  messages: {
    'en-US': enUS
  }
})

// Compatibility shim for existing callers (commands.js, App.vue, main.js).
export function getLocaleManager () {
  return {
    getI18n: () => i18n.global, // exposes `.t`
    changeLanguage: () => {},
    changeLanguageByLocale: () => {}
  }
}
