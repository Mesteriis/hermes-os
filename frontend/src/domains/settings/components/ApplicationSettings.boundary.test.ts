import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ApplicationSettings boundary', () => {
  it('renders application settings through SettingsPage while keeping orchestration in the surface', () => {
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
    expect(page).toContain("store.selectedSection === 'application'")
    expect(page).toContain('settings-registry-row')
    expect(page).toContain('applicationSettings.handleSave')

    expect(surface).toContain('useApplicationSettingsQuery')
    expect(surface).toContain('useSaveApplicationSettingMutation')
    expect(surface).toContain('isPublicApplicationSetting')
    expect(surface).toContain("setting.category !== 'ai'")
    expect(surface).toContain("!setting.setting_key.startsWith('ai.')")
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
