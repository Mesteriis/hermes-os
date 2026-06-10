import { derived, get, writable } from 'svelte/store';
import type {
	CommunicationMessageDetail,
	CommunicationMessageDetailItem,
	CommunicationMessageSummary,
	EmailDraft,
	EmailThread,
	MailMessageInsight,
	MailResourceSnapshot,
	MailResourceSummary,
	MailboxHealth,
	MessageAnalyzeResponse,
	MessageExportResponse,
	SendEmailResponse,
	SenderStats,
	WorkflowState,
	WorkflowStateCountItem
} from '$lib/api';
import type { CommunicationSectionId } from '$lib/layout';
import type { ComposeFormModel } from '$lib/services/communications';
import { navigateTo, navigateToCommunicationSection } from './navigation';
import type { NotificationItem } from './notifications';
import { emailProviderAccounts } from './settings';
import * as commsService from '$lib/services/communications';

export type ComposeForm = ComposeFormModel;

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
	bcc_text: '',
	subject: '',
	body: '',
	mode: 'compose',
	in_reply_to: null,
	references: []
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
export const mailResources = writable<MailResourceSnapshot>({ ...commsService.emptyMailResourceSnapshot });
export const mailResourceSummary = writable<MailResourceSummary>(
	commsService.summarizeMailResourceSnapshot(commsService.emptyMailResourceSnapshot)
);
export const mailMessageInsight = writable<MailMessageInsight | null>(null);
export const isMailActionRunning = writable(false);
export const mailActionStatus = writable('');
export const mailActionError = writable('');
export const lastMessageExport = writable<MessageExportResponse | null>(null);
export const isComposeOpen = writable(false);
export const composeForm = writable<ComposeForm>({ ...emptyComposeForm });
export const selectedMailAccountId = writable('');
export const messageSearchQuery = writable('');
export const isSendReviewOpen = writable(false);
export const isSendingMessage = writable(false);
export const composeSendError = writable('');
export const composeStatusMessage = writable('');
export const lastSendResponse = writable<SendEmailResponse | null>(null);
export const communicationProjects = writable<ProjectItem[]>([]);
export const communicationTasks = writable<TaskItem[]>([]);

export const mailAccountOptions = derived(emailProviderAccounts, ($emailProviderAccounts) =>
	commsService.buildMailAccountOptions($emailProviderAccounts)
);

export const selectedMailAccountOption = derived(
	[mailAccountOptions, selectedMailAccountId],
	([$mailAccountOptions, $selectedMailAccountId]) =>
		$mailAccountOptions.find((account) => account.accountId === $selectedMailAccountId) ??
		$mailAccountOptions[0] ??
		null
);

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
		loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined),
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadDrafts(),
		loadTopSenders(),
		loadThreads(),
		loadMailResources()
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
	await loadInsightForDetail(result.detail);
}

export async function loadCommunicationMessagesFiltered(filterState?: WorkflowState): Promise<void> {
	isCommunicationsLoading.set(true);
	const result = await commsService.loadCommunicationMessagesFiltered(
		filterState,
		get(selectedConversationIndex),
		get(selectedMailAccountId),
		get(messageSearchQuery)
	);
	communicationMessages.set(result.messages);
	selectedCommunicationDetail.set(result.detail);
	communicationsError.set(result.error);
	selectedConversationIndex.set(result.selectedIndex);
	isCommunicationsLoading.set(result.isLoading);
	await loadInsightForDetail(result.detail);
}

export async function loadMessageStateCounts(): Promise<void> {
	const result = await commsService.loadMessageStateCounts(get(selectedMailAccountId));
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
	await loadMailResources();
	isMailStateTransitioning.set(false);
}

export async function loadDrafts(): Promise<void> {
	const result = await commsService.loadDrafts(get(selectedMailAccountId));
	drafts.set(result.drafts);
}

export async function loadMailboxHealth(): Promise<void> {
	const result = await commsService.loadMailboxHealth(get(selectedMailAccountId));
	mailboxHealth.set(result.health);
}

export async function loadTopSenders(): Promise<void> {
	const result = await commsService.loadTopSenders(get(selectedMailAccountId));
	topSenders.set(result.senders);
}

export async function loadThreads(): Promise<void> {
	const result = await commsService.loadThreads(get(selectedMailAccountId));
	threads.set(result.threads);
}

export async function loadMailResources(): Promise<void> {
	const result = await commsService.loadMailResources(get(selectedMailAccountId));
	mailResources.set(result.resources);
	mailResourceSummary.set(result.summary);
}

export async function handleAnalyzeMessage(messageId: string): Promise<void> {
	const result = await commsService.handleAnalyzeMessage(messageId);
	aiAnalysisResult.set(result.result);
}

export async function handleSaveDraft(): Promise<void> {
	const result = await commsService.handleSaveDraft(get(composeForm));
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	composeForm.set({ ...emptyComposeForm });
	isComposeOpen.set(false);
	isSendReviewOpen.set(false);
	composeSendError.set('');
	composeStatusMessage.set('Draft saved');
	await loadDrafts();
}

export async function loadCommunicationDetail(messageId: string): Promise<void> {
	const result = await commsService.loadCommunicationDetail(messageId);
	selectedCommunicationDetail.set(result.detail);
	if (result.error) {
		communicationsError.set(result.error);
	}
	await loadInsightForDetail(result.detail);
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

export async function selectMailAccount(accountId: string): Promise<void> {
	selectedMailAccountId.set(accountId);
	selectedConversationIndex.set(0);
	await loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined);
	await Promise.all([
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadTopSenders(),
		loadDrafts(),
		loadThreads(),
		loadMailResources()
	]);
}

export async function updateMessageSearchQuery(query: string): Promise<void> {
	messageSearchQuery.set(query);
	selectedConversationIndex.set(0);
	await loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined);
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
		bcc_text: draft.bcc_recipients.join(', '),
		subject: draft.subject,
		body: draft.body_text,
		mode: 'compose',
		in_reply_to: draft.in_reply_to,
		references: draft.references
	});
	composeSendError.set('');
	composeStatusMessage.set('');
	isComposeOpen.set(true);
}

export function openNewMessage(accountId?: string): void {
	const selectedAccountId = accountId || defaultComposeAccountId();
	composeForm.set(commsService.newComposeForm(selectedAccountId));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openReplyToSelected(): void {
	const message = selectedMessageForCompose();
	if (!message) return;
	composeForm.set(commsService.buildReplyComposeForm(message, defaultComposeAccountId(message.account_id)));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openForwardSelected(): void {
	const message = selectedMessageForCompose();
	if (!message) return;
	composeForm.set(commsService.buildForwardComposeForm(message, defaultComposeAccountId(message.account_id)));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openSendReview(): void {
	composeSendError.set('');
	isSendReviewOpen.set(true);
}

export function closeSendReview(): void {
	isSendReviewOpen.set(false);
}

export async function confirmSendMessage(): Promise<void> {
	const form = get(composeForm);
	const account = get(emailProviderAccounts).find((candidate) => candidate.account_id === form.account_id);
	const capability = commsService.sendCapabilityForAccount(account);
	if (!capability.canSend) {
		composeSendError.set(capability.reason ?? 'Sending is unavailable for this account');
		isSendReviewOpen.set(false);
		return;
	}
	isSendingMessage.set(true);
	composeSendError.set('');
	const result = await commsService.handleSendMessage(form);
	isSendingMessage.set(false);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	lastSendResponse.set(result.result);
	composeStatusMessage.set(`Sent via ${result.result?.transport ?? 'provider'}`);
	isSendReviewOpen.set(false);
	isComposeOpen.set(false);
	composeForm.set({ ...emptyComposeForm });
	await Promise.all([loadDrafts(), loadMailboxHealth(), loadMailResources()]);
}

export async function togglePinSelectedMessage(): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleTogglePin(messageId), true);
}

export async function toggleMuteSelectedMessage(): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleToggleMute(messageId), true);
}

export async function snoozeSelectedMessage(): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleSnoozeMessage(messageId), true);
}

export async function addLabelToSelectedMessage(label: string): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleAddMessageLabel(messageId, label), true);
}

export async function exportSelectedMessage(format: 'md' | 'eml' | 'json'): Promise<void> {
	const messageId = selectedMessageId();
	if (!messageId) return;
	isMailActionRunning.set(true);
	mailActionError.set('');
	const result = await commsService.handleExportMessage(messageId, format);
	isMailActionRunning.set(false);
	if (!result.success) {
		mailActionError.set(result.error);
		return;
	}
	lastMessageExport.set(result.result);
	mailActionStatus.set('Export ready');
}

export async function generateReplyForSelectedMessage(): Promise<void> {
	await runSelectedInsightAction((messageId) => commsService.handleGenerateAiReply(messageId));
}

export async function extractTasksForSelectedMessage(): Promise<void> {
	await runSelectedInsightAction((messageId) => commsService.handleExtractTasks(messageId));
}

export async function extractNotesForSelectedMessage(): Promise<void> {
	await runSelectedInsightAction((messageId) => commsService.handleExtractNotes(messageId));
}

export async function translateSelectedMessage(targetLanguage = 'en'): Promise<void> {
	await runSelectedInsightAction((messageId) => commsService.handleTranslateMessage(messageId, targetLanguage));
}

function defaultComposeAccountId(fallbackAccountId = ''): string {
	const selected = get(selectedMailAccountId);
	if (selected) return selected;
	const options = get(mailAccountOptions);
	return (
		options.find((account) => account.canSend)?.accountId ??
		options[0]?.accountId ??
		fallbackAccountId
	);
}

function selectedMessageForCompose(): CommunicationMessageSummary | CommunicationMessageDetailItem | null {
	return get(selectedCommunicationDetail)?.message ?? get(selectedCommunication);
}

async function loadInsightForDetail(detail: CommunicationMessageDetail | null): Promise<void> {
	const messageId = detail?.message.message_id ?? null;
	if (!messageId) {
		mailMessageInsight.set(null);
		return;
	}
	const insight = await commsService.loadMessageInsights(messageId);
	mailMessageInsight.set(insight);
}

function selectedMessageId(): string | null {
	return get(selectedCommunicationDetail)?.message.message_id ?? get(selectedCommunication)?.message_id ?? null;
}

async function runSelectedMessageAction(
	action: (messageId: string) => Promise<{ success: boolean; message: string }>,
	reloadAfter: boolean
): Promise<void> {
	const messageId = selectedMessageId();
	if (!messageId) return;
	isMailActionRunning.set(true);
	mailActionError.set('');
	const result = await action(messageId);
	isMailActionRunning.set(false);
	if (!result.success) {
		mailActionError.set(result.message);
		return;
	}
	mailActionStatus.set(result.message);
	if (reloadAfter) {
		await loadCommunicationDetail(messageId);
		await loadMailResources();
	}
}

async function runSelectedInsightAction(
	action: (messageId: string) => Promise<{
		success: boolean;
		message: string;
		insightPatch: Partial<MailMessageInsight>;
	}>
): Promise<void> {
	const messageId = selectedMessageId();
	if (!messageId) return;
	isMailActionRunning.set(true);
	mailActionError.set('');
	const result = await action(messageId);
	isMailActionRunning.set(false);
	if (!result.success) {
		mailActionError.set(result.message);
		return;
	}
	mailActionStatus.set(result.message);
	mailMessageInsight.update((current) => {
		if (!current || current.messageId !== messageId) return current;
		return { ...current, ...result.insightPatch };
	});
}

export const communicationChannelIcon = commsService.communicationChannelIcon;
export const communicationChannelLabel = commsService.communicationChannelLabel;
export const senderLabel = commsService.senderLabel;
export const senderEmail = commsService.senderEmail;
export const messageTime = commsService.messageTime;
export const attachmentIcon = commsService.attachmentIcon;
