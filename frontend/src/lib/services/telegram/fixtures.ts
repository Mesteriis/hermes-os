import {
	ingestTelegramFixtureMessage,
	sendTelegramManualMessage,
	type TelegramManualSendResponse
} from '$lib/api';

export async function ingestTelegramMessageFixture(params: {
	account_id: string;
	provider_chat_id: string;
	provider_message_id: string;
	chat_kind: 'private' | 'group' | 'channel' | 'bot';
	chat_title: string;
	sender_id: string;
	sender_display_name: string;
	text: string;
	import_batch_id: string;
	occurred_at: string;
	delivery_state: string;
}): Promise<{
	message: string;
	error: string;
	providerChatId: string;
	nextProviderMessageId: string;
	nextOccurredAt: string;
}> {
	try {
		const providerMessageId = params.provider_message_id.trim() || `fixture-msg-${crypto.randomUUID()}`;
		const result = await ingestTelegramFixtureMessage({
			account_id: params.account_id,
			provider_chat_id: params.provider_chat_id,
			provider_message_id: providerMessageId,
			chat_kind: params.chat_kind,
			chat_title: params.chat_title,
			sender_id: params.sender_id,
			sender_display_name: params.sender_display_name,
			text: params.text,
			import_batch_id: params.import_batch_id,
			occurred_at: params.occurred_at || new Date().toISOString(),
			delivery_state: params.delivery_state as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
		});
		return {
			message: `Telegram message ${result.message_id} projected`,
			error: '',
			providerChatId: params.provider_chat_id,
			nextProviderMessageId: `fixture-msg-${crypto.randomUUID()}`,
			nextOccurredAt: new Date().toISOString()
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram fixture ingest failed',
			providerChatId: params.provider_chat_id,
			nextProviderMessageId: params.provider_message_id,
			nextOccurredAt: params.occurred_at
		};
	}
}

export async function sendTelegramManualMessageFromUi(params: {
	account_id: string;
	provider_chat_id: string;
	text: string;
}): Promise<{
	result: TelegramManualSendResponse | null;
	message: string;
	error: string;
	providerChatId: string;
	nextText: string;
}> {
	try {
		const result = await sendTelegramManualMessage({
			command_id: `telegram-manual-send-${crypto.randomUUID()}`,
			account_id: params.account_id,
			provider_chat_id: params.provider_chat_id,
			text: params.text
		});
		return {
			result,
			message: `Telegram message ${result.status} with preview hash ${result.rendered_preview_hash}`,
			error: '',
			providerChatId: result.provider_chat_id,
			nextText: ''
		};
	} catch (error) {
		return {
			result: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram manual send failed',
			providerChatId: params.provider_chat_id,
			nextText: params.text
		};
	}
}
