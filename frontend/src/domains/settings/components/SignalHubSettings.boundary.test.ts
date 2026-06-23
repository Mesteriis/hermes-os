import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SignalHubSettings boundary', () => {
  it('uses Settings-local queries and does not import integration internals directly', () => {
    const source = [
      readFileSync(new URL('./SignalHubSettings.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./useSignalHubSettingsController.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain("../queries/useSignalHubQuery")
    expect(source).toContain("../lib/signalHubReplay")
    expect(source).toContain("../types/signalHub")
    expect(source).not.toContain("/integrations/")
    expect(source).not.toContain("../integrations/")
    expect(source).not.toContain("../../integrations/")
    expect(source).not.toContain("ApiClient")
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })

  it('renders Signal Hub diagnostics from Settings-domain data instead of flattening them away', () => {
    const source = [
      readFileSync(new URL('./SignalHubSettings.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./SignalHubOperationsTab.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./SignalHubSourcesTab.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./useSignalHubSettingsController.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain('formatSettingsSummary(t, connection)')
    expect(source).toContain('formatRuntimeTimeline(t, runtime)')
    expect(source).toContain('formatRuntimeError(t, runtime)')
    expect(source).toContain('formatHealthStatus(t, connections, item)')
    expect(source).toContain('formatHealthEvidence(t, item)')
    expect(source).toContain('sourceControlState(policies, source)')
    expect(source).toContain('useSignalHubCapabilitiesQuery')
    expect(source).toContain('selectedSourceCapabilities')
  })
})
