import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('LanguageSettings boundary', () => {
  it('preserves locale orchestration after removing the Vue render layer', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useLanguageSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./LanguageSettings.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import LanguageSettings')
    expect(page).not.toContain('<LanguageSettings')
    expect(page).toContain('Language UI removed after logic extraction. Rebuild pending new design language.')
    expect(page).toContain('Language logic is preserved')

    expect(surface).toContain('useSaveFrontendLocaleMutation')
    expect(surface).toContain('localeOptions')
    expect(surface).toContain('handleLocaleChange')
    expect(surface).toContain('setLocale')
    expect(surface).not.toContain('useLanguageSettingsController')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('saveApplicationSetting')
    expect(surface).not.toContain('FRONTEND_LOCALE_SETTING_KEY')
    expect(surface).not.toContain('queryClient')
    expect(surface).not.toContain('fetch(')
  })
})
