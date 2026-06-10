import { ApiClient } from '../client';
import type {
	WhatsappCapabilitiesResponse,
	WhatsappWebMessageListResponse,
	WhatsappWebSessionListResponse,
	WhatsappWebAccountSetupRequest,
	WhatsappWebAccountSetupResponse,
	WhatsappWebFixtureMessageRequest,
	WhatsappWebMessageIngestResponse
} from '../types';

export async function fetchWhatsappCapabilities(): Promise<WhatsappCapabilitiesResponse> {
	return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
		'/api/v1/whatsapp/capabilities',
		'WhatsApp capabilities request failed'
	);
}

export async function fetchWhatsappWebMessages(
	accountId?: string,
	providerChatId?: string,
	limit = 50
): Promise<WhatsappWebMessageListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	if (providerChatId?.trim()) {
		params.set('provider_chat_id', providerChatId.trim());
	}
	return ApiClient.instance.get<WhatsappWebMessageListResponse>(
		`/api/v1/whatsapp/messages?${params.toString()}`,
		'WhatsApp Web messages request failed'
	);
}

export async function fetchWhatsappWebSessions(
	accountId?: string,
	limit = 50
): Promise<WhatsappWebSessionListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return ApiClient.instance.get<WhatsappWebSessionListResponse>(
		`/api/v1/whatsapp/sessions?${params.toString()}`,
		'WhatsApp Web sessions request failed'
	);
}

export async function setupWhatsappWebFixtureAccount(
	request: WhatsappWebAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
	return ApiClient.instance.post<WhatsappWebAccountSetupResponse>(
		'/api/v1/whatsapp/accounts/fixture',
		request,
		'WhatsApp Web account setup request failed'
	);
}

export async function ingestWhatsappWebFixtureMessage(
	request: WhatsappWebFixtureMessageRequest
): Promise<WhatsappWebMessageIngestResponse> {
	return ApiClient.instance.post<WhatsappWebMessageIngestResponse>(
		'/api/v1/whatsapp/messages',
		request,
		'WhatsApp Web fixture message request failed'
	);
}
