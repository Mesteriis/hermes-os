import type { CommunicationSectionId } from '$lib/layout';
import {
	fetchCommunicationMessages,
	fetchMailMessages,
	fetchMessageStateCounts,
	transitionMessageWorkflowState,
	trashMessage,
	restoreMessage,
	fetchMailSyncStatus,
	fetchMailSyncSettings,
	updateMailSyncSettings,
	runMailSyncNow,
	fetchDrafts,
	fetchMailboxHealth,
	fetchTopSenders,
	fetchThreads,
	analyzeMessage,
	createDraft,
	sendEmail,
	fetchCommunicationMessage,
	fetchMailMessage,
	fetchMessageExplain,
	fetchMessageSmartCc,
	fetchMessageAuth,
	fetchMessageSignature,
	detectMessageLanguage,
	translateMessage,
	generateAiReply,
	extractMessageTasks,
	extractMessageNotes,
	toggleMessagePin,
	toggleMessageMute,
	snoozeMessage,
	addMessageLabel,
	exportMessage,
	fetchSubscriptions,
	fetchAttachmentDuplicates,
	fetchInvoices,
	fetchLegalDocuments,
	fetchCertificates,
	fetchExpiringCertificates,
	fetchPersonas,
	fetchRichTemplates,
	fetchMailBlockers,
	runWorkflowAction,
	type CommunicationMessageSummary,
	type CommunicationMessageDetail,
	type CommunicationMessageSummaryV2,
	type WorkflowState,
	type LocalMessageState,
	type WorkflowStateCountItem,
	type EmailDraft,
	type MailSyncSettings,
	type MailSyncSettingsUpdate,
	type MailSyncStatus,
	type MailSyncRunResponse,
	type MailboxHealth,
	type SenderStats,
	type EmailThread,
	type MessageAnalyzeResponse,
	type CommunicationMessageDetailItem,
	type ProviderAccount,
	type SendEmailResponse,
	type MailMessageInsight,
	type MailResourceSnapshot,
	type MailResourceSummary,
	type MessageExportResponse,
	type WorkflowActionKind,
	type WorkflowActionRequest,
	type WorkflowActionResponse
} from '$lib/api';
import { formatDateTime } from './formatting';

export type ComposeMode = 'compose' | 'reply' | 'forward';

export type ComposeFormModel = {
	draft_id: string;
	account_id: string;
	to_text: string;
	cc_text: string;
	bcc_text: string;
	subject: string;
	body: string;
	mode: ComposeMode;
	in_reply_to: string | null;
	references: string[];
};

export type MailAccountOption = {
	accountId: string;
	providerKind: string;
	label: string;
	email: string;
	canSend: boolean;
	transport: 'smtp' | null;
	sendUnavailableReason: string | null;
};

export type SendCapability = {
	canSend: boolean;
	transport: 'smtp' | null;
	reason: string | null;
};

type DraftCreator = typeof createDraft;

export const COMMUNICATIONS_NAVIGATOR_LIMIT = 1000;

export const emptyMailResourceSnapshot: MailResourceSnapshot = {
	subscriptions: [],
	duplicates: [],
	invoices: [],
	legalDocuments: [],
	certificates: [],
	expiringCertificates: [],
	personas: [],
	templates: [],
	blockers: []
};

export async function loadCommunications(
	selectedConversationIndex: number
): Promise<{
	messages: CommunicationMessageSummary[];
	detail: CommunicationMessageDetail | null;
	error: string;
	isLoading: boolean;
	selectedIndex: number;
}> {
	try {
		const response = await fetchCommunicationMessages(COMMUNICATIONS_NAVIGATOR_LIMIT);
		const messages = response.items;
		let nextIndex = selectedConversationIndex;
		if (nextIndex >= messages.length) {
			nextIndex = 0;
		}
		let detail: CommunicationMessageDetail | null = null;
		if (messages.length > 0) {
			try {
				detail = await fetchCommunicationMessage(
					messages[nextIndex].message_id
				);
			} catch { /* detail optional */ }
		}
		return { messages, detail, error: '', isLoading: false, selectedIndex: nextIndex };
	} catch (error) {
		return {
			messages: [],
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown communications error',
			isLoading: false,
			selectedIndex: 0
		};
	}
}

export async function loadCommunicationMessagesFiltered(
	filterState?: WorkflowState,
	selectedConversationIndex?: number,
	accountId?: string,
	query?: string,
	localState: LocalMessageState = 'active'
): Promise<{
	messages: CommunicationMessageSummary[];
	detail: CommunicationMessageDetail | null;
	error: string;
	isLoading: boolean;
	selectedIndex: number;
}> {
	try {
		const response = await fetchMailMessages(
			accountId || undefined,
			filterState || undefined,
			undefined,
			query || undefined,
			localState,
			COMMUNICATIONS_NAVIGATOR_LIMIT
		);
		const messages = response.items as unknown as CommunicationMessageSummary[];
		let nextIndex = selectedConversationIndex ?? 0;
		if (nextIndex >= messages.length) {
			nextIndex = Math.max(0, messages.length - 1);
		}
		let detail: CommunicationMessageDetail | null = null;
		if (messages.length > 0) {
			try {
				detail = await fetchCommunicationMessage(
					messages[nextIndex].message_id
				);
			} catch { /* detail optional */ }
		}
		return { messages, detail, error: '', isLoading: false, selectedIndex: nextIndex };
	} catch (error) {
		return {
			messages: [],
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown communications error',
			isLoading: false,
			selectedIndex: 0
		};
	}
}

export async function loadMessageStateCounts(
	accountId?: string,
	localState: LocalMessageState = 'active'
): Promise<{ counts: WorkflowStateCountItem[]; error: string }> {
	try {
		const response = await fetchMessageStateCounts(accountId, localState);
		return { counts: response.counts ?? [], error: '' };
	} catch {
		return { counts: [], error: '' };
	}
}

export async function loadMailSyncStatuses(): Promise<{ statuses: MailSyncStatus[]; error: string }> {
	try {
		const response = await fetchMailSyncStatus();
		return { statuses: response.items ?? [], error: '' };
	} catch (error) {
		return {
			statuses: [],
			error: error instanceof Error ? error.message : 'Mail sync status failed'
		};
	}
}

export async function loadMailSyncSettings(
	accountId: string
): Promise<{ settings: MailSyncSettings | null; error: string }> {
	if (!accountId.trim()) {
		return { settings: null, error: '' };
	}
	try {
		return { settings: await fetchMailSyncSettings(accountId), error: '' };
	} catch (error) {
		return {
			settings: null,
			error: error instanceof Error ? error.message : 'Mail sync settings failed'
		};
	}
}

export async function saveMailSyncSettings(
	accountId: string,
	settings: MailSyncSettingsUpdate
): Promise<{ settings: MailSyncSettings | null; error: string }> {
	try {
		return { settings: await updateMailSyncSettings(accountId, settings), error: '' };
	} catch (error) {
		return {
			settings: null,
			error: error instanceof Error ? error.message : 'Mail sync settings update failed'
		};
	}
}

export async function triggerMailSyncNow(
	accountIds: string[]
): Promise<{ runs: MailSyncRunResponse[]; error: string }> {
	const targets = accountIds.map((accountId) => accountId.trim()).filter(Boolean);
	if (!targets.length) {
		return { runs: [], error: '' };
	}
	const runs: MailSyncRunResponse[] = [];
	try {
		for (const accountId of targets) {
			runs.push(await runMailSyncNow(accountId));
		}
		return { runs, error: '' };
	} catch (error) {
		return {
			runs,
			error: error instanceof Error ? error.message : 'Mail sync request failed'
		};
	}
}

export async function handleTrashMessage(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		await trashMessage(messageId);
		return { success: true, message: 'Moved to Trash' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Move message to trash failed' };
	}
}

export async function handleRestoreMessage(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		await restoreMessage(messageId);
		return { success: true, message: 'Restored from Trash' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Restore message failed' };
	}
}

export async function handleWorkflowStateTransition(
	messageId: string,
	newState: WorkflowState,
	mailStateFilter: WorkflowState | ''
): Promise<{ error: string }> {
	try {
		await transitionMessageWorkflowState(messageId, newState);
		// Side effects are handled by the caller reloading data
		return { error: '' };
	} catch (error) {
		return { error: error instanceof Error ? error.message : 'State transition failed' };
	}
}

export async function loadDrafts(accountId?: string): Promise<{ drafts: EmailDraft[] }> {
	try {
		const r = await fetchDrafts(accountId);
		return { drafts: r.items };
	} catch {
		return { drafts: [] };
	}
}

export async function loadMailboxHealth(accountId?: string): Promise<{ health: MailboxHealth | null }> {
	try {
		return { health: await fetchMailboxHealth(accountId) };
	} catch {
		return { health: null };
	}
}

export async function loadTopSenders(accountId?: string): Promise<{ senders: SenderStats[] }> {
	try {
		return { senders: await fetchTopSenders(accountId) };
	} catch {
		return { senders: [] };
	}
}

export async function loadThreads(accountId?: string): Promise<{ threads: EmailThread[] }> {
	try {
		const r = await fetchThreads(accountId);
		return { threads: r.items };
	} catch {
		return { threads: [] };
	}
}

export async function handleAnalyzeMessage(
	messageId: string
): Promise<{ result: MessageAnalyzeResponse | null; isAnalyzing: boolean }> {
	try {
		const result = await analyzeMessage(messageId);
		return { result, isAnalyzing: false };
	} catch {
		return { result: null, isAnalyzing: false };
	}
}

export async function handleSaveDraft(
	draft: ComposeFormModel,
	draftCreator: DraftCreator = createDraft
): Promise<{ success: boolean; error: string }> {
	if (!draft.draft_id || !draft.account_id) {
		return { success: false, error: 'Select a sending account before saving a draft' };
	}
	try {
		await draftCreator(buildComposeDraftPayload(draft));
		return { success: true, error: '' };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Draft save failed'
		};
	}
}

export function splitRecipients(value: string): string[] {
	return value
		.split(',')
		.map((recipient) => recipient.trim())
		.filter(Boolean);
}

export function buildComposeDraftPayload(draft: ComposeFormModel): Record<string, unknown> {
	return {
		draft_id: draft.draft_id,
		account_id: draft.account_id,
		to_recipients: splitRecipients(draft.to_text),
		cc_recipients: splitRecipients(draft.cc_text),
		bcc_recipients: splitRecipients(draft.bcc_text),
		subject: draft.subject,
		body_text: draft.body,
		in_reply_to: draft.in_reply_to,
		references: draft.references,
		status: 'draft',
		metadata: { compose_mode: draft.mode }
	};
}

export async function handleSendMessage(
	draft: ComposeFormModel
): Promise<{ success: boolean; error: string; result: SendEmailResponse | null }> {
	try {
		const result = await sendEmail({
			account_id: draft.account_id,
			to: splitRecipients(draft.to_text),
			cc: splitRecipients(draft.cc_text),
			bcc: splitRecipients(draft.bcc_text),
			subject: draft.subject,
			body_text: draft.body,
			in_reply_to: draft.in_reply_to,
			references: draft.references,
			confirmed_provider_write: true
		});
		return { success: true, error: '', result };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Email send failed',
			result: null
		};
	}
}

export async function loadMessageInsights(messageId: string): Promise<MailMessageInsight> {
	const [explain, smartCc, auth, signature, language] = await Promise.all([
		safe(() => fetchMessageExplain(messageId), null),
		safe(() => fetchMessageSmartCc(messageId), null),
		safe(() => fetchMessageAuth(messageId), null),
		safe(() => fetchMessageSignature(messageId), null),
		safe(() => detectMessageLanguage(messageId), null)
	]);
	return {
		messageId,
		explain,
		smartCc,
		auth,
		signature,
		language,
		aiReply: null,
		tasks: [],
		notes: [],
		translation: null
	};
}

export async function loadMailResources(accountId?: string): Promise<{
	resources: MailResourceSnapshot;
	summary: MailResourceSummary;
}> {
	const [
		subscriptions,
		duplicates,
		invoices,
		legalDocuments,
		certificates,
		expiringCertificates,
		personas,
		templates,
		blockers
	] = await Promise.all([
		safe(() => fetchSubscriptions(accountId), []),
		safe(() => fetchAttachmentDuplicates(), []),
		safe(async () => (await fetchInvoices()).items, []),
		safe(async () => (await fetchLegalDocuments()).items, []),
		safe(async () => (await fetchCertificates()).items, []),
		safe(async () => (await fetchExpiringCertificates()).items, []),
		safe(async () => (await fetchPersonas()).items, []),
		safe(async () => (await fetchRichTemplates()).templates ?? [], []),
		safe(() => fetchMailBlockers(), [])
	]);
	const resources: MailResourceSnapshot = {
		subscriptions,
		duplicates,
		invoices,
		legalDocuments,
		certificates,
		expiringCertificates,
		personas,
		templates,
		blockers
	};
	return { resources, summary: summarizeMailResourceSnapshot(resources) };
}

export function summarizeMailResourceSnapshot(resources: MailResourceSnapshot): MailResourceSummary {
	return {
		subscriptions: resources.subscriptions.length,
		duplicates: resources.duplicates.length,
		invoices: resources.invoices.length,
		legalDocuments: resources.legalDocuments.length,
		certificates: resources.certificates.length,
		expiringCertificates: resources.expiringCertificates.length,
		personas: resources.personas.length,
		templates: resources.templates.length,
		blockers: resources.blockers.length
	};
}

export async function handleTogglePin(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		const result = await toggleMessagePin(messageId);
		return { success: true, message: result.pinned ? 'Message pinned' : 'Message unpinned' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Pin action failed' };
	}
}

export async function handleToggleMute(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		const result = await toggleMessageMute(messageId);
		return { success: true, message: result.pinned ? 'Message muted' : 'Message unmuted' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Mute action failed' };
	}
}

export async function handleSnoozeMessage(messageId: string, hours = 24): Promise<{ success: boolean; message: string }> {
	const until = new Date(Date.now() + hours * 60 * 60 * 1000).toISOString();
	try {
		await snoozeMessage(messageId, until);
		return { success: true, message: 'Message snoozed' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Snooze action failed' };
	}
}

export async function handleAddMessageLabel(messageId: string, label: string): Promise<{ success: boolean; message: string }> {
	const trimmed = label.trim();
	if (!trimmed) return { success: false, message: 'Label is required' };
	try {
		await addMessageLabel(messageId, trimmed);
		return { success: true, message: 'Label added' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Label action failed' };
	}
}

export async function handleExportMessage(
	messageId: string,
	format: 'md' | 'eml' | 'json'
): Promise<{ success: boolean; error: string; result: MessageExportResponse | null }> {
	try {
		const result = await exportMessage(messageId, format);
		return { success: true, error: '', result };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Message export failed',
			result: null
		};
	}
}

export async function handleGenerateAiReply(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const aiReply = await generateAiReply(messageId, { tone: 'professional' });
		return { success: true, message: 'AI reply generated', insightPatch: { aiReply } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'AI reply generation failed',
			insightPatch: {}
		};
	}
}

export async function handleExtractTasks(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const response = await extractMessageTasks(messageId);
		return { success: true, message: 'Tasks extracted', insightPatch: { tasks: response.tasks } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Task extraction failed',
			insightPatch: {}
		};
	}
}

export async function handleExtractNotes(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const response = await extractMessageNotes(messageId);
		return { success: true, message: 'Notes extracted', insightPatch: { notes: response.notes } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Note extraction failed',
			insightPatch: {}
		};
	}
}

export async function handleTranslateMessage(messageId: string, targetLanguage = 'en'): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const translation = await translateMessage(messageId, targetLanguage);
		return { success: true, message: 'Translation requested', insightPatch: { translation } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Translation failed',
			insightPatch: {}
		};
	}
}

export async function handleWorkflowActionRequest(
	request: WorkflowActionRequest
): Promise<{ success: boolean; message: string; result: WorkflowActionResponse | null }> {
	try {
		const result = await runWorkflowAction(request);
		return { success: true, message: workflowActionStatusLabel(result), result };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Workflow action failed',
			result: null
		};
	}
}

export async function loadCommunicationDetail(
	messageId: string
): Promise<{ detail: CommunicationMessageDetail | null; error: string }> {
	try {
		const detail = await fetchCommunicationMessage(messageId);
		return { detail, error: '' };
	} catch (error) {
		return {
			detail: null,
			error: error instanceof Error ? error.message : 'Unknown communication detail error'
		};
	}
}

async function safe<T>(task: () => Promise<T>, fallback: T): Promise<T> {
	try {
		return await task();
	} catch {
		return fallback;
	}
}

export async function askAiAboutSelectedMessage(
	message: { message_id: string; subject: string } | null
): Promise<{ aiQuestion: string } | null> {
	if (!message) return null;
	return {
		aiQuestion: `Answer from local sources for message ${message.message_id}: ${message.subject}`
	};
}

export function senderLabel(sender: string) {
	const match = sender.match(/^"?([^"<]+)"?\s*</);
	return (match?.[1] ?? senderEmail(sender) ?? sender).trim();
}

export function senderEmail(sender: string) {
	const angleMatch = sender.match(/<([^>]+)>/);
	if (angleMatch?.[1]) {
		return angleMatch[1].trim();
	}
	const emailMatch = sender.match(/[^\s<>]+@[^\s<>]+/);
	return emailMatch?.[0]?.trim() ?? sender.trim();
}

export function sendCapabilityForAccount(account: ProviderAccount | null | undefined): SendCapability {
	if (!account) {
		return { canSend: false, transport: null, reason: 'Select a sending account' };
	}
	if (account.provider_kind === 'gmail') {
		return {
			canSend: false,
			transport: null,
			reason: 'Gmail send is unavailable until OAuth send scopes are configured'
		};
	}
	if (!['icloud', 'imap'].includes(account.provider_kind)) {
		return { canSend: false, transport: null, reason: 'This provider cannot send email' };
	}
	if (
		typeof account.config.smtp_host !== 'string' ||
		account.config.smtp_host.trim() === '' ||
		typeof account.config.smtp_port !== 'number'
	) {
		return {
			canSend: false,
			transport: null,
			reason: 'Reconnect this account to enable SMTP send'
		};
	}
	return { canSend: true, transport: 'smtp', reason: null };
}

export function buildMailAccountOptions(accounts: ProviderAccount[]): MailAccountOption[] {
	return accounts.map((account) => {
		const capability = sendCapabilityForAccount(account);
		return {
			accountId: account.account_id,
			providerKind: account.provider_kind,
			label: account.display_name || account.external_account_id || account.account_id,
			email: account.external_account_id,
			canSend: capability.canSend,
			transport: capability.transport,
			sendUnavailableReason: capability.reason
		};
	});
}

export function filterMessagesForWorkbench(
	messages: CommunicationMessageSummary[],
	accountId: string,
	query: string
): CommunicationMessageSummary[] {
	const trimmedAccountId = accountId.trim();
	const terms = query
		.toLowerCase()
		.trim()
		.split(/\s+/)
		.filter(Boolean);
	return messages.filter((message) => {
		if (trimmedAccountId && message.account_id !== trimmedAccountId) {
			return false;
		}
		if (terms.length === 0) {
			return true;
		}
		const haystack = [
			message.subject,
			message.sender,
			message.sender_display_name ?? '',
			message.body_text_preview,
			message.provider_record_id
		]
			.join(' ')
			.toLowerCase();
		return terms.every((term) => haystack.includes(term));
	});
}

type CommunicationListMessage = CommunicationMessageSummary | CommunicationMessageSummaryV2;

export type RelatedCommunicationMessage = CommunicationListMessage & {
	relation: 'same_contact' | 'same_conversation';
};

export function relatedMessagesForSelection(
	messages: CommunicationListMessage[],
	selected: CommunicationListMessage | null,
	limit = 8
): RelatedCommunicationMessage[] {
	if (!selected) return [];
	const selectedSender = senderEmail(selected.sender).toLowerCase();
	const selectedConversationId = selected.conversation_id?.trim() ?? '';
	return messages
		.filter((message) => message.message_id !== selected.message_id)
		.map((message) => {
			const sameConversation =
				selectedConversationId !== '' && message.conversation_id === selectedConversationId;
			const sameContact = senderEmail(message.sender).toLowerCase() === selectedSender;
			if (!sameConversation && !sameContact) return null;
			return {
				...message,
				relation: sameConversation ? 'same_conversation' : 'same_contact'
			} satisfies RelatedCommunicationMessage;
		})
		.filter((message): message is RelatedCommunicationMessage => message !== null)
		.sort((a, b) => messageSortTimestamp(b) - messageSortTimestamp(a))
		.slice(0, limit);
}

function messageSortTimestamp(message: CommunicationListMessage): number {
	const value = message.occurred_at ?? message.projected_at;
	const timestamp = Date.parse(value);
	return Number.isFinite(timestamp) ? timestamp : 0;
}

export function conversationPreview(
	message:
		| CommunicationMessageSummary
		| CommunicationMessageSummaryV2
		| (CommunicationMessageSummary & { ai_summary?: string | null })
): string {
	const aiSummary = 'ai_summary' in message ? cleanPreviewText(message.ai_summary ?? '') : '';
	if (aiSummary) return truncatePreview(aiSummary);

	const bodyPreview = cleanPreviewText(message.body_text_preview);
	if (bodyPreview) return truncatePreview(bodyPreview);

	return truncatePreview(cleanPreviewText(message.subject) || message.subject.trim());
}

export function messageContentText(value: string): string {
	return cleanPreviewText(value);
}

export type RenderedMessageContent = {
	html: string;
	mode: 'html' | 'text';
};

export type OriginalMailSrcdocOptions = {
	messageId?: string | null;
	apiBaseUrl?: string | null;
};

export function renderMessageContent(value: string): RenderedMessageContent {
	const withoutHeaders = stripLeadingMimeHeaderBlock(value);
	const decodedHtml = looksLikeEscapedHtml(withoutHeaders)
		? decodeBasicEntities(withoutHeaders)
		: withoutHeaders;

	if (looksLikeHtml(decodedHtml)) {
		return {
			html: sanitizeEmailHtml(decodedHtml),
			mode: 'html'
		};
	}

	return {
		html: renderPlainMessageText(withoutHeaders),
		mode: 'text'
	};
}

export function originalMailSrcdoc(
	value: string,
	options: OriginalMailSrcdocOptions = {}
): string {
	const html = rewriteRemoteMailImageSources(value.trim(), options);
	if (!html) return '';
	const base = '<base target="_blank">';
	if (/<html[\s>]/i.test(html)) {
		if (/<head[\s>]/i.test(html)) {
			return html.replace(/<head(\s[^>]*)?>/i, (match) => `${match}${base}`);
		}
		return html.replace(/<html(\s[^>]*)?>/i, (match) => `${match}<head>${base}</head>`);
	}
	return `<!doctype html><html><head>${base}</head><body>${html}</body></html>`;
}

export function remoteMailImageProxyUrl(
	messageId: string,
	imageUrl: string,
	apiBaseUrl: string
): string {
	const proxyUrl = new URL(
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}/remote-image`,
		apiBaseUrl.replace(/\/+$/, '') + '/'
	);
	proxyUrl.searchParams.set('url', decodeBasicEntities(imageUrl.trim()));
	return proxyUrl.toString();
}

export function buildWorkflowActionRequest(
	action: WorkflowActionKind,
	message: Pick<CommunicationMessageSummary, 'message_id' | 'subject'> | null,
	commandId = workflowCommandId(action)
): WorkflowActionRequest {
	const request: WorkflowActionRequest = {
		command_id: commandId,
		action
	};
	if (message) {
		request.source = { kind: 'communication_message', id: message.message_id };
	}
	if (['create_task', 'create_note', 'create_document', 'link_document', 'create_event'].includes(action)) {
		request.input = { title: message?.subject?.trim() || 'Untitled' };
	}
	return request;
}

export function newComposeForm(accountId: string): ComposeFormModel {
	return {
		draft_id: `draft-${Date.now()}`,
		account_id: accountId,
		to_text: '',
		cc_text: '',
		bcc_text: '',
		subject: '',
		body: '',
		mode: 'compose',
		in_reply_to: null,
		references: []
	};
}

export function buildReplyComposeForm(
	message: CommunicationMessageSummary | CommunicationMessageDetailItem,
	accountId: string
): ComposeFormModel {
	const providerRecordId = message.provider_record_id || message.message_id;
	return {
		...newComposeForm(accountId),
		to_text: senderEmail(message.sender),
		subject: subjectWithPrefix(message.subject, 'Re:'),
		mode: 'reply',
		in_reply_to: providerRecordId,
		references: [providerRecordId]
	};
}

export function buildForwardComposeForm(
	message: CommunicationMessageSummary | CommunicationMessageDetailItem,
	accountId: string
): ComposeFormModel {
	return {
		...newComposeForm(accountId),
		subject: subjectWithPrefix(message.subject, 'Fwd:'),
		body: `\n\nForwarded message:\nFrom: ${message.sender}\nSubject: ${message.subject}\n\n${
			'body_text' in message ? message.body_text : message.body_text_preview
		}`,
		mode: 'forward'
	};
}

function subjectWithPrefix(subject: string, prefix: 'Re:' | 'Fwd:'): string {
	return subject.toLowerCase().startsWith(prefix.toLowerCase()) ? subject : `${prefix} ${subject}`;
}

function workflowCommandId(action: WorkflowActionKind): string {
	const entropy =
		typeof crypto !== 'undefined' && 'randomUUID' in crypto
			? crypto.randomUUID()
			: `${Date.now()}-${Math.random().toString(16).slice(2)}`;
	return `mail-${action}-${entropy}`;
}

function workflowActionStatusLabel(result: WorkflowActionResponse): string {
	switch (result.action) {
		case 'create_task':
			return 'Task created';
		case 'create_note':
			return 'Note created';
		case 'create_document':
		case 'link_document':
			return 'Document linked';
		case 'create_event':
			return 'Event created';
		case 'create_contact':
			return 'Contact created';
		case 'archive':
			return 'Archived';
		case 'reply':
			return 'Reply opened';
		default:
			return 'Workflow action completed';
	}
}

function rewriteRemoteMailImageSources(
	html: string,
	options: OriginalMailSrcdocOptions
): string {
	const messageId = options.messageId?.trim();
	const apiBaseUrl = options.apiBaseUrl?.trim();
	if (!messageId || !apiBaseUrl) return html;

	return rewriteRemoteMailCssImageUrls(
		html.replace(/<img\b[^>]*>/gi, (tag) =>
			rewriteRemoteMailImageAttribute(tag, 'src', messageId, apiBaseUrl)
		),
		messageId,
		apiBaseUrl
	).replace(/<(?:table|td|th|div|body)\b[^>]*>/gi, (tag) =>
		rewriteRemoteMailImageAttribute(tag, 'background', messageId, apiBaseUrl)
	);
}

function rewriteRemoteMailImageAttribute(
	tag: string,
	attributeName: 'background' | 'src',
	messageId: string,
	apiBaseUrl: string
): string {
	const attributePattern = new RegExp(
		`(\\s+${attributeName}\\s*=\\s*)(?:"([^"]*)"|'([^']*)'|([^\\s"'=<>` + '`' + `]+))`,
		'i'
	);
	return tag.replace(
		attributePattern,
		(match, prefix: string, doubleQuoted?: string, singleQuoted?: string, unquoted?: string) => {
			const src = doubleQuoted ?? singleQuoted ?? unquoted ?? '';
			if (!isRemoteMailImageSource(src)) return match;
			const proxyUrl = remoteMailImageProxyUrl(messageId, src, apiBaseUrl);
			if (doubleQuoted !== undefined) return `${prefix}"${escapeHtml(proxyUrl)}"`;
			if (singleQuoted !== undefined) return `${prefix}'${escapeHtml(proxyUrl)}'`;
			return `${prefix}${escapeHtml(proxyUrl)}`;
		}
	);
}

function rewriteRemoteMailCssImageUrls(
	html: string,
	messageId: string,
	apiBaseUrl: string
): string {
	return html.replace(
		/url\(\s*(?:"([^"]*)"|'([^']*)'|([^'")\s]+))\s*\)/gi,
		(match, doubleQuoted?: string, singleQuoted?: string, unquoted?: string) => {
			const src = doubleQuoted ?? singleQuoted ?? unquoted ?? '';
			if (!isRemoteMailImageSource(src)) return match;
			const proxyUrl = remoteMailImageProxyUrl(messageId, src, apiBaseUrl);
			return `url('${escapeHtml(proxyUrl)}')`;
		}
	);
}

function isRemoteMailImageSource(value: string): boolean {
	const decoded = decodeBasicEntities(value).trim();
	return /^https?:\/\//i.test(decoded);
}

function cleanPreviewText(value: string): string {
	const withoutBlocks = value
		.replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ')
		.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, ' ')
		.replace(/<\/?(?:div|p|span|br|table|tbody|tr|td|th|html|body|head)[^>]*>/gi, ' ')
		.replace(/<[^>]+>/g, ' ');
	const cleanedLines = withoutBlocks
		.split(/\r?\n/)
		.map((line) => decodeBasicEntities(line).trim())
		.filter((line) => line && !looksLikeMimeHeader(line) && !looksLikeCssDeclaration(line) && !looksLikeCssRule(line));
	return cleanedLines.join(' ').replace(/\s+/g, ' ').trim();
}

function renderPlainMessageText(value: string): string {
	const cleaned = value
		.split(/\r?\n/)
		.filter((line) => !looksLikeMimeHeader(line) && !looksLikeCssDeclaration(line) && !looksLikeCssRule(line))
		.join('\n')
		.trim();
	if (!cleaned) return '';
	return cleaned
		.split(/\n{2,}/)
		.map((paragraph) => `<p>${escapeHtml(decodeBasicEntities(paragraph)).replace(/\n/g, '<br>')}</p>`)
		.join('');
}

function sanitizeEmailHtml(value: string): string {
	const withoutUnsafeBlocks = value
		.replace(/<!doctype[^>]*>/gi, ' ')
		.replace(/<!--[\s\S]*?-->/g, ' ')
		.replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, ' ')
		.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, ' ')
		.replace(/<head\b[^>]*>[\s\S]*?<\/head>/gi, ' ')
		.replace(/<title\b[^>]*>[\s\S]*?<\/title>/gi, ' ')
		.replace(/<meta\b[^>]*>/gi, ' ')
		.replace(/<link\b[^>]*>/gi, ' ')
		.replace(/<iframe\b[^>]*>[\s\S]*?<\/iframe>/gi, ' ')
		.replace(/<object\b[^>]*>[\s\S]*?<\/object>/gi, ' ')
		.replace(/<embed\b[^>]*>/gi, ' ')
		.replace(/<svg\b[^>]*>[\s\S]*?<\/svg>/gi, ' ')
		.replace(/<math\b[^>]*>[\s\S]*?<\/math>/gi, ' ')
		.replace(/<form\b[^>]*>[\s\S]*?<\/form>/gi, ' ');
	const output: string[] = [];
	let lastIndex = 0;
	const tagPattern = /<[^>]*>/g;
	let match: RegExpExecArray | null;

	while ((match = tagPattern.exec(withoutUnsafeBlocks)) !== null) {
		output.push(escapeHtml(decodeBasicEntities(withoutUnsafeBlocks.slice(lastIndex, match.index))));
		output.push(sanitizeEmailTag(match[0]));
		lastIndex = match.index + match[0].length;
	}
	output.push(escapeHtml(decodeBasicEntities(withoutUnsafeBlocks.slice(lastIndex))));

	const sanitized = cleanupSanitizedEmailHtml(output.join('').replace(/[ \t]{2,}/g, ' ').trim());
	return sanitized || '<p></p>';
}

const allowedEmailTags = new Set([
	'a',
	'blockquote',
	'br',
	'code',
	'div',
	'em',
	'h1',
	'h2',
	'h3',
	'h4',
	'h5',
	'h6',
	'hr',
	'li',
	'ol',
	'p',
	'pre',
	'small',
	'span',
	'strong',
	'table',
	'tbody',
	'td',
	'th',
	'thead',
	'tr',
	'u',
	'ul'
]);

const voidEmailTags = new Set(['br', 'hr']);

function sanitizeEmailTag(rawTag: string): string {
	const parsed = rawTag.match(/^<\s*(\/)?\s*([a-zA-Z][a-zA-Z0-9:-]*)([\s\S]*?)(\/?)\s*>$/);
	if (!parsed) return '';
	const isClosing = Boolean(parsed[1]);
	const originalTagName = parsed[2].toLowerCase();
	const tagName = normalizedEmailTagName(originalTagName);
	if (!allowedEmailTags.has(tagName)) {
		if (originalTagName === 'img') {
			const alt = readHtmlAttribute(rawTag, 'alt');
			return alt ? `<span class="mail-image-placeholder">${escapeHtml(decodeBasicEntities(alt))}</span>` : '';
		}
		return '';
	}
	if (voidEmailTags.has(tagName)) {
		return `<${tagName}>`;
	}
	if (isClosing) {
		return `</${tagName}>`;
	}
	return `<${tagName}${sanitizeEmailAttributes(tagName, rawTag)}>`;
}

function normalizedEmailTagName(tagName: string): string {
	if (tagName === 'b') return 'strong';
	if (tagName === 'i') return 'em';
	if (tagName === 'font') return 'span';
	return tagName;
}

function sanitizeEmailAttributes(tagName: string, rawTag: string): string {
	const attributes: string[] = [];
	if (tagName === 'a') {
		const href = readHtmlAttribute(rawTag, 'href');
		if (href && isSafeEmailHref(href)) {
			attributes.push(`href="${escapeHtml(decodeBasicEntities(href))}"`);
			attributes.push('target="_blank"');
			attributes.push('rel="noopener noreferrer"');
		}
	}
	if (tagName === 'td' || tagName === 'th') {
		for (const name of ['colspan', 'rowspan']) {
			const value = readHtmlAttribute(rawTag, name);
			if (value && /^\d{1,2}$/.test(value) && Number(value) > 0 && Number(value) <= 20) {
				attributes.push(`${name}="${value}"`);
			}
		}
	}
	return attributes.length ? ` ${attributes.join(' ')}` : '';
}

function readHtmlAttribute(rawTag: string, name: string): string | null {
	const pattern = new RegExp(`${name}\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s"'=<>` + '`' + `]+))`, 'i');
	const match = rawTag.match(pattern);
	return match?.[1] ?? match?.[2] ?? match?.[3] ?? null;
}

function isSafeEmailHref(value: string): boolean {
	const decoded = decodeBasicEntities(value).trim();
	return /^(https?:|mailto:|tel:)/i.test(decoded);
}

function cleanupSanitizedEmailHtml(value: string): string {
	let previous = '';
	let cleaned = value;
	while (cleaned !== previous) {
		previous = cleaned;
		cleaned = cleaned
			.replace(/<a\b[^>]*>\s*<\/a>/gi, '')
			.replace(/<(span|strong|em|small)\b[^>]*>\s*<\/\1>/gi, '');
	}
	return cleaned.trim();
}

function looksLikeHtml(value: string): boolean {
	return /<\/?(?:html|body|div|p|span|table|tr|td|th|br|strong|b|em|i|a|style|head)\b[\s\S]*?>/i.test(value);
}

function looksLikeEscapedHtml(value: string): boolean {
	return /&lt;\/?(?:html|body|div|p|span|table|tr|td|th|br|strong|b|em|i|a|style|head)\b[\s\S]*?&gt;/i.test(value);
}

function stripLeadingMimeHeaderBlock(value: string): string {
	const lines = value.split(/\r?\n/);
	let sawHeader = false;
	let index = 0;
	while (index < lines.length) {
		const line = lines[index].trim();
		if (looksLikeMimeHeader(line)) {
			sawHeader = true;
			index += 1;
			continue;
		}
		if (sawHeader && !line) {
			index += 1;
			break;
		}
		break;
	}
	return sawHeader ? lines.slice(index).join('\n') : value;
}

function escapeHtml(value: string): string {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;');
}

function looksLikeMimeHeader(line: string): boolean {
	return /^(content-type|mime-version|content-transfer-encoding|received|dkim-signature|message-id|from|to|subject|date)\s*:/i.test(line);
}

function looksLikeCssDeclaration(line: string): boolean {
	const declarations = line.split(';').map((part) => part.trim()).filter(Boolean);
	if (!declarations.length) return false;
	const cssLike = declarations.filter((part) =>
		/^(margin|padding|font|font-family|font-size|line-height|color|background|border|width|height|display|box-sizing|min-width|max-width|table-layout|border-collapse)\s*:/i.test(part)
	);
	return cssLike.length > 0 && cssLike.length === declarations.length;
}

function looksLikeCssRule(line: string): boolean {
	return /\{[^}]*\b(margin|padding|font|font-family|font-size|line-height|color|background|border|width|height|display|box-sizing|min-width|max-width|table-layout|border-collapse)\s*:/i.test(line);
}

function decodeBasicEntities(value: string): string {
	return value
		.replace(/&zwnj;|&zwj;/gi, ' ')
		.replace(/&shy;/gi, '')
		.replace(/&nbsp;/g, ' ')
		.replace(/&amp;/g, '&')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#39;/g, "'")
		.replace(/&shy;/gi, '')
		.replace(/&#x([0-9a-f]+);/gi, (_, code: string) => entityCodeToText(Number.parseInt(code, 16)))
		.replace(/&#(\d+);/g, (_, code: string) => entityCodeToText(Number.parseInt(code, 10)));
}

function entityCodeToText(code: number): string {
	if (!Number.isFinite(code)) return ' ';
	if (code === 8204 || code === 8205 || code === 65279) return ' ';
	if (code < 32 || code === 127) return ' ';
	try {
		const value = String.fromCodePoint(code);
		return value.trim() ? value : ' ';
	} catch {
		return ' ';
	}
}

function truncatePreview(value: string, limit = 140): string {
	if (value.length <= limit) return value;
	return `${value.slice(0, Math.max(0, limit - 3)).trimEnd()}...`;
}

export function messageTime(message: CommunicationMessageSummary | CommunicationMessageDetailItem) {
	return formatDateTime(message.occurred_at ?? message.projected_at);
}

export function communicationChannelIcon(channelKind: string) {
	if (channelKind === 'telegram_user' || channelKind === 'telegram_bot') {
		return 'tabler:brand-telegram';
	}
	if (channelKind === 'whatsapp_web') {
		return 'tabler:brand-whatsapp';
	}
	return 'tabler:mail';
}

export function communicationChannelLabel(channelKind: string) {
	if (channelKind === 'telegram_user') {
		return 'Telegram user';
	}
	if (channelKind === 'telegram_bot') {
		return 'Telegram bot';
	}
	if (channelKind === 'whatsapp_web') {
		return 'WhatsApp Web';
	}
	return 'Email';
}

export function attachmentIcon(contentType: string) {
	if (contentType.includes('pdf')) {
		return 'tabler:file-type-pdf';
	}
	if (contentType.startsWith('image/')) {
		return 'tabler:photo';
	}
	if (contentType.includes('spreadsheet') || contentType.includes('excel')) {
		return 'tabler:file-spreadsheet';
	}
	return 'tabler:file';
}



export function communicationSectionBadge(sectionId: CommunicationSectionId, mailboxHealth: MailboxHealth | null) {
	if (sectionId === 'inbox') {
		return mailboxHealth?.unread ? String(mailboxHealth.unread) : undefined;
	}
	if (sectionId === 'waiting') {
		return mailboxHealth?.waiting ? String(mailboxHealth.waiting) : undefined;
	}
	if (sectionId === 'needs_reply') {
		return mailboxHealth?.needs_action ? String(mailboxHealth.needs_action) : undefined;
	}
	return undefined;
}

export function communicationSectionWorkflowState(sectionId: CommunicationSectionId): WorkflowState | '' | null {
	switch (sectionId) {
		case 'inbox':
			return 'new';
		case 'waiting':
			return 'waiting';
		case 'needs_reply':
			return 'needs_action';
		case 'unified':
		case 'mail':
			return '';
		default:
			return null;
	}
}

export function selectCommunicationSection(
	sectionId: CommunicationSectionId
): { viewId: 'communications'; sectionId: CommunicationSectionId; workflowState: WorkflowState | '' } | null {
	const workflowState = communicationSectionWorkflowState(sectionId);
	if (workflowState !== null) {
		return { viewId: 'communications', sectionId, workflowState };
	}
	return null;
}
