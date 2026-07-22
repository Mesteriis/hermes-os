import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('LanguageSettings boundary', () => {
  it('renders locale choices through SettingsPage while keeping persistence in the surface', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(
      new URL('./LanguageSettingsPanel.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useLanguageSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./LanguageSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(page).toContain('import LanguageSettingsPanel')
    expect(page).toContain('<LanguageSettingsPanel')
    expect(page).toContain("selectedSection === 'language'")
    expect(page).toContain(':surface="languageSettings"')
    expect(panel).toContain('useLanguageSettingsPanelController')
    expect(panel).toContain('handleLocaleSelection')
    expect(panel).not.toContain('handleLocaleChange(localeOption.value)')

    expect(surface).toContain('useSaveFrontendLocaleMutation')
    expect(surface).toContain('localeOptions')
    expect(surface).toContain('handleLocaleChange')
    expect(surface).toContain('setLocale')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('saveApplicationSetting')
    expect(surface).not.toContain('FRONTEND_LOCALE_SETTING_KEY')
    expect(surface).not.toContain('queryClient')
    expect(surface).not.toContain('fetch(')
  })
})
