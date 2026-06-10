import { derived, get, writable } from 'svelte/store';
import type {
	CommunicationMessageDetail,
	CommunicationMessageSummary,
	EmailDraft,
	EmailThread,
	MailboxHealth,
	MessageAnalyzeResponse,
	SenderStats,
	WorkflowState,
	WorkflowStateCountItem
} from '$lib/api';
import type { CommunicationSectionId } from '$lib/layout';
import { navigateTo, navigateToCommunicationSection } from './navigation';
import type { NotificationItem } from './notifications';
import * as commsService from '$lib/services/communications';

export type ComposeForm = {
	draft_id: string;
	account_id: string;
	to_text: string;
	cc_text: string;
	subject: string;
	body: string;
};

export type ProjectItem = {
	name: string;
	kind: string;
	progress: number;
	icon: string;
	tone: string;
};

export type TaskItem = {
	title: string;
	due: string;
};

const emptyComposeForm: ComposeForm = {
	draft_id: '',
	account_id: '',
	to_text: '',
	cc_text: '',
	subject: '',
	body: ''
};

export const communicationMessages = writable<CommunicationMessageSummary[]>([]);
export const selectedCommunicationDetail = writable<CommunicationMessageDetail | null>(null);
export const communicationsError = writable('');
export const isCommunicationsLoading = writable(false);
export const selectedConversationIndex = writable(0);
export const mailStateFilter = writable<WorkflowState | ''>('');
export const mailStateCounts = writable<WorkflowStateCountItem[]>([]);
export const isMailStateTransitioning = writable(false);
export const isAiAnswerSubmitting = writable(false);
export const aiAnalysisResult = writable<MessageAnalyzeResponse | null>(null);
export const drafts = writable<EmailDraft[]>([]);
export const mailboxHealth = writable<MailboxHealth | null>(null);
export const topSenders = writable<SenderStats[]>([]);
export const threads = writable<EmailThread[]>([]);
export const isComposeOpen = writable(false);
export const composeForm = writable<ComposeForm>({ ...emptyComposeForm });
export const communicationProjects = writable<ProjectItem[]>([]);
export const communicationTasks = writable<TaskItem[]>([]);

export const selectedCommunication = derived(
	[communicationMessages, selectedConversationIndex],
	([$communicationMessages, $selectedConversationIndex]) =>
		$communicationMessages[$selectedConversationIndex] ?? null
);

export const conversations = derived(communicationMessages, ($communicationMessages) => {
	const channelLabels: Record<string, string> = {
		email: 'Email',
		gmail: 'Gmail',
		icloud: 'iCloud',
		imap: 'IMAP',
		telegram_user: 'Telegram',
		telegram_bot: 'Telegram',
		whatsapp_web: 'WhatsApp'
	};
	return $communicationMessages.map((msg) => ({
		name: msg.sender_display_name || msg.sender || 'Unknown',
		role: msg.sender || '',
		project: msg.subject || msg.body_text_preview,
		channel: channelLabels[msg.channel_kind] || msg.channel_kind,
		time: msg.occurred_at || msg.projected_at,
		preview: msg.body_text_preview
	}));
});

export async function loadCommunicationsWorkspace(): Promise<void> {
	await Promise.all([
		loadCommunications(),
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadDrafts(),
		loadTopSenders(),
		loadThreads()
	]);
}

export async function loadCommunications(): Promise<void> {
	isCommunicationsLoading.set(true);
	const result = await commsService.loadCommunications(get(selectedConversationIndex));
	communicationMessages.set(result.messages);
	selectedCommunicationDetail.set(result.detail);
	communicationsError.set(result.error);
	selectedConversationIndex.set(result.selectedIndex);
	isCommunicationsLoading.set(result.isLoading);
}

export async function loadCommunicationMessagesFiltered(filterState?: WorkflowState): Promise<void> {
	isCommunicationsLoading.set(true);
	const result = await commsService.loadCommunicationMessagesFiltered(filterState, get(selectedConversationIndex));
	communicationMessages.set(result.messages);
	selectedCommunicationDetail.set(result.detail);
	communicationsError.set(result.error);
	selectedConversationIndex.set(result.selectedIndex);
	isCommunicationsLoading.set(result.isLoading);
}

export async function loadMessageStateCounts(): Promise<void> {
	const result = await commsService.loadMessageStateCounts();
	mailStateCounts.set(result.counts);
	if (result.error) {
		communicationsError.set(result.error);
	}
}

export async function handleWorkflowStateTransition(messageId: string, newState: WorkflowState): Promise<void> {
	isMailStateTransitioning.set(true);
	const currentFilter = get(mailStateFilter);
	const result = await commsService.handleWorkflowStateTransition(messageId, newState, currentFilter);
	if (result.error) {
		communicationsError.set(result.error);
	}
	await loadCommunicationMessagesFiltered(currentFilter || undefined);
	await loadMessageStateCounts();
	await loadMailboxHealth();
	await loadTopSenders();
	await loadDrafts();
	await loadThreads();
	isMailStateTransitioning.set(false);
}

export async function loadDrafts(): Promise<void> {
	const result = await commsService.loadDrafts();
	drafts.set(result.drafts);
}

export async function loadMailboxHealth(): Promise<void> {
	const result = await commsService.loadMailboxHealth();
	mailboxHealth.set(result.health);
}

export async function loadTopSenders(): Promise<void> {
	const result = await commsService.loadTopSenders();
	topSenders.set(result.senders);
}

export async function loadThreads(): Promise<void> {
	const result = await commsService.loadThreads();
	threads.set(result.threads);
}

export async function handleAnalyzeMessage(messageId: string): Promise<void> {
	const result = await commsService.handleAnalyzeMessage(messageId);
	aiAnalysisResult.set(result.result);
}

export async function handleSaveDraft(): Promise<void> {
	const result = await commsService.handleSaveDraft(get(composeForm));
	if (!result.success) {
		return;
	}
	composeForm.set({ ...emptyComposeForm });
	isComposeOpen.set(false);
	await loadDrafts();
}

export async function loadCommunicationDetail(messageId: string): Promise<void> {
	const result = await commsService.loadCommunicationDetail(messageId);
	selectedCommunicationDetail.set(result.detail);
	if (result.error) {
		communicationsError.set(result.error);
	}
}

export function selectCommunication(index: number): void {
	const messages = get(communicationMessages);
	if (index < 0 || index >= messages.length) return;
	selectedConversationIndex.set(index);
	void loadCommunicationDetail(messages[index].message_id);
}

export async function askAiAboutSelectedMessage(): Promise<void> {
	const result = await commsService.askAiAboutSelectedMessage(get(selectedCommunication));
	if (result) {
		navigateTo('agents');
	}
}

export async function selectCommunicationSection(sectionId: CommunicationSectionId): Promise<void> {
	navigateToCommunicationSection(sectionId);
	const workflowState = commsService.communicationSectionWorkflowState(sectionId);
	if (workflowState !== null) {
		mailStateFilter.set(workflowState);
		await loadCommunicationMessagesFiltered(workflowState || undefined);
	}
}

export async function openCommunicationNotificationTarget(notification: NotificationItem): Promise<void> {
	const targetSection = notification.targetSection ?? (notification.source === 'telegram' ? 'telegram' : 'unified');
	await selectCommunicationSection(targetSection);
	if (!notification.messageId) {
		return;
	}

	const messages = get(communicationMessages);
	const messageIndex = messages.findIndex((message) => message.message_id === notification.messageId);
	if (messageIndex >= 0) {
		selectCommunication(messageIndex);
	} else {
		await loadCommunicationDetail(notification.messageId);
	}
}

export function openComposeForDraft(draft: EmailDraft): void {
	composeForm.set({
		draft_id: draft.draft_id,
		account_id: draft.account_id,
		to_text: draft.to_recipients.join(', '),
		cc_text: draft.cc_recipients.join(', '),
		subject: draft.subject,
		body: draft.body_text
	});
	isComposeOpen.set(true);
}

export const communicationChannelIcon = commsService.communicationChannelIcon;
export const communicationChannelLabel = commsService.communicationChannelLabel;
export const senderLabel = commsService.senderLabel;
export const senderEmail = commsService.senderEmail;
export const messageTime = commsService.messageTime;
export const attachmentIcon = commsService.attachmentIcon;
