import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Communications settings boundary', () => {
  it('keeps mail reliability controls in Settings and uses persisted provider contracts', () => {
    const page = readFileSync(new URL('../views/SettingsPage.vue', import.meta.url), 'utf8')
    const settingsStore = readFileSync(new URL('../stores/settings.ts', import.meta.url), 'utf8')
    const pagePresentation = readFileSync(new URL('../queries/settingsPagePresentation.ts', import.meta.url), 'utf8')
    const surface = readFileSync(
      new URL('../queries/useCommunicationsSettingsSurface.ts', import.meta.url),
      'utf8'
    )
    const mailSyncActions = readFileSync(
      new URL('../queries/communicationsMailSyncActions.ts', import.meta.url),
      'utf8'
    )
    const contentEgressActions = readFileSync(
      new URL('../queries/communicationsContentEgressActions.ts', import.meta.url),
      'utf8'
    )
    const forwardingActions = readFileSync(
      new URL('../queries/communicationsForwardingActions.ts', import.meta.url),
      'utf8'
    )
    const resourceActions = readFileSync(
      new URL('../queries/communicationsResourceMappingActions.ts', import.meta.url),
      'utf8'
    )
    const forwardingPresentation = readFileSync(
      new URL('../queries/communicationsForwardingPresentation.ts', import.meta.url),
      'utf8'
    )
    const panel = readFileSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url), 'utf8')
    const panelPresentation = readFileSync(new URL('./communicationsSettingsPanelPresentation.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./CommunicationsSettingsPanel.vue', import.meta.url))).toBe(true)
    expect(settingsStore).toContain("'communications'")
    expect(pagePresentation).toContain("id: 'communications'")
    expect(page).toContain('import CommunicationsSettingsPanel')
    expect(page).toContain("selectedSection === 'communications'")
    expect(page).toContain(':surface="communicationsSettings"')
    expect(surface).toContain('communications.mail.consecutive_failures_before_degraded')
    expect(surface).toContain('communications.telegram.read_receipt_reports_enabled')
    expect(surface).toContain('updateTelegramReadReceiptReports')
    expect(surface).toContain('useMailSyncSettingsQuery')
    expect(surface).toContain('useUpdateMailSyncSettingsMutation')
    expect(surface).toContain('useSyncStatusesQuery')
    expect(surface).toContain('mailSyncActionDependencies')
    expect(mailSyncActions).toContain('saveMailSyncSettings')
    expect(mailSyncActions).toContain('toggleMailSync')
    expect(mailSyncActions).toContain('Number.parseInt')
    expect(contentEgressActions).toContain('updateMailContentEgress')
    expect(contentEgressActions).toContain('Mail content access preference saved')
    expect(forwardingActions).toContain('saveSensitiveForwardingPolicyAction')
    expect(forwardingActions).toContain('deleteSensitiveForwardingPolicyAction')
    expect(resourceActions).toContain('saveProviderResourceMappingAction')
    expect(resourceActions).toContain('resource.writable')
    expect(forwardingPresentation).toContain('newSensitiveForwardingPolicyDraft')
    expect(forwardingPresentation).toContain('sensitiveForwardingPolicyInput')
    expect(panel).toContain("t('Mail')")
    expect(panel).toContain("t('Failures before degradation')")
    expect(panel).toContain("t('Send read reports to Telegram')")
    expect(panel).toContain('Telegram delivery receipts are provider-managed')
    expect(panelPresentation).toContain('parseSemanticRole')
    expect(panelPresentation).toContain('commandStatusCount')
    expect(panel).toContain('useCommunicationsSettingsPanelController')
    expect(panel).toContain('handleDegradationThresholdInput')
    expect(panel).toContain('handleSelectedMailSyncToggle')
    expect(panel).toContain('handleContentEgressBodyToggle')
    expect(panel).toContain('handlePolicySelection')
    expect(panel).toContain('handleRecipientInput')
    expect(panel).toContain('handleResourceRoleInput')
    expect(panel).toContain('handleResourceLocalFolderInput')
    expect(panel).toContain('handleRefreshCommandDiagnostics')
    expect(panel).toContain('handleRetryCommand')
    expect(panel).toContain('handleSaveDegradationThreshold')
    expect(panel).toContain('handleSaveSelectedMailSyncSettings')
    expect(panel).not.toContain('eventValue(')
    expect(panel).not.toContain('eventChecked(')
    expect(panel).not.toContain('surface.batchSizeDraft.value =')
    expect(panel).not.toContain('surface.updateSensitiveForwardingDraft({')
    expect(panel).not.toContain('surface.saveDegradationThreshold(')
    expect(panel).not.toContain('surface.saveSelectedMailSyncSettings(')
  })
})
