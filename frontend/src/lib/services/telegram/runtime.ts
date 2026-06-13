import {
	downloadTelegramMedia,
	startTelegramRuntime,
	syncTelegramChats,
	syncTelegramHistory,
	type TelegramChat,
	type TelegramMediaDownloadRequest,
	type TelegramMediaDownloadResponse,
	type TelegramRuntimeStatus
} from '$lib/api';

export async function startTelegramRuntimeFromUi(accountId: string): Promise<{
	status: TelegramRuntimeStatus | null;
	message: string;
	error: string;
}> {
	try {
		const status = await startTelegramRuntime({ account_id: accountId });
		return {
			status,
			message: `Telegram runtime ${status.status} for ${status.account_id}`,
			error: ''
		};
	} catch (error) {
		return {
			status: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram runtime start failed'
		};
	}
}

export async function syncTelegramChatsFromUi(accountId: string): Promise<{
	message: string;
	error: string;
	accountId: string;
}> {
	try {
		const result = await syncTelegramChats({
			account_id: accountId,
			limit: 100
		});
		return {
			message: `Telegram chats synced: ${result.synced_count}`,
			error: '',
			accountId: result.account_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram chat sync failed',
			accountId
		};
	}
}

export async function syncTelegramSelectedHistoryFromUi(params: {
	account_id: string;
	provider_chat_id: string;
	chat_kind?: TelegramChat['chat_kind'];
	mode?: 'latest' | 'older' | 'full';
	from_message_id?: number;
}): Promise<{
	message: string;
	error: string;
	providerChatId: string;
	hasMore: boolean;
	nextFromMessageId: number | null;
}> {
	try {
		const mode = params.mode ?? (params.chat_kind === 'private' ? 'full' : 'latest');
		const request = {
			account_id: params.account_id,
			provider_chat_id: params.provider_chat_id,
			mode,
			limit: 100
		};
		if (params.from_message_id != null) {
			Object.assign(request, { from_message_id: params.from_message_id });
		}
		const result = await syncTelegramHistory(request);
		return {
			message: `Telegram history synced: ${result.synced_count}`,
			error: '',
			providerChatId: result.provider_chat_id,
			hasMore: result.has_more,
			nextFromMessageId: result.next_from_message_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram history sync failed',
			providerChatId: params.provider_chat_id,
			hasMore: false,
			nextFromMessageId: null
		};
	}
}

export async function syncTelegramOlderHistoryFromUi(params: {
	account_id: string;
	provider_chat_id: string;
	from_message_id: number;
}): Promise<{
	message: string;
	error: string;
	providerChatId: string;
	hasMore: boolean;
	nextFromMessageId: number | null;
}> {
	return syncTelegramSelectedHistoryFromUi({
		account_id: params.account_id,
		provider_chat_id: params.provider_chat_id,
		from_message_id: params.from_message_id,
		mode: 'older'
	});
}

export async function downloadTelegramMediaFromUi(
	request: TelegramMediaDownloadRequest
): Promise<{
	response: TelegramMediaDownloadResponse | null;
	message: string;
	error: string;
}> {
	try {
		const response = await downloadTelegramMedia(request);
		const identifier = response.attachment_id ?? response.local_path ?? `tdlib file ${response.tdlib_file_id}`;
		return {
			response,
			message: `Telegram media ${response.status}: ${identifier}`,
			error: ''
		};
	} catch (error) {
		return {
			response: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram media download failed'
		};
	}
}
