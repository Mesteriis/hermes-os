import type { CommunicationsSettingsSurface } from './useCommunicationsSettingsSurface'
import {
  nullableLocalFolder,
  parseForwardingSeverity,
  parseSemanticRole,
} from '../components/communicationsSettingsPanelPresentation'

export function useCommunicationsSettingsPanelController(
  options: { surface: CommunicationsSettingsSurface },
) {
  const surface = options.surface

  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement
      || event.target instanceof HTMLSelectElement
      || event.target instanceof HTMLTextAreaElement
      ? event.target.value
      : ''
  }

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleDegradationThresholdInput(event: Event): void {
    surface.updateDegradationThreshold(eventValue(event))
  }

  function handleTelegramReadReceiptReportsChange(event: Event): void {
    void surface.updateTelegramReadReceiptReports(eventChecked(event))
  }

  function handleSelectMailAccount(accountId: string): void {
    surface.selectMailAccount(accountId)
  }

  function handleSelectedMailSyncToggle(event: Event): void {
    void surface.toggleSelectedMailSync(eventChecked(event))
  }

  function handleBatchSizeDraftInput(event: Event): void {
    surface.batchSizeDraft.value = eventValue(event)
  }

  function handlePollIntervalDraftInput(event: Event): void {
    surface.pollIntervalDraft.value = eventValue(event)
  }

  function handleWindowsDraftInput(event: Event): void {
    surface.windowsDraft.value = eventValue(event)
  }

  function handleContentEgressBodyToggle(event: Event): void {
    void surface.updateSelectedMailContentEgress('body', eventChecked(event))
  }

  function handleContentEgressAttachmentsToggle(event: Event): void {
    void surface.updateSelectedMailContentEgress('attachments', eventChecked(event))
  }

  function handleContentEgressExtractedTextToggle(event: Event): void {
    void surface.updateSelectedMailContentEgress('extracted_text', eventChecked(event))
  }

  function handlePolicySelection(policyId: string): void {
    surface.selectSensitiveForwardingPolicy(policyId)
  }

  function handleNewPolicy(): void {
    surface.createSensitiveForwardingPolicy()
  }

  function handlePolicyNameInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ name: eventValue(event) })
  }

  function handleDeliveryAccountInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ delivery_account_id: eventValue(event) })
  }

  function handleRecipientInput(event: Event): void {
    surface.updateSensitiveForwardingRecipients(eventValue(event))
  }

  function handleSeverityInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ minimum_severity: parseForwardingSeverity(eventValue(event)) })
  }

  function handleMaxSendsInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ max_sends_per_hour: Number(eventValue(event)) })
  }

  function handleQuietHoursStartInput(event: Event): void {
    surface.updateSensitiveForwardingQuietHours(
      eventValue(event),
      surface.sensitiveForwardingQuietHour('end'),
    )
  }

  function handleQuietHoursEndInput(event: Event): void {
    surface.updateSensitiveForwardingQuietHours(
      surface.sensitiveForwardingQuietHour('start'),
      eventValue(event),
    )
  }

  function handleExpiryInput(event: Event): void {
    surface.updateSensitiveForwardingExpiry(eventValue(event))
  }

  function handlePolicyEnabledInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ enabled: eventChecked(event) })
  }

  function handleIncludeMessageBodyInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ include_message_body: eventChecked(event) })
  }

  function handleIncludeAttachmentsInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ include_attachments: eventChecked(event) })
  }

  function handleSubjectTemplateInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ subject_template: eventValue(event) })
  }

  function handleBodyTemplateInput(event: Event): void {
    surface.updateSensitiveForwardingDraft({ body_template: eventValue(event) })
  }

  function handleResourceRoleInput(
    resource: Parameters<typeof surface.updateProviderResourceRole>[0],
    event: Event
  ): void {
    surface.updateProviderResourceRole(resource, parseSemanticRole(eventValue(event)))
  }

  function handleResourceLocalFolderInput(
    resource: Parameters<typeof surface.updateProviderResourceLocalFolder>[0],
    event: Event
  ): void {
    surface.updateProviderResourceLocalFolder(resource, nullableLocalFolder(eventValue(event)))
  }

  function handleRefreshCommandDiagnostics(): void {
    void surface.refreshCommandDiagnostics()
  }

  function handleRetryCommand(commandId: string): void {
    void surface.retryMailProviderCommand(commandId)
  }

  return {
    handleDegradationThresholdInput,
    handleTelegramReadReceiptReportsChange,
    handleSelectMailAccount,
    handleSelectedMailSyncToggle,
    handleBatchSizeDraftInput,
    handlePollIntervalDraftInput,
    handleWindowsDraftInput,
    handleContentEgressBodyToggle,
    handleContentEgressAttachmentsToggle,
    handleContentEgressExtractedTextToggle,
    handlePolicySelection,
    handleNewPolicy,
    handlePolicyNameInput,
    handleDeliveryAccountInput,
    handleRecipientInput,
    handleSeverityInput,
    handleMaxSendsInput,
    handleQuietHoursStartInput,
    handleQuietHoursEndInput,
    handleExpiryInput,
    handlePolicyEnabledInput,
    handleIncludeMessageBodyInput,
    handleIncludeAttachmentsInput,
    handleSubjectTemplateInput,
    handleBodyTemplateInput,
    handleResourceRoleInput,
    handleResourceLocalFolderInput,
    handleRefreshCommandDiagnostics,
    handleRetryCommand,
    handleSaveDegradationThreshold: surface.saveDegradationThreshold,
    handleSaveSelectedMailSyncSettings: surface.saveSelectedMailSyncSettings,
    handleSaveSensitiveForwardingPolicy: surface.saveSensitiveForwardingPolicy,
    handleRemoveSelectedSensitiveForwardingPolicy: surface.removeSelectedSensitiveForwardingPolicy,
  }
}
