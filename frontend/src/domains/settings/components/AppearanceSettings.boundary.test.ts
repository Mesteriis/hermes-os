import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('AppearanceSettings boundary', () => {
  it('does not expose appearance controls through SettingsPage', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const pageSurface = readFileSync(
      new URL('../queries/useSettingsPageSurface.ts', import.meta.url),
      'utf8'
    )
    const domainSurface = readFileSync(
      new URL('../queries/useSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const predicates = readFileSync(
      new URL('../queries/appearanceSettingsPredicates.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./AppearanceSettings.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/AccentPicker.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/AppearanceHeader.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/AppearanceLivePreview.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/BackgroundPicker.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/SpacingDensityControl.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./appearance/ThemeRangeControl.vue', import.meta.url))).toBe(false)

    expect(page).not.toContain('import AppearanceSettings')
    expect(page).not.toContain('<AppearanceSettings')
    expect(page).not.toContain("selectedSection === 'appearance'")
    expect(page).not.toContain('updateShellBackground')
    expect(page).not.toContain('appearanceSettings.')

    expect(pageSurface).not.toContain('useAppearanceSettingsSurface')
    expect(pageSurface).not.toContain("label: 'Interface'")
    expect(pageSurface).not.toContain("id: 'appearance'")
    expect(domainSurface).not.toContain('settings-appearance')
    expect(domainSurface).not.toContain('settings-interface')
    expect(predicates).toContain('pickAllowedThemeNumber')
  })
})
