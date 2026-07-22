import type { Locale } from '../../../platform/i18n/types'

interface SaveLocaleDependencies {
  setLocale: (locale: Locale) => void
  saveLocale: (locale: Locale) => Promise<unknown>
  reportError: (error: unknown) => void
}

export async function saveLocaleWithRollback(
  nextLocale: Locale,
  previousLocale: Locale,
  dependencies: SaveLocaleDependencies
): Promise<void> {
  dependencies.setLocale(nextLocale)
  try {
    await dependencies.saveLocale(nextLocale)
  } catch (error) {
    dependencies.setLocale(previousLocale)
    dependencies.reportError(error)
  }
}
