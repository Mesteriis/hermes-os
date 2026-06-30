import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { Locale } from '../../../platform/i18n/types'
import { useSaveFrontendLocaleMutation } from './useSettingsQuery'

const localeOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' }
] as const

const validLocaleValues = ['en', 'ru'] as const
type LocaleValue = (typeof validLocaleValues)[number]

export function useLanguageSettingsSurface() {
  const { locale, setLocale } = useI18n()
  const saveLocale = useSaveFrontendLocaleMutation()

  const currentLocale = computed(() => locale.value)
  const isBusy = computed(() => saveLocale.isPending.value)

  async function handleLocaleChange(value: string) {
    if (!validLocaleValues.includes(value as LocaleValue)) return

    const nextLocale = value as Locale
    const previousLocale = locale.value

    setLocale(nextLocale)
    try {
      await saveLocale.mutateAsync(nextLocale)
    } catch (error) {
      setLocale(previousLocale as Locale)
      console.error('Failed to save locale setting:', error)
    }
  }

  return {
    localeOptions,
    currentLocale,
    isBusy,
    handleLocaleChange
  }
}
