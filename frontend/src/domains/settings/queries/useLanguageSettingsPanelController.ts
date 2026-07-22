import type { useLanguageSettingsSurface } from './useLanguageSettingsSurface'

type LanguageSettingsSurface = ReturnType<typeof useLanguageSettingsSurface>

export function useLanguageSettingsPanelController(options: {
  surface: LanguageSettingsSurface
}) {
  function handleLocaleSelection(locale: string): void {
    options.surface.handleLocaleChange(locale)
  }

  return {
    localeOptions: options.surface.localeOptions,
    currentLocale: options.surface.currentLocale,
    isBusy: options.surface.isBusy,
    handleLocaleSelection,
  }
}
