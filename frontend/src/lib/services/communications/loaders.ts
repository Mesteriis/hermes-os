import {
	analyzeMessage,
	fetchCommunicationMessage,
	fetchCommunicationMessages,
	fetchDrafts,
	fetchMailMessages,
	fetchMailboxHealth,
	fetchMailSyncSettings,
	fetchMailSyncStatus,
	fetchMessageStateCounts,
	fetchThreads,
	fetchTopSenders,
	restoreMessage,
	runMailFullResync,
	runMailSyncNow,
	transitionMessageWorkflowState,
	trashMessage,
	updateMailSyncSettings,
	type CommunicationMessageDetail,
	type CommunicationMessageSummary,
	type EmailDraft,
	type EmailThread,
	type LocalMessageState,
	type MailboxHealth,
	type MailSyncRunResponse,
	type MailSyncSettings,
	type MailSyncSettingsUpdate,
	type MailSyncStatus,
	type MessageAnalyzeResponse,
	type SenderStats,
	type WorkflowState,
	type WorkflowStateCountItem
} from '$lib/api';
import { COMMUNICATIONS_NAVIGATOR_LIMIT } from './constants';

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
				detail = await fetchCommunicationMessage(messages[nextIndex].message_id);
			} catch {
				/* detail optional */
			}
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
				detail = await fetchCommunicationMessage(messages[nextIndex].message_id);
			} catch {
				/* detail optional */
			}
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

export async function triggerMailFullResync(
	accountId: string
): Promise<{ run: MailSyncRunResponse | null; error: string }> {
	const target = accountId.trim();
	if (!target) {
		return { run: null, error: 'Select one account before full resync' };
	}
	try {
		return { run: await runMailFullResync(target), error: '' };
	} catch (error) {
		return {
			run: null,
			error: error instanceof Error ? error.message : 'Mail full resync request failed'
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
		// Side effects are handled by the caller reloading data.
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
