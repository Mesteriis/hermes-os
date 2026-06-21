import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationMessagesResponse,
  CommunicationMessageDetailResponse,
  WorkflowState,
  LocalMessageState,
  WorkflowStateCountsResponse,
  WorkflowStateTransitionResponse,
  WorkflowStateTransitionRequest,
  LocalMessageStateResponse,
  MessageAnalyzeResponse,
  WorkflowActionRequest,
  WorkflowActionResponse,
  DraftListResponse,
  CommunicationDraft,
  DraftDeleteResponse,
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
  limit = 250,
  cursor?: string | null
): Promise<CommunicationMessagesResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (workflowState?.trim()) params.set('workflow_state', workflowState.trim())
  if (channelKind?.trim()) params.set('channel_kind', channelKind.trim())
  if (query?.trim()) params.set('q', query.trim())
  if (localState?.trim()) params.set('local_state', localState.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Mail messages request failed'
  )
}

export async function fetchCommunicationMessage(messageId: string): Promise<CommunicationMessageDetailResponse> {
  return ApiClient.instance.get<CommunicationMessageDetailResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
    'Mail message detail request failed'
  )
}

export async function transitionMessageWorkflowState(
  messageId: string,
  workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
  const body: WorkflowStateTransitionRequest = { workflow_state: workflowState }
  return ApiClient.instance.put<WorkflowStateTransitionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/workflow-state`,
    body,
    'Workflow state transition failed'
  )
}

export async function fetchMessageStateCounts(
  accountId?: string,
  localState?: LocalMessageState
): Promise<WorkflowStateCountsResponse> {
  const params = new URLSearchParams()
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (localState?.trim()) params.set('local_state', localState.trim())
  const qs = params.toString()
  return ApiClient.instance.get<WorkflowStateCountsResponse>(
    `/api/v1/communications/messages/states${qs ? `?${qs}` : ''}`,
    'Message state counts request failed'
  )
}

export async function trashMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return ApiClient.instance.post<LocalMessageStateResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/trash`,
    {},
    'Move message to trash failed'
  )
}

export async function restoreMessage(messageId: string): Promise<LocalMessageStateResponse> {
  return ApiClient.instance.post<LocalMessageStateResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/restore`,
    {},
    'Restore message failed'
  )
}

export async function markMessageRead(messageId: string): Promise<Record<string, unknown>> {
  return ApiClient.instance.post<Record<string, unknown>>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/imap-mark-read`,
    {},
    'Mark message as read failed'
  )
}

export async function deleteMessageFromProvider(messageId: string): Promise<LocalMessageStateResponse> {
  return ApiClient.instance.post<LocalMessageStateResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/imap-delete`,
    {},
    'Delete message from provider failed'
  )
}

export async function bulkMessageAction(
  request: BulkMessageActionRequest
): Promise<BulkMessageActionResponse> {
  return ApiClient.instance.post<BulkMessageActionResponse>(
    '/api/v1/communications/messages/bulk-actions',
    request,
    'Bulk message action failed'
  )
}

export async function analyzeMessage(messageId: string): Promise<MessageAnalyzeResponse> {
  return ApiClient.instance.post<MessageAnalyzeResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/analyze`,
    {},
    'Message analysis failed'
  )
}

export async function runWorkflowAction(
  request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
  return ApiClient.instance.post<WorkflowActionResponse>(
    '/api/v1/workflow-actions',
    request,
    'Workflow action failed'
  )
}

export async function searchEmails(query: string, limit = 20): Promise<CommunicationSearchResponse> {
  const params = new URLSearchParams({ q: query, limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<CommunicationSearchResponse>(
    `/api/v1/communications/search?${params.toString()}`,
    'Email search failed'
  )
}

export async function fetchDrafts(
  accountId?: string,
  status?: string,
  limit = 100,
  cursor?: string | null
): Promise<DraftListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (status?.trim()) params.set('status', status.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  const qs = params.toString()
  return ApiClient.instance.get<DraftListResponse>(
    `/api/v1/communications/drafts${qs ? `?${qs}` : ''}`,
    'Drafts request failed'
  )
}

export async function createDraft(draft: unknown): Promise<CommunicationDraft> {
  return ApiClient.instance.post<CommunicationDraft>(
    '/api/v1/communications/drafts',
    draft,
    'Draft creation failed'
  )
}

export async function deleteDraft(draftId: string): Promise<DraftDeleteResponse> {
  return ApiClient.instance.delete<DraftDeleteResponse>(
    `/api/v1/communications/drafts/${encodeURIComponent(draftId)}`,
    'Draft deletion failed'
  )
}

export async function fetchMessageExplain(messageId: string): Promise<MessageExplainResponse> {
  return ApiClient.instance.get<MessageExplainResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/explain`,
    'Message explain failed'
  )
}

export async function fetchMessageSmartCc(messageId: string): Promise<SmartCcResponse> {
  return ApiClient.instance.get<SmartCcResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/smart-cc`,
    'Smart CC request failed'
  )
}

export async function toggleMessagePin(messageId: string): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/pin`,
    {},
    'Pin message failed'
  )
}

export async function toggleMessageImportant(messageId: string): Promise<MessageImportantToggleResponse> {
  return ApiClient.instance.post<MessageImportantToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/important`,
    {},
    'Important message toggle failed'
  )
}

export async function toggleMessageMute(messageId: string): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/mute`,
    {},
    'Mute message failed'
  )
}

export async function snoozeMessage(
  messageId: string,
  until: string
): Promise<Record<string, unknown>> {
  return ApiClient.instance.post<Record<string, unknown>>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/snooze`,
    { until },
    'Snooze message failed'
  )
}

export async function addMessageLabel(
  messageId: string,
  label: string
): Promise<Record<string, unknown>> {
  return ApiClient.instance.post<Record<string, unknown>>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/labels`,
    { label },
    'Add message label failed'
  )
}

export async function exportMessage(
  messageId: string,
  format: 'md' | 'eml' | 'json'
): Promise<MessageExportResponse> {
  const params = new URLSearchParams({ format })
  return ApiClient.instance.get<MessageExportResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/export?${params.toString()}`,
    'Message export failed'
  )
}

export async function fetchMessageAuth(messageId: string): Promise<MessageAuthCheckResponse> {
  return ApiClient.instance.get<MessageAuthCheckResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/spf-dkim`,
    'Message authentication check failed'
  )
}

export async function fetchMessageSignature(messageId: string): Promise<SignatureDetection> {
  return ApiClient.instance.get<SignatureDetection>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/signature`,
    'Signature detection failed'
  )
}

export async function detectMessageLanguage(messageId: string): Promise<LanguageDetection> {
  return ApiClient.instance.get<LanguageDetection>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/detect-language`,
    'Language detection failed'
  )
}

export async function translateMessage(
  messageId: string,
  targetLanguage: string
): Promise<TranslationResponse> {
  return ApiClient.instance.post<TranslationResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/translate`,
    { target_language: targetLanguage },
    'Message translation failed'
  )
}

export async function generateAiReply(
  messageId: string,
  request: { tone?: string; language?: string; context?: string } = {}
): Promise<AiReplyResponse> {
  return ApiClient.instance.post<AiReplyResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-reply`,
    request,
    'AI reply generation failed'
  )
}

export async function generateAiReplyVariants(
  messageId: string,
  request: AiReplyVariantsRequest = {}
): Promise<AiReplyVariantsResponse> {
  return ApiClient.instance.post<AiReplyVariantsResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-reply-variants`,
    request,
    'AI reply variants generation failed'
  )
}

export async function extractMessageTasks(messageId: string): Promise<ExtractTasksResponse> {
  return ApiClient.instance.post<ExtractTasksResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/extract-tasks`,
    {},
    'Task extraction failed'
  )
}

export async function extractMessageNotes(messageId: string): Promise<ExtractNotesResponse> {
  return ApiClient.instance.post<ExtractNotesResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/extract-notes`,
    {},
    'Note extraction failed'
  )
}

export async function fetchSubscriptions(
  accountId?: string,
  limit = 50,
  cursor?: string | null
): Promise<SubscriptionListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<SubscriptionListResponse>(
    `/api/v1/communications/subscriptions?${params.toString()}`,
    'Subscriptions request failed'
  )
}

export async function fetchMailboxHealth(accountId?: string): Promise<MailboxHealth> {
  const params = new URLSearchParams()
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  const qs = params.toString()
  return ApiClient.instance.get<MailboxHealth>(
    `/api/v1/communications/analytics/health${qs ? `?${qs}` : ''}`,
    'Health request failed'
  )
}

export async function fetchTopSenders(
  accountId?: string,
  limit = 20,
  cursor?: string | null
): Promise<SenderStatsListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<SenderStatsListResponse>(
    `/api/v1/communications/analytics/senders?${params.toString()}`,
    'Senders request failed'
  )
}

export async function fetchPersonas(): Promise<{ items: CommunicationPersona[] }> {
  return ApiClient.instance.get<{ items: CommunicationPersona[] }>(
    '/api/v1/communications/personas',
    'Persona request failed'
  )
}

export async function fetchRichTemplates(): Promise<{ templates: CommunicationTemplate[] }> {
  return ApiClient.instance.get<{ templates: CommunicationTemplate[] }>(
    '/api/v1/communications/templates/rich',
    'Rich template request failed'
  )
}

export async function saveRichTemplate(
  request: RichTemplateUpsertRequest
): Promise<RichTemplateUpsertResponse> {
  return ApiClient.instance.post<RichTemplateUpsertResponse>(
    '/api/v1/communications/templates/rich',
    request,
    'Rich template save failed'
  )
}

export async function deleteRichTemplate(templateId: string): Promise<RichTemplateDeleteResponse> {
  return ApiClient.instance.delete<RichTemplateDeleteResponse>(
    `/api/v1/communications/templates/rich/${encodeURIComponent(templateId)}`,
    'Rich template delete failed'
  )
}

export async function renderRichTemplate(
  request: RichTemplateRenderRequest
): Promise<RichTemplateRenderResponse> {
  return ApiClient.instance.post<RichTemplateRenderResponse>(
    '/api/v1/communications/templates/rich/render',
    request,
    'Rich template render failed'
  )
}

export async function previewRichTemplateMailMerge(
  request: RichTemplateMailMergePreviewRequest
): Promise<RichTemplateMailMergePreviewResponse> {
  return ApiClient.instance.post<RichTemplateMailMergePreviewResponse>(
    '/api/v1/communications/templates/rich/mail-merge-preview',
    request,
    'Rich template mail merge preview failed'
  )
}

export async function fetchCommunicationBlockers(): Promise<CommunicationArchitectureBlocker[]> {
  return ApiClient.instance.get<CommunicationArchitectureBlocker[]>(
    '/api/v1/communications/blockers',
    'Mail blockers request failed'
  )
}
