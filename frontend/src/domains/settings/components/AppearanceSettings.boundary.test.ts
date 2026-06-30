import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('AppearanceSettings boundary', () => {
  it('preserves appearance orchestration after removing the Vue render layer', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useAppearanceSettingsSurface.ts', import.meta.url),
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
    expect(page).toContain('Appearance UI removed after logic extraction. Rebuild pending new design language.')
    expect(page).toContain('Appearance logic is preserved')

    expect(surface).toContain('useThemeStore')
    expect(surface).toContain('theme.saveThemeSettings')
    expect(surface).toContain('backgroundBrightnessValues')
    expect(surface).toContain('panelOpacityValues')
    expect(surface).toContain('panelBlurValues')
    expect(surface).toContain('resetTheme')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('fetch(')
  })
})
