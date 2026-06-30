import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ApplicationSettings boundary', () => {
  it('preserves application settings orchestration after removing the Vue render layer', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useApplicationSettingsSurface.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ApplicationSettings.vue', import.meta.url))).toBe(false)
    expect(page).not.toContain('import ApplicationSettings')
    expect(page).not.toContain('<ApplicationSettings')
    expect(page).toContain('Application settings UI removed after logic extraction. Rebuild pending new design language.')
    expect(page).toContain('Application settings logic is preserved')

    expect(surface).toContain('useApplicationSettingsQuery')
    expect(surface).toContain('useSaveApplicationSettingMutation')
    expect(surface).toContain('groupSettingsByCategory')
    expect(surface).toContain('settingControlType')
    expect(surface).toContain('settingAllowedValues')
    expect(surface).toContain('settingMetadataFlag')
    expect(surface).toContain('settingMetadataText')
    expect(surface).toContain('categoryLabel')
    expect(surface).toContain('handleSave')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('fetch(')
  })
})
