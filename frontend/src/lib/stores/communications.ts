import { derived, get, writable } from 'svelte/store';
import type {
	CommunicationMessageDetail,
	CommunicationMessageDetailItem,
	CommunicationMessageSummary,
	EmailDraft,
	EmailThread,
	LocalMessageState,
	MailMessageInsight,
	MailResourceSnapshot,
	MailResourceSummary,
	MailSyncRunResponse,
	MailSyncSettings,
	MailSyncStatus,
	MailboxHealth,
	MessageAnalyzeResponse,
	MessageExportResponse,
	SendEmailResponse,
	WorkflowActionKind,
	WorkflowActionResponse,
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

export type CommunicationsNavigatorMode = 'threads' | 'contacts';
export type CommunicationsInspectorMode = 'context' | 'contact' | 'organization' | null;
export type MessageContextTab = 'message' | 'attachments' | 'headers' | 'related' | 'timeline';

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
export const selectedCommunicationMessageId = writable<string | null>(null);
export const mailStateFilter = writable<WorkflowState | ''>('');
export const mailLocalStateFilter = writable<LocalMessageState>('active');
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
export const mailSyncStatuses = writable<MailSyncStatus[]>([]);
export const selectedMailSyncSettings = writable<MailSyncSettings | null>(null);
export const lastMailSyncRuns = writable<MailSyncRunResponse[]>([]);
export const isMailSyncBusy = writable(false);
export const mailSyncStatusMessage = writable('');
export const mailSyncError = writable('');
export const isComposeOpen = writable(false);
export const composeForm = writable<ComposeForm>({ ...emptyComposeForm });
export const selectedMailAccountId = writable('');
export const messageSearchQuery = writable('');
export const communicationsNavigatorMode = writable<CommunicationsNavigatorMode>('threads');
export const expandedCommunicationContactKey = writable<string | null>(null);
export const communicationsInspectorMode = writable<CommunicationsInspectorMode>(null);
export const activeMessageContextTab = writable<MessageContextTab>('message');
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
	[communicationMessages, selectedCommunicationMessageId, selectedConversationIndex],
	([$communicationMessages, $selectedCommunicationMessageId, $selectedConversationIndex]) =>
		($selectedCommunicationMessageId
			? $communicationMessages.find((message) => message.message_id === $selectedCommunicationMessageId)
			: null) ??
		$communicationMessages[$selectedConversationIndex] ??
		null
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
		loadMailResources(),
		loadMailSyncStatus(),
		loadSelectedMailSyncSettings()
	]);
}

export async function loadCommunications(): Promise<void> {
	isCommunicationsLoading.set(true);
	const result = await commsService.loadCommunications(get(selectedConversationIndex));
	communicationsError.set(result.error);
	await applyLoadedCommunicationResult(result.messages, result.detail, result.selectedIndex);
	isCommunicationsLoading.set(result.isLoading);
}

export async function loadCommunicationMessagesFiltered(filterState?: WorkflowState): Promise<void> {
	isCommunicationsLoading.set(true);
	const result = await commsService.loadCommunicationMessagesFiltered(
		filterState,
		get(selectedConversationIndex),
		get(selectedMailAccountId),
		get(messageSearchQuery),
		get(mailLocalStateFilter)
	);
	communicationsError.set(result.error);
	await applyLoadedCommunicationResult(result.messages, result.detail, result.selectedIndex);
	isCommunicationsLoading.set(result.isLoading);
}

export async function loadMessageStateCounts(): Promise<void> {
	const result = await commsService.loadMessageStateCounts(get(selectedMailAccountId), get(mailLocalStateFilter));
	mailStateCounts.set(result.counts);
	if (result.error) {
		communicationsError.set(result.error);
	}
}

export async function loadMailSyncStatus(): Promise<void> {
	const result = await commsService.loadMailSyncStatuses();
	mailSyncStatuses.set(result.statuses);
	if (result.error) {
		mailSyncError.set(result.error);
	}
}

export async function loadSelectedMailSyncSettings(): Promise<void> {
	const result = await commsService.loadMailSyncSettings(get(selectedMailAccountId));
	selectedMailSyncSettings.set(result.settings);
	if (result.error) {
		mailSyncError.set(result.error);
	}
}

export async function updateSelectedMailSyncSettings(
	patch: Partial<Pick<MailSyncSettings, 'sync_enabled' | 'batch_size' | 'poll_interval_seconds'>>
): Promise<void> {
	const accountId = get(selectedMailAccountId);
	const current = get(selectedMailSyncSettings);
	if (!accountId || !current) return;
	isMailSyncBusy.set(true);
	mailSyncError.set('');
	const result = await commsService.saveMailSyncSettings(accountId, {
		sync_enabled: patch.sync_enabled ?? current.sync_enabled,
		batch_size: patch.batch_size ?? current.batch_size,
		poll_interval_seconds: patch.poll_interval_seconds ?? current.poll_interval_seconds
	});
	isMailSyncBusy.set(false);
	if (result.error) {
		mailSyncError.set(result.error);
		return;
	}
	selectedMailSyncSettings.set(result.settings);
	mailSyncStatusMessage.set('Sync settings saved');
	await loadMailSyncStatus();
}

export async function runMailSyncNow(accountId?: string): Promise<void> {
	const explicitAccountId = accountId?.trim();
	const selectedAccountId = get(selectedMailAccountId).trim();
	const targets = explicitAccountId
		? [explicitAccountId]
		: selectedAccountId
			? [selectedAccountId]
			: get(mailAccountOptions).map((account) => account.accountId);
	isMailSyncBusy.set(true);
	mailSyncError.set('');
	mailSyncStatusMessage.set('Checking mail now');
	const result = await commsService.triggerMailSyncNow(targets);
	isMailSyncBusy.set(false);
	lastMailSyncRuns.set(result.runs);
	if (result.error) {
		mailSyncError.set(result.error);
	} else {
		mailSyncStatusMessage.set('Mail check finished');
	}
	await Promise.all([
		loadMailSyncStatus(),
		loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined),
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadTopSenders(),
		loadDrafts(),
		loadThreads(),
		loadMailResources()
	]);
}

export async function runMailFullResync(accountId?: string): Promise<void> {
	const target = accountId?.trim() || get(selectedMailAccountId).trim();
	isMailSyncBusy.set(true);
	mailSyncError.set('');
	mailSyncStatusMessage.set('Full resync started');
	const result = await commsService.triggerMailFullResync(target);
	isMailSyncBusy.set(false);
	if (result.run) {
		lastMailSyncRuns.set([result.run]);
	}
	if (result.error) {
		mailSyncError.set(result.error);
	} else {
		mailSyncStatusMessage.set('Full resync finished');
	}
	await Promise.all([
		loadMailSyncStatus(),
		loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined),
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadTopSenders(),
		loadDrafts(),
		loadThreads(),
		loadMailResources()
	]);
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

export async function toggleSelectedReadState(): Promise<void> {
	const message = selectedMessageForCompose();
	if (!message) return;
	const currentState =
		'workflow_state' in message ? (message.workflow_state as WorkflowState | null) : null;
	const nextState = commsService.nextReadWorkflowState(currentState);
	await handleWorkflowStateTransition(message.message_id, nextState);
	mailActionStatus.set(nextState === 'new' ? 'Marked unread' : 'Marked read');
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

export async function handleDeleteDraft(draftId: string): Promise<void> {
	const normalizedDraftId = draftId.trim();
	if (!normalizedDraftId) return;
	composeSendError.set('');
	composeStatusMessage.set('');
	const result = await commsService.handleDeleteDraft(normalizedDraftId);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	drafts.update((items) => items.filter((draft) => draft.draft_id !== normalizedDraftId));
	if (get(composeForm).draft_id === normalizedDraftId) {
		composeForm.set({ ...emptyComposeForm });
		isComposeOpen.set(false);
		isSendReviewOpen.set(false);
	}
	composeStatusMessage.set(result.deleted ? 'Draft deleted' : 'Draft was already deleted');
	await loadDrafts();
}

export async function autoSaveOpenComposeDraft(): Promise<void> {
	if (!get(isComposeOpen)) return;
	const form = get(composeForm);
	if (!form.account_id || !form.draft_id || !composeHasDraftContent(form)) return;
	const result = await commsService.handleSaveDraft(form);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	composeSendError.set('');
}

export async function restoreComposeDraftById(draftId: string | null | undefined): Promise<boolean> {
	if (!draftId?.trim()) {
		isComposeOpen.set(false);
		return false;
	}
	if (!get(drafts).some((draft) => draft.draft_id === draftId)) {
		await loadDrafts();
	}
	const draft = get(drafts).find((candidate) => candidate.draft_id === draftId);
	if (!draft) {
		isComposeOpen.set(false);
		return false;
	}
	openComposeForDraft(draft);
	return true;
}

export async function loadCommunicationDetail(messageId: string): Promise<void> {
	const result = await commsService.loadCommunicationDetail(messageId);
	if (result.detail?.message.message_id) {
		selectedCommunicationMessageId.set(result.detail.message.message_id);
	}
	selectedCommunicationDetail.set(result.detail);
	if (result.error) {
		communicationsError.set(result.error);
	}
	await loadInsightForDetail(result.detail);
}

async function applyLoadedCommunicationResult(
	messages: CommunicationMessageSummary[],
	detail: CommunicationMessageDetail | null,
	fallbackIndex: number
): Promise<void> {
	communicationMessages.set(messages);
	if (!messages.length) {
		selectedConversationIndex.set(0);
		selectedCommunicationMessageId.set(null);
		selectedCommunicationDetail.set(null);
		await loadInsightForDetail(null);
		return;
	}

	const desiredMessageId = get(selectedCommunicationMessageId);
	const restoredIndex = desiredMessageId
		? messages.findIndex((message) => message.message_id === desiredMessageId)
		: -1;
	const nextIndex =
		restoredIndex >= 0
			? restoredIndex
			: Math.min(Math.max(fallbackIndex, 0), messages.length - 1);
	const selectedMessage = messages[nextIndex];

	selectedConversationIndex.set(nextIndex);
	selectedCommunicationMessageId.set(selectedMessage.message_id);
	if (detail?.message.message_id === selectedMessage.message_id) {
		selectedCommunicationDetail.set(detail);
		await loadInsightForDetail(detail);
		return;
	}
	await loadCommunicationDetail(selectedMessage.message_id);
}

export function selectCommunication(index: number): void {
	const messages = get(communicationMessages);
	if (index < 0 || index >= messages.length) return;
	selectedConversationIndex.set(index);
	selectedCommunicationMessageId.set(messages[index].message_id);
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
		mailLocalStateFilter.set('active');
		mailStateFilter.set(workflowState);
		selectedCommunicationMessageId.set(null);
		await loadCommunicationMessagesFiltered(workflowState || undefined);
		await loadMessageStateCounts();
	}
}

export async function selectMailLocalState(localState: LocalMessageState): Promise<void> {
	mailLocalStateFilter.set(localState);
	selectedConversationIndex.set(0);
	selectedCommunicationMessageId.set(null);
	if (localState === 'trash') {
		mailStateFilter.set('');
	}
	await loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined);
	await loadMessageStateCounts();
}

export async function selectMailAccount(accountId: string): Promise<void> {
	selectedMailAccountId.set(accountId);
	selectedConversationIndex.set(0);
	selectedCommunicationMessageId.set(null);
	await loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined);
	await Promise.all([
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadTopSenders(),
		loadDrafts(),
		loadThreads(),
		loadMailResources(),
		loadSelectedMailSyncSettings()
	]);
}

export async function updateMessageSearchQuery(query: string): Promise<void> {
	messageSearchQuery.set(query);
	selectedConversationIndex.set(0);
	selectedCommunicationMessageId.set(null);
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

export async function toggleImportantSelectedMessage(): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleToggleImportant(messageId), true);
}

export async function toggleMuteSelectedMessage(): Promise<void> {
	await runSelectedMessageAction((messageId) => commsService.handleToggleMute(messageId), true);
}

export async function trashSelectedMessage(): Promise<void> {
	await runSelectedLocalStateAction((messageId) => commsService.handleTrashMessage(messageId));
}

export async function restoreSelectedMessage(): Promise<void> {
	await runSelectedLocalStateAction((messageId) => commsService.handleRestoreMessage(messageId));
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
	const downloaded = result.result ? commsService.downloadMessageExport(result.result) : false;
	mailActionStatus.set(downloaded ? 'Export downloaded' : 'Export ready');
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

export async function runSelectedWorkflowAction(action: WorkflowActionKind): Promise<void> {
	const message = selectedMessageForCompose();
	if (!message) return;
	if (action === 'reply') {
		await runWorkflowAction(commsService.buildWorkflowActionRequest(action, message));
		openReplyToSelected();
		return;
	}
	const request = commsService.buildWorkflowActionRequest(action, message);
	if (action === 'create_event') {
		const start = new Date(Date.now() + 60 * 60 * 1000);
		const end = new Date(start.getTime() + 60 * 60 * 1000);
		request.input = {
			...request.input,
			title: `Review: ${message.subject}`,
			starts_at: start.toISOString(),
			ends_at: end.toISOString()
		};
	}
	await runWorkflowAction(request);
	if (action === 'archive') {
		await Promise.all([
			loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined),
			loadMessageStateCounts()
		]);
		return;
	}
	await Promise.all([loadCommunicationDetail(message.message_id), loadMailResources()]);
}

export async function runNewWorkflowAction(action: Exclude<WorkflowActionKind, 'reply' | 'archive' | 'link_document'>): Promise<void> {
	const title = newWorkflowTitle(action);
	if (action === 'create_contact') {
		const email = promptValue('Email', '');
		if (!email) return;
		await runWorkflowAction({
			...commsService.buildWorkflowActionRequest(action, null),
			input: { title, email }
		});
		return;
	}
	if (action === 'create_event') {
		const start = new Date(Date.now() + 60 * 60 * 1000);
		const end = new Date(start.getTime() + 60 * 60 * 1000);
		await runWorkflowAction({
			...commsService.buildWorkflowActionRequest(action, null),
			input: { title, starts_at: start.toISOString(), ends_at: end.toISOString() }
		});
		return;
	}
	await runWorkflowAction({
		...commsService.buildWorkflowActionRequest(action, null),
		input: { title, body: title }
	});
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

function composeHasDraftContent(form: ComposeForm): boolean {
	return Boolean(
		form.to_text.trim() ||
		form.cc_text.trim() ||
		form.bcc_text.trim() ||
		form.subject.trim() ||
		form.body.trim()
	);
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

async function runSelectedLocalStateAction(
	action: (messageId: string) => Promise<{ success: boolean; message: string }>
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
	await Promise.all([
		loadCommunicationMessagesFiltered(get(mailStateFilter) || undefined),
		loadMessageStateCounts(),
		loadMailboxHealth(),
		loadTopSenders(),
		loadThreads(),
		loadMailResources()
	]);
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

async function runWorkflowAction(request: ReturnType<typeof commsService.buildWorkflowActionRequest>): Promise<void> {
	isMailActionRunning.set(true);
	mailActionError.set('');
	const result = await commsService.handleWorkflowActionRequest(request);
	isMailActionRunning.set(false);
	if (!result.success) {
		mailActionError.set(result.message);
		return;
	}
	mailActionStatus.set(result.message);
	applyWorkflowActionResult(result.result);
}

function applyWorkflowActionResult(result: WorkflowActionResponse | null): void {
	if (!result) return;
	if (result.action === 'archive') {
		const targetId = result.target.id;
		if (!targetId) return;
		communicationMessages.update((messages) => messages.filter((message) => message.message_id !== targetId));
	}
}

function newWorkflowTitle(action: WorkflowActionKind): string {
	switch (action) {
		case 'create_note':
			return 'New Note';
		case 'create_task':
			return 'New Task';
		case 'create_document':
			return 'New Document';
		case 'create_contact':
			return 'New Contact';
		case 'create_event':
			return 'New Event';
		default:
			return 'New';
	}
}

function promptValue(label: string, fallback: string): string {
	if (typeof window === 'undefined') return fallback;
	try {
		return window.prompt(label, fallback)?.trim() ?? '';
	} catch {
		return fallback.trim();
	}
}

export const communicationChannelIcon = commsService.communicationChannelIcon;
export const communicationChannelLabel = commsService.communicationChannelLabel;
export const conversationPreview = commsService.conversationPreview;
export const messageContentHtml = commsService.renderMessageContent;
export const messageContentText = commsService.messageContentText;
export const senderLabel = commsService.senderLabel;
export const senderEmail = commsService.senderEmail;
export const messageTime = commsService.messageTime;
export const attachmentIcon = commsService.attachmentIcon;
