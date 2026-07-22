import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('SettingsPage Signal Hub boundary', () => {
  it('keeps Signal Hub under Settings navigation and delegates UI to dedicated panel', () => {
    const pageSource = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useSettingsPageSurface.ts', import.meta.url), 'utf8')
    const presentationSource = readFileSync(new URL('../queries/settingsPagePresentation.ts', import.meta.url), 'utf8')

    expect(presentationSource).toContain("id: 'signal-hub'")
    expect(presentationSource).toContain("label: 'Signal Hub'")
    expect(existsSync(new URL('../components/SignalHubSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(pageSource).toContain('import SignalHubSettingsPanel')
    expect(pageSource).toContain('<SignalHubSettingsPanel')
    expect(pageSource).toContain("selectedSection === 'signal-hub'")
    expect(pageSource).toContain(':surface="signalHubSettings"')
    expect(pageSource).not.toContain('settings-signal-view-tabs')
    expect(pageSource).not.toContain('settings-signal-category-tabs')
    expect(pageSource).not.toContain('settings-signal-graph')
    expect(pageSource).not.toContain('settings-signal-table')
    expect(pageSource).not.toContain('handlePauseSourceSignals')
    expect(pageSource).not.toContain('handleSelectGraphSource')
    expect(pageSource).not.toContain('handleSelectInventorySource')
    expect(pageSource).not.toContain('handleSelectSignalView')
    expect(presentationSource).toContain("id: 'accounts'")
    expect(presentationSource).toContain("label: 'Accounts'")
    expect(presentationSource).not.toContain("id: 'integrations'")
    expect(presentationSource).not.toContain("label: 'Integrations'")
    expect(surfaceSource).not.toContain("path: '/signal-hub'")
    expect(surfaceSource).not.toContain("name: 'signal-hub'")
  })
})
