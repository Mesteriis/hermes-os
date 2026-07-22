import type { Locale } from '../../../platform/i18n/types'

export const languageLocaleOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' }
] as const

export function isSupportedLocale(value: string): value is Locale {
  return languageLocaleOptions.some((option) => option.value === value)
}
