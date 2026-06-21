import { computed, ref } from 'vue'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

export const locales = ['zh-CN', 'en-US'] as const
export type Locale = (typeof locales)[number]
export type LanguagePreference = 'system' | Locale

interface Messages {
  [key: string]: string | Messages
}
type MessageKey = string
type MessageParams = Record<string, string | number | boolean | null | undefined>

const messages: Record<Locale, Messages> = {
  'zh-CN': zhCN as Messages,
  'en-US': enUS as Messages
}

export const currentLocale = ref<Locale>('zh-CN')
export const currentLanguagePreference = ref<LanguagePreference>('system')

export const currentLanguageTag = computed(() => currentLocale.value)
export const isChineseLocale = computed(() => currentLocale.value === 'zh-CN')

export function isLocale(value: unknown): value is Locale {
  return typeof value === 'string' && locales.includes(value as Locale)
}

export function isLanguagePreference(value: unknown): value is LanguagePreference {
  return value === 'system' || isLocale(value)
}

export function setLocale(locale: Locale) {
  currentLocale.value = locale
  document.documentElement.lang = locale
}

export function setLanguagePreference(preference: LanguagePreference) {
  currentLanguagePreference.value = preference
  setLocale(resolveLanguagePreference(preference))
}

export function initLocale(preference?: string | null) {
  setLanguagePreference(isLanguagePreference(preference) ? preference : 'system')
}

export function resolveLanguagePreference(preference: LanguagePreference): Locale {
  if (preference !== 'system') return preference
  const language = navigator.language || navigator.languages?.[0] || ''
  return language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

export function t(key: MessageKey, params?: MessageParams): string {
  const value = lookup(messages[currentLocale.value], key)
  if (typeof value === 'string') return interpolate(value, params)

  const fallback = lookup(messages['zh-CN'], key)
  if (typeof fallback === 'string') return interpolate(fallback, params)

  return key
}

function lookup(source: unknown, key: string): unknown {
  return key.split('.').reduce<unknown>((node, part) => {
    if (!node || typeof node !== 'object') return undefined
    return (node as Record<string, unknown>)[part]
  }, source)
}

function interpolate(message: string, params?: MessageParams) {
  if (!params) return message
  return message.replace(/\{([A-Za-z0-9_]+)\}/g, (_, name: string) => {
    const value = params[name]
    return value === null || value === undefined ? '' : String(value)
  })
}

export function useI18n() {
  return {
    locale: currentLocale,
    languagePreference: currentLanguagePreference,
    languageTag: currentLanguageTag,
    setLocale,
    setLanguagePreference,
    t
  }
}
