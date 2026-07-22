import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSaveFrontendLocaleMutation } from './useSettingsQuery'
import { saveLocaleWithRollback } from './languageSettingsActions'
import { isSupportedLocale, languageLocaleOptions } from './languageSettingsPresentation'

export function useLanguageSettingsSurface() {
  const { locale, setLocale } = useI18n()
  const saveLocale = useSaveFrontendLocaleMutation()

  const currentLocale = computed(() => locale.value)
  const isBusy = computed(() => saveLocale.isPending.value)

  async function handleLocaleChange(value: string) {
    if (!isSupportedLocale(value)) return

    const nextLocale = value
    const previousLocale = locale.value
    await saveLocaleWithRollback(nextLocale, previousLocale, {
      setLocale,
      saveLocale: (next) => saveLocale.mutateAsync(next),
      reportError: (error) => console.error('Failed to save locale setting:', error)
    })
  }

  return {
    localeOptions: languageLocaleOptions,
    currentLocale,
    isBusy,
    handleLocaleChange
  }
}
