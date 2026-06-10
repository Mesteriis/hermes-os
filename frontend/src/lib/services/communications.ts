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
	fetchCommunicationMessage,
	fetchMailMessage,
	type CommunicationMessageSummary,
	type CommunicationMessageDetail,
	type WorkflowState,
	type WorkflowStateCountItem,
	type EmailDraft,
	type MailboxHealth,
	type SenderStats,
	type EmailThread,
	type MessageAnalyzeResponse,
	type CommunicationMessageDetailItem
} from '$lib/api';
import { formatDateTime } from './formatting';

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
	selectedConversationIndex?: number
): Promise<{
	messages: CommunicationMessageSummary[];
	detail: CommunicationMessageDetail | null;
	error: string;
	isLoading: boolean;
	selectedIndex: number;
}> {
	try {
		const response = await fetchMailMessages(
			undefined, filterState || undefined, undefined, 50
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

export async function loadMessageStateCounts(): Promise<{ counts: WorkflowStateCountItem[]; error: string }> {
	try {
		const response = await fetchMessageStateCounts();
		return { counts: response.counts, error: '' };
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

export async function loadDrafts(): Promise<{ drafts: EmailDraft[] }> {
	try {
		const r = await fetchDrafts();
		return { drafts: r.items };
	} catch {
		return { drafts: [] };
	}
}

export async function loadMailboxHealth(): Promise<{ health: MailboxHealth | null }> {
	try {
		return { health: await fetchMailboxHealth() };
	} catch {
		return { health: null };
	}
}

export async function loadTopSenders(): Promise<{ senders: SenderStats[] }> {
	try {
		return { senders: await fetchTopSenders() };
	} catch {
		return { senders: [] };
	}
}

export async function loadThreads(): Promise<{ threads: EmailThread[] }> {
	try {
		const r = await fetchThreads();
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
	draft: {
		draft_id: string;
		account_id: string;
		to_text: string;
		cc_text: string;
		subject: string;
		body: string;
	}
): Promise<{ success: boolean }> {
	if (!draft.draft_id || !draft.subject) return { success: false };
	try {
		await createDraft({
			draft_id: draft.draft_id,
			account_id: draft.account_id || 'gmail-primary',
			to_recipients: draft.to_text.split(',').map((s) => s.trim()).filter(Boolean),
			cc_recipients: draft.cc_text.split(',').map((s) => s.trim()).filter(Boolean),
			subject: draft.subject,
			body_text: draft.body,
			status: 'draft'
		});
		return { success: true };
	} catch {
		return { success: false };
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
