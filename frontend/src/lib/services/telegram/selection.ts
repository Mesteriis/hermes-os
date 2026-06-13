import type { TelegramCall, TelegramChat } from '$lib/api';

export function selectTelegramChat(
	chat: TelegramChat,
	messageForm: Record<string, unknown>,
	policyForm: Record<string, unknown>,
	sendForm: Record<string, unknown>,
	callForm: Record<string, unknown>,
	transcriptForm: Record<string, unknown>
): {
	messageForm: Record<string, unknown>;
	policyForm: Record<string, unknown>;
	sendForm: Record<string, unknown>;
	callForm: Record<string, unknown>;
	transcriptForm: Record<string, unknown>;
} {
	return {
		messageForm: { ...messageForm, account_id: chat.account_id, provider_chat_id: chat.provider_chat_id, chat_kind: telegramChatKindValue(chat.chat_kind), chat_title: chat.title },
		policyForm: { ...policyForm, account_id: chat.account_id, allowed_chat_ids_text: chat.provider_chat_id },
		sendForm: { ...sendForm, provider_chat_id: chat.provider_chat_id },
		callForm: { ...callForm, account_id: chat.account_id, provider_chat_id: chat.provider_chat_id },
		transcriptForm: { ...transcriptForm, account_id: chat.account_id, provider_chat_id: chat.provider_chat_id }
	};
}

export function selectTelegramCall(
	call: TelegramCall,
	callForm: Record<string, unknown>,
	transcriptForm: Record<string, unknown>
): {
	callForm: Record<string, unknown>;
	transcriptForm: Record<string, unknown>;
} {
	return {
		callForm: {
			...callForm,
			call_id: call.call_id,
			account_id: call.account_id,
			provider_call_id: call.provider_call_id,
			provider_chat_id: call.provider_chat_id,
			direction: call.direction,
			call_state: call.call_state,
			started_at: call.started_at ?? '',
			ended_at: call.ended_at ?? '',
			transcription_policy_id: call.transcription_policy_id ?? '',
			metadata_text: JSON.stringify(call.metadata, null, 2)
		},
		transcriptForm: {
			...transcriptForm,
			account_id: call.account_id,
			provider_chat_id: call.provider_chat_id
		}
	};
}

function telegramChatKindValue(value: string): 'private' | 'group' | 'channel' | 'bot' {
	if (value === 'group' || value === 'channel' || value === 'bot') {
		return value;
	}
	return 'private';
}
