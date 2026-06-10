import type { CommunicationSectionId } from '$lib/layout';
import {
	fetchCommunicationMessages,
	fetchMailMessages,
	fetchMessageStateCounts,
	transitionMessageWorkflowState,
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
	type CommunicationMessageSummary,
	type CommunicationMessageDetail,
	type WorkflowState,
	type WorkflowStateCountItem,
	type EmailDraft,
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
	type MessageExportResponse
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
		const response = await fetchCommunicationMessages(50);
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
	query?: string
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
			50
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

export async function loadMessageStateCounts(accountId?: string): Promise<{ counts: WorkflowStateCountItem[]; error: string }> {
	try {
		const response = await fetchMessageStateCounts(accountId);
		return { counts: response.counts ?? [], error: '' };
	} catch {
		return { counts: [], error: '' };
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
