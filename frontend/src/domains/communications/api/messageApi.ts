import {
  analyzeMessageConnect,
  bulkMessageActionConnect,
  addMessageLabelConnect,
  createCommunicationDraftConnect,
  deleteMessageFromProviderConnect,
  detectMessageLanguageConnect,
  deleteCommunicationDraftConnect,
  extractMessageNotesConnect,
  extractMessageTasksConnect,
  fetchCommunicationMessageConnect,
  fetchCommunicationMessagesConnect,
  fetchCommunicationDraftsConnect,
  fetchCommunicationBlockersConnect,
  fetchCommunicationPersonasConnect,
  fetchMessageAuthConnect,
  fetchMessageExplainConnect,
  fetchMailboxHealthConnect,
  fetchMessageSignatureConnect,
  fetchMessageSmartCcConnect,
  fetchMessageStateCountsConnect,
  fetchSubscriptionsConnect,
  fetchTopSendersConnect,
  fetchRichTemplatesConnect,
  generateAiReplyConnect,
  generateAiReplyVariantsConnect,
  markMessageReadConnect,
  restoreMessageConnect,
  runWorkflowActionConnect,
  searchMessagesConnect,
  saveRichTemplateConnect,
  snoozeMessageConnect,
  trashMessageConnect,
  toggleMessageImportantConnect,
  toggleMessageMuteConnect,
  toggleMessagePinConnect,
  translateMessageConnect,
  transitionMessageWorkflowStateConnect,
  exportMessageConnect,
  deleteRichTemplateConnect,
  previewRichTemplateMailMergeConnect,
  renderRichTemplateConnect
} from './connectCommunications'
import type {
  CommunicationMessagesResponse,
  CommunicationMessageDetailResponse,
  WorkflowState,
  LocalMessageState,
  WorkflowStateCountsResponse,
  WorkflowStateTransitionResponse,
  LocalMessageStateResponse,
  MessageAnalyzeResponse,
  WorkflowActionRequest,
  WorkflowActionResponse,
  DraftListResponse,
  CommunicationDraft,
  DraftDeleteResponse,
  DraftUpsertRequest,
  MailboxHealth,
  SenderStatsListResponse,
  MessageExplainResponse,
  SmartCcResponse,
  MessagePinToggleResponse,
  MessageImportantToggleResponse,
  MessageExportResponse,
  MessageAuthCheckResponse,
  SignatureDetection,
  LanguageDetection,
  TranslationResponse,
  AiReplyResponse,
  AiReplyVariantsRequest,
  AiReplyVariantsResponse,
  ExtractTasksResponse,
  ExtractNotesResponse,
  CommunicationSearchResponse,
  SubscriptionListResponse,
  CommunicationArchitectureBlocker,
  CommunicationPersona,
  CommunicationTemplate,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse,
  BulkMessageActionRequest,
  BulkMessageActionResponse
} from '../types/communications'

export async function fetchCommunicationMessages(
  accountId?: string,
  workflowState?: WorkflowState,
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState,
  limit = 100,
  cursor?: string | null
): Promise<CommunicationMessagesResponse> {
  return fetchCommunicationMessagesConnect({
    account_id: accountId,
    workflow_state: workflowState,
    channel_kind: channelKind,
    query,
    local_state: localState,
    limit,
    cursor: cursor ?? undefined
  })
}

export async function fetchCommunicationMessage(messageId: string): Promise<CommunicationMessageDetailResponse> {
  return fetchCommunicationMessageConnect(messageId)
}

export async function transitionMessageWorkflowState(
  messageId: string,
  workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
  return transitionMessageWorkflowStateConnect(messageId, workflowState)
}

export async function fetchMessageStateCounts(
  accountId?: string,
  localState?: LocalMessageState
): Promise<WorkflowStateCountsResponse> {
  return fetchMessageStateCountsConnect(accountId, localState)
}

export async function trashMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return trashMessageConnect(messageId)
}

export async function restoreMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return restoreMessageConnect(messageId)
}

export async function markMessageRead(messageId: string): Promise<Record<string, unknown>> {
  return markMessageReadConnect(messageId)
}

export async function deleteMessageFromProvider(messageId: string): Promise<LocalMessageStateResponse> {
  return deleteMessageFromProviderConnect(messageId)
}

export async function bulkMessageAction(
  request: BulkMessageActionRequest
): Promise<BulkMessageActionResponse> {
  return bulkMessageActionConnect(request)
}

export async function analyzeMessage(messageId: string): Promise<MessageAnalyzeResponse> {
  return analyzeMessageConnect(messageId)
}

export async function runWorkflowAction(
  request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
  return runWorkflowActionConnect(request)
}

export async function searchEmails(query: string, limit = 20): Promise<CommunicationSearchResponse> {
  return searchMessagesConnect(query, limit)
}

export async function fetchDrafts(
  accountId?: string,
  status?: string,
  limit = 100,
  cursor?: string | null
): Promise<DraftListResponse> {
  return fetchCommunicationDraftsConnect(accountId, status, limit, cursor ?? undefined)
}

export async function createDraft(draft: DraftUpsertRequest): Promise<CommunicationDraft> {
  return createCommunicationDraftConnect(draft)
}

export async function deleteDraft(draftId: string): Promise<DraftDeleteResponse> {
  return deleteCommunicationDraftConnect(draftId)
}

export async function fetchMessageExplain(messageId: string): Promise<MessageExplainResponse> {
  return fetchMessageExplainConnect(messageId)
}

export async function fetchMessageSmartCc(messageId: string): Promise<SmartCcResponse> {
  return fetchMessageSmartCcConnect(messageId)
}

export async function toggleMessagePin(messageId: string): Promise<MessagePinToggleResponse> {
  return toggleMessagePinConnect(messageId)
}

export async function toggleMessageImportant(messageId: string): Promise<MessageImportantToggleResponse> {
  return toggleMessageImportantConnect(messageId)
}

export async function toggleMessageMute(messageId: string): Promise<MessagePinToggleResponse> {
  return toggleMessageMuteConnect(messageId)
}

export async function snoozeMessage(
  messageId: string,
  until: string
): Promise<Record<string, unknown>> {
  return snoozeMessageConnect(messageId, until)
}

export async function addMessageLabel(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return addMessageLabelConnect(messageId, label)
}

export async function exportMessage(
  messageId: string,
  format: 'md' | 'eml' | 'json'
): Promise<MessageExportResponse> {
  return exportMessageConnect(messageId, format)
}

export async function fetchMessageAuth(messageId: string): Promise<MessageAuthCheckResponse> {
  return fetchMessageAuthConnect(messageId)
}

export async function fetchMessageSignature(messageId: string): Promise<SignatureDetection> {
  return fetchMessageSignatureConnect(messageId)
}

export async function detectMessageLanguage(messageId: string): Promise<LanguageDetection> {
  return detectMessageLanguageConnect(messageId)
}

export async function translateMessage(
  messageId: string,
  targetLanguage: string
): Promise<TranslationResponse> {
  return translateMessageConnect(messageId, targetLanguage)
}

export async function generateAiReply(
  messageId: string,
  request: { tone?: string; language?: string; context?: string } = {}
): Promise<AiReplyResponse> {
  return generateAiReplyConnect(messageId, request)
}

export async function generateAiReplyVariants(
  messageId: string,
  request: AiReplyVariantsRequest = {}
): Promise<AiReplyVariantsResponse> {
  return generateAiReplyVariantsConnect(messageId, request)
}

export async function extractMessageTasks(messageId: string): Promise<ExtractTasksResponse> {
  return extractMessageTasksConnect(messageId)
}

export async function extractMessageNotes(messageId: string): Promise<ExtractNotesResponse> {
  return extractMessageNotesConnect(messageId)
}

export async function fetchSubscriptions(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<SubscriptionListResponse> {
  return fetchSubscriptionsConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchMailboxHealth(accountId?: string): Promise<MailboxHealth> {
  return fetchMailboxHealthConnect(accountId)
}

export async function fetchTopSenders(
  accountId?: string,
  limit = 20,
  cursor?: string | null
): Promise<SenderStatsListResponse> {
  return fetchTopSendersConnect(accountId, limit, cursor ?? undefined)
}

export async function fetchPersonas(): Promise<{ items: CommunicationPersona[] }> {
  return fetchCommunicationPersonasConnect()
}

export async function fetchRichTemplates(): Promise<{ templates: CommunicationTemplate[] }> {
  return fetchRichTemplatesConnect()
}

export async function saveRichTemplate(
  request: RichTemplateUpsertRequest
): Promise<RichTemplateUpsertResponse> {
  return saveRichTemplateConnect(request)
}

export async function deleteRichTemplate(templateId: string): Promise<RichTemplateDeleteResponse> {
  return deleteRichTemplateConnect(templateId)
}

export async function renderRichTemplate(
  request: RichTemplateRenderRequest
): Promise<RichTemplateRenderResponse> {
  return renderRichTemplateConnect(request)
}

export async function previewRichTemplateMailMerge(
  request: RichTemplateMailMergePreviewRequest
): Promise<RichTemplateMailMergePreviewResponse> {
  return previewRichTemplateMailMergeConnect(request)
}

export async function fetchCommunicationBlockers(): Promise<CommunicationArchitectureBlocker[]> {
  return fetchCommunicationBlockersConnect()
}
