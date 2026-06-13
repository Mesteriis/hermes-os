import {
	fetchAutomationPolicies,
	fetchAutomationTemplates,
	fetchCallTranscript,
	fetchTelegramAccounts,
	fetchTelegramCalls,
	fetchTelegramCapabilities,
	fetchTelegramChats,
	fetchTelegramMessages,
	fetchTelegramRuntimeStatus,
	type AutomationPolicy,
	type AutomationTemplate,
	type CallTranscript,
	type TelegramAccount,
	type TelegramCall,
	type TelegramCapabilitiesResponse,
	type TelegramChat,
	type TelegramMessage,
	type TelegramRuntimeStatus
} from '$lib/api';
import {
	TELEGRAM_CHAT_METADATA_LIMIT,
	TELEGRAM_SELECTED_CHAT_MESSAGE_LIMIT
} from './constants';

export async function loadTelegramWorkspace(
	selectedTelegramChatId: string,
	selectedTelegramCallId: string
): Promise<{
	capabilities: TelegramCapabilitiesResponse | null;
	accounts: TelegramAccount[];
	chats: TelegramChat[];
	messages: TelegramMessage[];
	templates: AutomationTemplate[];
	policies: AutomationPolicy[];
	calls: TelegramCall[];
	transcript: CallTranscript | null;
	runtimeStatuses: Record<string, TelegramRuntimeStatus>;
	selectedChatId: string;
	selectedCallId: string;
	error: string;
}> {
	try {
		const [
			capabilityResponse,
			accountResponse,
			chatResponse,
			messageResponse,
			templateResponse,
			policyResponse,
			callResponse
		] = await Promise.all([
			fetchTelegramCapabilities(),
			fetchTelegramAccounts(),
			fetchTelegramChats(undefined, TELEGRAM_CHAT_METADATA_LIMIT),
			fetchTelegramMessages(),
			fetchAutomationTemplates(),
			fetchAutomationPolicies(),
			fetchTelegramCalls()
		]);

		const accounts = accountResponse.items;
		const chats = chatResponse.items;
		const calls = callResponse.items;
		let nextChatId = selectedTelegramChatId;
		let nextCallId = selectedTelegramCallId;
		if (!chats.some((chat) => chat.provider_chat_id === nextChatId)) {
			nextChatId = chats[0]?.provider_chat_id ?? '';
		}
		if (!calls.some((call) => call.call_id === nextCallId)) {
			nextCallId = calls[0]?.call_id ?? '';
		}
		const selectedChat = chats.find((chat) => chat.provider_chat_id === nextChatId) ?? null;
		const messages = selectedChat
			? mergeTelegramMessages(
					messageResponse.items,
					(
						await fetchTelegramMessages(
							selectedChat.account_id,
							selectedChat.provider_chat_id,
							TELEGRAM_SELECTED_CHAT_MESSAGE_LIMIT
						)
					).items
				)
			: messageResponse.items;

		let transcript: CallTranscript | null = null;
		if (nextCallId) {
			try {
				const response = await fetchCallTranscript(nextCallId);
				transcript = response.transcript;
			} catch {
				/* transcript optional */
			}
		}
		const runtimeStatuses = await loadTelegramRuntimeStatuses(accounts, chats);

		return {
			capabilities: capabilityResponse,
			accounts,
			chats,
			messages,
			templates: templateResponse.items,
			policies: policyResponse.items,
			calls,
			transcript,
			runtimeStatuses,
			selectedChatId: nextChatId,
			selectedCallId: nextCallId,
			error: ''
		};
	} catch (error) {
		return {
			capabilities: null,
			accounts: [],
			chats: [],
			messages: [],
			templates: [],
			policies: [],
			calls: [],
			transcript: null,
			runtimeStatuses: {},
			selectedChatId: selectedTelegramChatId,
			selectedCallId: selectedTelegramCallId,
			error: telegramWorkspaceErrorMessage(error)
		};
	}
}

function mergeTelegramMessages(
	recentMessages: TelegramMessage[],
	selectedChatMessages: TelegramMessage[]
): TelegramMessage[] {
	const byId = new Map<string, TelegramMessage>();
	for (const message of recentMessages) {
		byId.set(message.message_id, message);
	}
	for (const message of selectedChatMessages) {
		byId.set(message.message_id, message);
	}
	return Array.from(byId.values());
}

function telegramWorkspaceErrorMessage(error: unknown): string {
	if (!(error instanceof Error)) return 'Unknown Telegram workspace error';
	if (error.message === 'Failed to fetch') {
		return 'Telegram backend is unreachable at the configured API URL.';
	}
	return error.message;
}

async function loadTelegramRuntimeStatuses(
	accounts: TelegramAccount[],
	chats: TelegramChat[]
): Promise<Record<string, TelegramRuntimeStatus>> {
	const accountIds = Array.from(
		new Set([
			...accounts.map((account) => account.account_id),
			...chats.map((chat) => chat.account_id)
		].filter(Boolean))
	);
	const entries = await Promise.all(
		accountIds.map(async (accountId) => {
			try {
				const status = await fetchTelegramRuntimeStatus(accountId);
				return [accountId, status] as const;
			} catch {
				return null;
			}
		})
	);

	return Object.fromEntries(entries.filter((entry): entry is [string, TelegramRuntimeStatus] => entry !== null));
}
