import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('BackgroundJobsSettings boundary', () => {
  it('keeps legacy Background Jobs controls outside the recovery/System Control shell', () => {
    const appRoot = readFileSync(new URL('../../../app/layout/AppLayoutRoot.vue', import.meta.url), 'utf8')
    const systemControl = readFileSync(new URL('../../../platform/system-control/SystemControlPage.vue', import.meta.url), 'utf8')
    const legacyPanel = readFileSync(new URL('./BackgroundJobsSettingsPanel.vue', import.meta.url), 'utf8')

    expect(appRoot).toContain('SystemControlPage')
    expect(appRoot).not.toContain('BackgroundJobsSettingsPanel')
    expect(systemControl).not.toContain('BackgroundJobsSettingsPanel')
    expect(systemControl).not.toMatch(/\bfetch\s*\(/)
    expect(systemControl).not.toContain('ApiClient')
    expect(legacyPanel).toContain('settings-background-job-list')
    expect(legacyPanel).toContain('useBackgroundJobsSettingsPanelController')
    expect(legacyPanel).toContain('handleRefresh')
    expect(legacyPanel).toContain('handleSelectJobFilter')
    expect(legacyPanel).toContain('handleOpenControl')
    expect(legacyPanel).not.toContain('surface.handleRefresh()')
    expect(legacyPanel).not.toContain('surface.handleSelectJobFilter(')
    expect(legacyPanel).not.toContain('surface.handleOpenControl(')
  })
})
