import { get } from 'svelte/store';
import type {
	LocalMessageState,
	MailMessageInsight,
	WorkflowActionKind,
	WorkflowActionResponse,
	WorkflowState
} from '$lib/api';
import type { CommunicationSectionId } from '$lib/layout';
import type { NotificationItem } from '../notifications';
import { navigateTo, navigateToCommunicationSection } from '../navigation';
import * as commsService from '$lib/services/communications';
import { openReplyToSelected } from './compose';
import {
	aiAnalysisResult,
	communicationMessages,
	communicationsError,
	isAiAnswerSubmitting,
	isMailActionRunning,
	isMailStateTransitioning,
	lastMessageExport,
	mailActionError,
	mailActionStatus,
	mailLocalStateFilter,
	mailMessageInsight,
	mailResources,
	mailStateFilter,
	messageSearchQuery,
	selectedCommunication,
	selectedCommunicationMessageId,
	selectedConversationIndex,
	selectedMailAccountId
} from './state';
import {
	loadCommunicationDetail,
	loadCommunicationMessagesFiltered,
	loadDrafts,
	loadMailboxHealth,
	loadMailResources,
	loadMessageStateCounts,
	loadSelectedMailSyncSettings,
	loadThreads,
	loadTopSenders
} from './loaders';
import {
	selectedMessageForCompose,
	selectedMessageId
} from './selectors';

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

export async function handleAnalyzeMessage(messageId: string): Promise<void> {
	const result = await commsService.handleAnalyzeMessage(messageId);
	aiAnalysisResult.set(result.result);
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
