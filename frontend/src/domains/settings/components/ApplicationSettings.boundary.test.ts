import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ApplicationSettings boundary', () => {
  it('renders application settings through SettingsPage while keeping orchestration in the surface', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(
      new URL('./ApplicationSettingsPanel.vue', import.meta.url),
      'utf8'
    )
    const panelController = readFileSync(
      new URL('../queries/useApplicationSettingsPanelController.ts', import.meta.url),
      'utf8'
    )
    const surface = readFileSync(
      new URL('../queries/useApplicationSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const predicates = readFileSync(
      new URL('../queries/applicationSettingsPredicates.ts', import.meta.url),
      'utf8'
    )
    const presentation = readFileSync(
      new URL('../queries/applicationSettingsPresentation.ts', import.meta.url),
      'utf8'
    )
    const values = readFileSync(
      new URL('../queries/applicationSettingsValue.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ApplicationSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(page).toContain('import ApplicationSettingsPanel')
    expect(page).toContain('<ApplicationSettingsPanel')
    expect(page).toContain("selectedSection === 'application'")
    expect(page).not.toContain('updateSettingDraft')
    expect(page).not.toContain('updateBooleanSettingDraft')
    expect(page).not.toContain('applicationSettings.handleSave')
    expect(page).not.toContain('settings-registry-row')

    expect(panel).toContain("v-else-if=\"setting.value_kind === 'json'\"")
    expect(panel).toContain('handleSaveSetting(setting)')
    expect(panel).toContain('settingsByCategory')
    expect(panel).toContain("v-for=\"(settings, category) in settingsByCategory\"")
    expect(panel).toContain('handleSettingInput(setting, $event)')
    expect(panel).toContain('handleSettingBooleanInput(setting, $event)')
    expect(panelController).toContain('eventValue')
    expect(panelController).toContain('eventChecked')
    expect(panelController).toContain('handleSettingInput')
    expect(panelController).toContain('handleSettingBooleanInput')
    expect(panelController).toContain('handleSaveSetting')

    expect(surface).toContain('useApplicationSettingsQuery')
    expect(surface).toContain('useSaveApplicationSettingMutation')
    expect(surface).toContain('isPublicApplicationSetting')
    expect(predicates).toContain("setting.category !== 'ai'")
    expect(predicates).toContain("!setting.setting_key.startsWith('ai.')")
    expect(surface).toContain('allApplicationSettings')
    expect(surface).toContain('groupSettingsByCategory')
    expect(surface).toContain('settingControlType')
    expect(surface).toContain('settingAllowedValues')
    expect(surface).toContain('settingMetadataFlag')
    expect(surface).toContain('settingMetadataText')
    expect(surface).toContain('categoryLabel')
    expect(presentation).toContain('settingControlType')
    expect(values).toContain('coerceApplicationSettingValue')
    expect(surface).toContain('handleSave')
    expect(surface).not.toContain('../api/')
    expect(surface).not.toContain('fetch(')
  })
})
