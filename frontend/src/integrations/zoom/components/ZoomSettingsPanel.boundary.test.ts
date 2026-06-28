import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomSettingsPanel boundary', () => {
  it('wires the full Zoom setup and credential maintenance flow through query mutations', () => {
    const source = readFileSync(new URL('./ZoomSettingsPanel.vue', import.meta.url), 'utf8')
    const recordingMaintenanceSource = readFileSync(
      new URL('./ZoomRecordingMaintenancePanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useStartZoomOAuthMutation')
    expect(source).toContain('useCompleteZoomOAuthMutation')
    expect(source).toContain('useAuthorizeZoomServerToServerMutation')
    expect(source).toContain('useRefreshZoomTokenMutation')
    expect(source).toContain('useMaintainZoomTokensMutation')
    expect(source).toContain('useZoomCapabilitiesQuery')
    expect(source).toContain('token_rotation_policy')
    expect(source).toContain('planned_features')
    expect(source).toContain("import ZoomAuditEventsPanel from './ZoomAuditEventsPanel.vue'")
    expect(source).toContain("import ZoomBridgeLab from './ZoomBridgeLab.vue'")
    expect(source).toContain("import ZoomRecordingMaintenancePanel from './ZoomRecordingMaintenancePanel.vue'")
    expect(source).toContain("import ZoomObservedCallsPanel from './ZoomObservedCallsPanel.vue'")
    expect(source).toContain("import ZoomRecordingImportsPanel from './ZoomRecordingImportsPanel.vue'")
    expect(source).toContain('<ZoomAuditEventsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomBridgeLab :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomRecordingMaintenancePanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomObservedCallsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomRecordingImportsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('window.open(response.authorization_url')
    expect(recordingMaintenanceSource).toContain('useSyncZoomRecordingsMutation')
    expect(recordingMaintenanceSource).toContain('useCleanupZoomRetentionMutation')
    expect(recordingMaintenanceSource).toContain('Manual recording sync')
    expect(recordingMaintenanceSource).toContain('handleSyncZoomRecordings')
    expect(recordingMaintenanceSource).toContain('Sync cloud recordings')
    expect(recordingMaintenanceSource).toContain('handleCleanupZoomRetention')
    expect(recordingMaintenanceSource).toContain('Run retention cleanup')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_remote_transcript_download_enabled')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_remote_recording_download_enabled')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_recording_import_retention_days')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_transcript_retention_days')
    expect(source).not.toContain('fetch(')
    expect(recordingMaintenanceSource).not.toContain('fetch(')
  })
})
