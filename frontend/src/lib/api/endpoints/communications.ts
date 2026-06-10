import { ApiClient } from '../client';
import type {
	CommunicationMessagesResponse,
	CommunicationMessageDetail,
	WorkflowState,
	WorkflowStateTransitionResponse,
	WorkflowStateCountsResponse,
	MessageAnalyzeResponse,
	EmailSearchResponse,
	MailMessagesResponse,
	MailMessageDetailResponse,
	DraftListResponse,
	EmailDraft,
	MailboxHealth,
	SenderStats,
	ThreadListResponse,
	ThreadMessagesResponse
} from '../types';

export async function fetchCommunicationMessages(limit = 50): Promise<CommunicationMessagesResponse> {
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

export async function fetchMessageStateCounts(accountId?: string): Promise<WorkflowStateCountsResponse> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
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
	limit = 50
): Promise<MailMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (workflowState?.trim()) params.set('workflow_state', workflowState.trim());
	if (channelKind?.trim()) params.set('channel_kind', channelKind.trim());
	return ApiClient.instance.get<MailMessagesResponse>(
		`/api/v1/communications/messages?${params.toString()}`,
		'Mail messages request failed'
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
