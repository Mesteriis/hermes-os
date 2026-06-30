import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('ZoomSettingsPanel boundary', () => {
  it('removes the legacy Zoom settings render layer while preserving runtime and audit queries in TS', () => {
    const runtimeQuerySource = readFileSync(
      new URL('../queries/useZoomRuntimeQuery.ts', import.meta.url),
      'utf8'
    )
    const bridgeSource = readFileSync(
      new URL('../../../shared/zoom/settingsBridge.ts', import.meta.url),
      'utf8'
    )

    expect(existsSync(new URL('./ZoomSettingsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ZoomRecordingMaintenancePanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ZoomObservedCallsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ZoomRecordingImportsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('./ZoomAuditEventsPanel.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../../../shared/zoom/ZoomSettingsPanelShell.vue', import.meta.url))).toBe(false)

    expect(runtimeQuerySource).toContain('useStartZoomRuntimeMutation')
    expect(runtimeQuerySource).toContain('useStopZoomRuntimeMutation')
    expect(runtimeQuerySource).toContain('useRemoveZoomRuntimeMutation')
    expect(runtimeQuerySource).toContain('useZoomCapabilitiesQuery')
    expect(runtimeQuerySource).toContain('useZoomProviderCallsQuery')
    expect(runtimeQuerySource).toContain('useZoomCallTranscriptQuery')
    expect(runtimeQuerySource).toContain('useZoomRecordingImportsQuery')
    expect(runtimeQuerySource).toContain('useRemoveZoomRecordingImportMutation')
    expect(runtimeQuerySource).toContain('useCleanupZoomRetentionMutation')
    expect(runtimeQuerySource).toContain('useSyncZoomRecordingsMutation')
    expect(runtimeQuerySource).toContain('useZoomAuditEventsQuery')
    expect(runtimeQuerySource).toContain('settingsKeys.workspace()')
    expect(runtimeQuerySource).toContain('invalidateZoomDerived')
    expect(runtimeQuerySource).not.toContain('.vue')
    expect(runtimeQuerySource).not.toContain('ZoomBridgeLab')
    expect(runtimeQuerySource).not.toContain('useSetupZoomSyntheticAccountMutation')

    expect(bridgeSource).toContain('useApplicationSettingsQuery')
    expect(bridgeSource).toContain('useSettingsStore')
  })
})
