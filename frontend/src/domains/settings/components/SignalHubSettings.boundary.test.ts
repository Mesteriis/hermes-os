import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('SignalHubSettings boundary', () => {
  it('keeps Signal Hub render shell under SettingsPage while preserving orchestration surfaces', () => {
    const page = readFileSync(
      new URL('../views/SettingsPage.vue', import.meta.url),
      'utf8'
    )
    const source = [
      readFileSync(new URL('../queries/useSignalHubSettingsSurface.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('./signalHubSettingsPresentation.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(existsSync(new URL('./SignalHubSettings.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubOperationsTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubProfilesPoliciesTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubSourcesTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubSettings.css', import.meta.url))).toBe(false)

    expect(page).not.toContain('import SignalHubSettings')
    expect(page).not.toContain('<SignalHubSettings')
    expect(page).toContain("store.selectedSection === 'signal-hub'")
    expect(page).toContain('Signal Hub contracts are preserved')
    expect(page).toContain('settings-note-panel')

    expect(source).toContain("./useSignalHubQuery")
    expect(source).toContain("../lib/signalHubReplay")
    expect(source).toContain("../types/signalHub")
    expect(source).not.toContain("useSignalHubSettingsController")
    expect(source).not.toContain("/integrations/")
    expect(source).not.toContain("../integrations/")
    expect(source).not.toContain("../../integrations/")
    expect(source).not.toContain("ApiClient")
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })

  it('keeps Signal Hub diagnostics in Settings-domain helpers after removing the Vue tabs', () => {
    const source = [
      readFileSync(new URL('./signalHubSettingsPresentation.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('../queries/useSignalHubSettingsSurface.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain('export function formatSettingsSummary')
    expect(source).toContain('export function formatRuntimeTimeline')
    expect(source).toContain('export function formatRuntimeError')
    expect(source).toContain('export function formatHealthStatus')
    expect(source).toContain('export function formatHealthEvidence')
    expect(source).toContain('sourceControlState(policies.value, source)')
    expect(source).toContain('useSignalHubCapabilitiesQuery')
    expect(source).toContain('selectedSourceCapabilities')
  })
})
