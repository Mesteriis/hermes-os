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
      readFileSync(new URL('../queries/signalHubSettingsSelectors.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('./signalHubRoutePresentation.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('./signalHubSettingsPresentation.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(existsSync(new URL('./SignalHubSettings.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubOperationsTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubProfilesPoliciesTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubSourcesTab.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./SignalHubSettings.css', import.meta.url))).toBe(false)

    expect(existsSync(new URL('./SignalHubSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(page).toContain('import SignalHubSettingsPanel')
    expect(page).toContain('<SignalHubSettingsPanel')
    expect(page).toContain("selectedSection === 'signal-hub'")
    expect(page).toContain(':surface="signalHubSettings"')

    expect(source).toContain("./useSignalHubQuery")
    expect(source).toContain("../lib/signalHubReplay")
    expect(source).toContain("../types/signalHub")
    expect(source).toContain("buildSignalConsumerGraphRoute")
    expect(source).toContain("buildSignalGraphTabs")
    expect(source).toContain("buildSignalInventoryTabs")
    expect(source).toContain("buildSignalInventoryRow")
    expect(source).not.toContain("useSignalHubSettingsController")
    expect(source).not.toContain('/integrations/')
    expect(source).not.toContain('../integrations/')
    expect(source).not.toContain('../../integrations/')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })

  it('delegates Signal Hub UI from SettingsPage into SignalHubSettingsPanel', () => {
    const panel = readFileSync(
      new URL('./SignalHubSettingsPanel.vue', import.meta.url),
      'utf8'
    )
    const panelController = readFileSync(
      new URL('../queries/useSignalHubSettingsPanelController.ts', import.meta.url),
      'utf8'
    )

    expect(panel).toContain('settings-signal-view-tabs')
    expect(panel).toContain('settings-signal-category-tabs')
    expect(panel).toContain('settings-signal-graph')
    expect(panel).toContain('settings-signal-table')
    expect(panel).toContain('surface.filteredSignalConsumerGraph')
    expect(panel).toContain('surface.filteredSignalInventoryRows')
    expect(panel).toContain('surface.signalInventoryRows')
    expect(panel).toContain('handleSelectGraphSource')
    expect(panel).toContain('handleSelectInventorySource')
    expect(panel).toContain('handleSelectSignalView')
    expect(panel).toContain('handlePauseSourceSignals')
    expect(panel).toContain('handleMuteSourceSignals')
    expect(panelController).toContain('activeSignalViewPresentation')
    expect(panelController).toContain('signalHubViewPresentation')
  })

  it('keeps Signal Hub diagnostics in Settings-domain helpers after removing the Vue tabs', () => {
    const source = [
      readFileSync(new URL('./signalHubSettingsPresentation.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('./signalHubRoutePresentation.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('../queries/signalHubSettingsSelectors.ts', import.meta.url), 'utf8'),
      readFileSync(new URL('../queries/useSignalHubSettingsSurface.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain('export function formatSettingsSummary')
    expect(source).toContain('export function formatRuntimeTimeline')
    expect(source).toContain('export function formatRuntimeError')
    expect(source).toContain('export function formatHealthStatus')
    expect(source).toContain('export function formatHealthEvidence')
    expect(source).toContain('countRunningSources(sources.value, policies.value)')
    expect(source).toContain('signalConsumerGraph')
    expect(source).toContain('filteredSignalConsumerGraph')
    expect(source).toContain('filteredSignalInventoryRows')
    expect(source).toContain('graphSourceTabs')
    expect(source).toContain('inventorySourceTabs')
    expect(source).toContain('signalViewTabs')
    expect(source).toContain('signalInventoryRows')
    expect(source).toContain('useSignalHubCapabilitiesQuery')
    expect(source).toContain('selectedSourceCapabilities')
  })
})
