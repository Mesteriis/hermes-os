import { ApiClient } from '../client';
import type {
	CommunicationMessagesResponse,
	CommunicationMessageDetail,
	LocalMessageState,
	WorkflowState,
	WorkflowStateTransitionResponse,
	WorkflowStateCountsResponse,
	LocalMessageStateResponse,
	MailSyncSettings,
	MailSyncSettingsUpdate,
	MailSyncStatusListResponse,
	MailSyncRunResponse,
	MessageAnalyzeResponse,
	EmailSearchResponse,
	MailMessagesResponse,
	MailMessageDetailResponse,
	DraftListResponse,
	EmailDraft,
	DraftDeleteResponse,
	MailboxHealth,
	SenderStats,
	ThreadListResponse,
	ThreadMessagesResponse,
	SendEmailRequest,
	SendEmailResponse,
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
	ExtractTasksResponse,
	ExtractNotesResponse,
	SubscriptionSource,
	DuplicateAttachmentGroup,
	InvoiceListResponse,
	LegalDocumentListResponse,
	CertificateListResponse,
	EmailPersona,
	RichTemplateListResponse,
	RenderTemplateResponse,
	MailArchitectureBlocker,
	WorkflowActionRequest,
	WorkflowActionResponse
} from '../types';

export async function fetchCommunicationMessages(limit = 1000): Promise<CommunicationMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<CommunicationMessagesResponse>(
		`/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	);
}

export async function fetchCommunicationMessage(messageId: string): Promise<CommunicationMessageDetail> {
	return ApiClient.instance.get<CommunicationMessageDetail>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Communication message detail request failed'
	);
}

export async function transitionMessageWorkflowState(
	messageId: string,
	workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
	return ApiClient.instance.put<WorkflowStateTransitionResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/workflow-state`,
		{ workflow_state: workflowState },
		'Workflow state transition failed'
	);
}

export async function fetchMessageStateCounts(
	accountId?: string,
	localState?: LocalMessageState
): Promise<WorkflowStateCountsResponse> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (localState?.trim()) params.set('local_state', localState.trim());
	const qs = params.toString();
	return ApiClient.instance.get<WorkflowStateCountsResponse>(
		`/api/v1/communications/messages/states${qs ? '?' + qs : ''}`,
		'Message state counts request failed'
	);
}

export async function analyzeMessage(messageId: string): Promise<MessageAnalyzeResponse> {
	return ApiClient.instance.post<MessageAnalyzeResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/analyze`,
		{},
		'Message analysis failed'
	);
}

export async function runWorkflowAction(
	request: WorkflowActionRequest
): Promise<WorkflowActionResponse> {
	return ApiClient.instance.post<WorkflowActionResponse>(
		'/api/v1/workflow-actions',
		request,
		'Workflow action failed'
	);
}

export async function searchEmails(query: string, limit = 20): Promise<EmailSearchResponse> {
	const params = new URLSearchParams({ q: query, limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<EmailSearchResponse>(
		`/api/v1/communications/search?${params.toString()}`,
		'Email search failed'
	);
}

export async function fetchMailMessages(
	accountId?: string,
	workflowState?: WorkflowState,
	channelKind?: string,
	query?: string,
	localState?: LocalMessageState,
	limit = 1000
): Promise<MailMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (workflowState?.trim()) params.set('workflow_state', workflowState.trim());
	if (channelKind?.trim()) params.set('channel_kind', channelKind.trim());
	if (query?.trim()) params.set('q', query.trim());
	if (localState?.trim()) params.set('local_state', localState.trim());
	return ApiClient.instance.get<MailMessagesResponse>(
		`/api/v1/communications/messages?${params.toString()}`,
		'Mail messages request failed'
	);
}

export async function trashMessage(messageId: string): Promise<LocalMessageStateResponse> {
	return ApiClient.instance.post<LocalMessageStateResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/trash`,
		{},
		'Move message to trash failed'
	);
}

export async function restoreMessage(messageId: string): Promise<LocalMessageStateResponse> {
	return ApiClient.instance.post<LocalMessageStateResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/restore`,
		{},
		'Restore message failed'
	);
}

export async function fetchMailSyncStatus(): Promise<MailSyncStatusListResponse> {
	return ApiClient.instance.get<MailSyncStatusListResponse>(
		'/api/v1/email-accounts/sync-status',
		'Mail sync status request failed'
	);
}

export async function fetchMailSyncSettings(accountId: string): Promise<MailSyncSettings> {
	return ApiClient.instance.get<MailSyncSettings>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/sync-settings`,
		'Mail sync settings request failed'
	);
}

export async function updateMailSyncSettings(
	accountId: string,
	settings: MailSyncSettingsUpdate
): Promise<MailSyncSettings> {
	return ApiClient.instance.put<MailSyncSettings>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/sync-settings`,
		settings,
		'Mail sync settings update failed'
	);
}

export async function runMailSyncNow(accountId: string): Promise<MailSyncRunResponse> {
	return ApiClient.instance.post<MailSyncRunResponse>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/sync-now`,
		{},
		'Mail sync request failed'
	);
}

export async function runMailFullResync(accountId: string): Promise<MailSyncRunResponse> {
	return ApiClient.instance.post<MailSyncRunResponse>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/sync-full-resync`,
		{},
		'Mail full resync request failed'
	);
}

export async function sendEmail(request: SendEmailRequest): Promise<SendEmailResponse> {
	return ApiClient.instance.post<SendEmailResponse>(
		'/api/v1/communications/send',
		request,
		'Email send failed'
	);
}

export async function fetchMailMessage(messageId: string): Promise<MailMessageDetailResponse> {
	return ApiClient.instance.get<MailMessageDetailResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Mail message detail request failed'
	);
}

export async function fetchDrafts(accountId?: string, status?: string): Promise<DraftListResponse> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (status?.trim()) params.set('status', status.trim());
	const qs = params.toString();
	return ApiClient.instance.get<DraftListResponse>(
		`/api/v1/communications/drafts${qs ? '?' + qs : ''}`,
		'Drafts request failed'
	);
}

export async function createDraft(draft: Record<string, unknown>): Promise<EmailDraft> {
	return ApiClient.instance.post<EmailDraft>('/api/v1/communications/drafts', draft, 'Draft creation failed');
}

export async function deleteDraft(draftId: string): Promise<DraftDeleteResponse> {
	return ApiClient.instance.delete<DraftDeleteResponse>(
		`/api/v1/communications/drafts/${encodeURIComponent(draftId)}`,
		'Draft deletion failed'
	);
}

export async function fetchMailboxHealth(accountId?: string): Promise<MailboxHealth> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	const qs = params.toString();
	return ApiClient.instance.get<MailboxHealth>(
		`/api/v1/communications/analytics/health${qs ? '?' + qs : ''}`,
		'Health request failed'
	);
}

export async function fetchTopSenders(accountId?: string, limit = 20): Promise<SenderStats[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	return ApiClient.instance.get<SenderStats[]>(
		`/api/v1/communications/analytics/senders?${params.toString()}`,
		'Senders request failed'
	);
}

export async function fetchThreads(accountId?: string, limit = 50): Promise<ThreadListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	return ApiClient.instance.get<ThreadListResponse>(
		`/api/v1/communications/threads?${params.toString()}`,
		'Threads request failed'
	);
}

export async function fetchThreadMessages(
	accountId: string,
	subject: string,
	limit = 50
): Promise<ThreadMessagesResponse> {
	const params = new URLSearchParams({ account_id: accountId, subject, limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<ThreadMessagesResponse>(
		`/api/v1/communications/threads/messages?${params.toString()}`,
		'Thread messages failed'
	);
}

export async function fetchMessageExplain(messageId: string): Promise<MessageExplainResponse> {
	return ApiClient.instance.get<MessageExplainResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/explain`,
		'Message explain failed'
	);
}

export async function fetchMessageSmartCc(messageId: string): Promise<SmartCcResponse> {
	return ApiClient.instance.get<SmartCcResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/smart-cc`,
		'Smart CC request failed'
	);
}

export async function toggleMessagePin(messageId: string): Promise<MessagePinToggleResponse> {
	return ApiClient.instance.post<MessagePinToggleResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/pin`,
		{},
		'Pin message failed'
	);
}

export async function toggleMessageImportant(
	messageId: string
): Promise<MessageImportantToggleResponse> {
	return ApiClient.instance.post<MessageImportantToggleResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/important`,
		{},
		'Important message toggle failed'
	);
}

export async function toggleMessageMute(messageId: string): Promise<MessagePinToggleResponse> {
	return ApiClient.instance.post<MessagePinToggleResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/mute`,
		{},
		'Mute message failed'
	);
}

export async function snoozeMessage(messageId: string, until: string): Promise<Record<string, unknown>> {
	return ApiClient.instance.post<Record<string, unknown>>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/snooze`,
		{ until },
		'Snooze message failed'
	);
}

export async function addMessageLabel(messageId: string, label: string): Promise<Record<string, unknown>> {
	return ApiClient.instance.post<Record<string, unknown>>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/labels`,
		{ label },
		'Add message label failed'
	);
}

export async function exportMessage(
	messageId: string,
	format: 'md' | 'eml' | 'json'
): Promise<MessageExportResponse> {
	const params = new URLSearchParams({ format });
	return ApiClient.instance.get<MessageExportResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/export?${params.toString()}`,
		'Message export failed'
	);
}

export async function fetchMessageAuth(messageId: string): Promise<MessageAuthCheckResponse> {
	return ApiClient.instance.get<MessageAuthCheckResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/spf-dkim`,
		'Message authentication check failed'
	);
}

export async function fetchMessageSignature(messageId: string): Promise<SignatureDetection> {
	return ApiClient.instance.get<SignatureDetection>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/signature`,
		'Signature detection failed'
	);
}

export async function detectMessageLanguage(messageId: string): Promise<LanguageDetection> {
	return ApiClient.instance.get<LanguageDetection>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/detect-language`,
		'Language detection failed'
	);
}

export async function translateMessage(messageId: string, targetLanguage: string): Promise<TranslationResponse> {
	return ApiClient.instance.post<TranslationResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/translate`,
		{ target_language: targetLanguage },
		'Message translation failed'
	);
}

export async function generateAiReply(
	messageId: string,
	request: { tone?: string; language?: string; context?: string } = {}
): Promise<AiReplyResponse> {
	return ApiClient.instance.post<AiReplyResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/ai-reply`,
		request,
		'AI reply generation failed'
	);
}

export async function extractMessageTasks(messageId: string): Promise<ExtractTasksResponse> {
	return ApiClient.instance.post<ExtractTasksResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/extract-tasks`,
		{},
		'Task extraction failed'
	);
}

export async function extractMessageNotes(messageId: string): Promise<ExtractNotesResponse> {
	return ApiClient.instance.post<ExtractNotesResponse>(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/extract-notes`,
		{},
		'Note extraction failed'
	);
}

export async function fetchSubscriptions(accountId?: string, limit = 50): Promise<SubscriptionSource[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	return ApiClient.instance.get<SubscriptionSource[]>(
		`/api/v1/communications/subscriptions?${params.toString()}`,
		'Subscriptions request failed'
	);
}

export async function fetchAttachmentDuplicates(limit = 20): Promise<DuplicateAttachmentGroup[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<DuplicateAttachmentGroup[]>(
		`/api/v1/communications/attachments/duplicates?${params.toString()}`,
		'Attachment duplicate request failed'
	);
}

export async function fetchInvoices(status?: string): Promise<InvoiceListResponse> {
	const params = new URLSearchParams();
	if (status?.trim()) params.set('status', status.trim());
	const qs = params.toString();
	return ApiClient.instance.get<InvoiceListResponse>(
		`/api/v1/communications/finance/invoices${qs ? '?' + qs : ''}`,
		'Invoice request failed'
	);
}

export async function fetchLegalDocuments(
	documentType?: string,
	status?: string
): Promise<LegalDocumentListResponse> {
	const params = new URLSearchParams();
	if (documentType?.trim()) params.set('document_type', documentType.trim());
	if (status?.trim()) params.set('status', status.trim());
	const qs = params.toString();
	return ApiClient.instance.get<LegalDocumentListResponse>(
		`/api/v1/communications/legal${qs ? '?' + qs : ''}`,
		'Legal document request failed'
	);
}

export async function fetchCertificates(): Promise<CertificateListResponse> {
	return ApiClient.instance.get<CertificateListResponse>(
		'/api/v1/communications/certificates',
		'Certificate request failed'
	);
}

export async function fetchExpiringCertificates(days = 90): Promise<CertificateListResponse> {
	const params = new URLSearchParams({ days: String(Math.trunc(days)) });
	return ApiClient.instance.get<CertificateListResponse>(
		`/api/v1/communications/certificates/expiring?${params.toString()}`,
		'Expiring certificate request failed'
	);
}

export async function fetchPersonas(): Promise<{ items: EmailPersona[] }> {
	return ApiClient.instance.get<{ items: EmailPersona[] }>(
		'/api/v1/communications/personas',
		'Persona request failed'
	);
}

export async function fetchRichTemplates(): Promise<RichTemplateListResponse> {
	return ApiClient.instance.get<RichTemplateListResponse>(
		'/api/v1/communications/templates/rich',
		'Rich template request failed'
	);
}

export async function renderRichTemplate(
	templateId: string,
	variables: Record<string, string>
): Promise<RenderTemplateResponse> {
	return ApiClient.instance.post<RenderTemplateResponse>(
		'/api/v1/communications/templates/rich/render',
		{ template_id: templateId, variables },
		'Template render failed'
	);
}

export async function fetchMailBlockers(): Promise<MailArchitectureBlocker[]> {
	return ApiClient.instance.get<MailArchitectureBlocker[]>(
		'/api/v1/communications/blockers',
		'Mail blockers request failed'
	);
}
