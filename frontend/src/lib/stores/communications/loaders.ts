import { get } from 'svelte/store';
import type {
	CommunicationMessageDetail,
	CommunicationMessageSummary,
	MailSyncSettings,
	WorkflowState
} from '$lib/api';
import * as commsService from '$lib/services/communications';
import {
	communicationMessages,
	communicationsError,
	drafts,
	isCommunicationsLoading,
	isMailSyncBusy,
	lastMailSyncRuns,
	mailboxHealth,
	mailLocalStateFilter,
	mailMessageInsight,
	mailResourceSummary,
	mailResources,
	mailStateCounts,
	mailStateFilter,
	mailSyncError,
	mailSyncStatusMessage,
	mailSyncStatuses,
	messageSearchQuery,
	selectedCommunicationDetail,
	selectedCommunicationMessageId,
	selectedConversationIndex,
	selectedMailAccountId,
	selectedMailSyncSettings,
	threads,
	topSenders,
	mailAccountOptions
} from './state';

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
	await refreshMailWorkspaceData();
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
	await refreshMailWorkspaceData();
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

export async function refreshMailWorkspaceData(): Promise<void> {
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

async function loadInsightForDetail(detail: CommunicationMessageDetail | null): Promise<void> {
	const messageId = detail?.message.message_id ?? null;
	if (!messageId) {
		mailMessageInsight.set(null);
		return;
	}
	const insight = await commsService.loadMessageInsights(messageId);
	mailMessageInsight.set(insight);
}
