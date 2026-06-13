import { ApiClient } from '../client';
import type {
	TelegramCapabilitiesResponse,
	TelegramCallListResponse,
	TelegramChatListResponse,
	TelegramMessageListResponse,
	TelegramQrLoginStatusResponse,
	TelegramQrLoginCancelResponse,
	TelegramQrLoginStartRequest,
	TelegramQrLoginPasswordRequest,
	TelegramAccountSetupRequest,
	TelegramLiveAccountSetupRequest,
	TelegramAccountSetupResponse,
	TelegramAccountListResponse,
	TelegramAccountLifecycleResponse,
	TelegramRuntimeStartRequest,
	TelegramRuntimeStatus,
	TelegramChatSyncRequest,
	TelegramChatSyncResponse,
	TelegramHistorySyncRequest,
	TelegramHistorySyncResponse,
	TelegramMediaDownloadRequest,
	TelegramMediaDownloadResponse,
	TelegramCallRequest,
	TelegramCall,
	TelegramFixtureMessageRequest,
	TelegramManualSendRequest,
	TelegramManualSendResponse,
	TelegramMessageIngestResponse,
	TelegramSendDryRunRequest,
	TelegramSendDryRunResponse,
	CallTranscriptFixtureRequest,
	CallTranscript,
	CallTranscriptResponse,
	AutomationPolicyListResponse,
	AutomationTemplateListResponse,
	AutomationPolicyRequest,
	AutomationTemplateRequest,
	AutomationPolicy,
	AutomationTemplate
} from '../types';

export async function fetchTelegramCapabilities(): Promise<TelegramCapabilitiesResponse> {
	return ApiClient.instance.get<TelegramCapabilitiesResponse>(
		'/api/v1/telegram/capabilities',
		'Telegram capabilities request failed'
	);
}

export async function fetchTelegramCalls(accountId?: string, limit = 50): Promise<TelegramCallListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return ApiClient.instance.get<TelegramCallListResponse>(
		`/api/v1/calls?${params.toString()}`,
		'Telegram call request failed'
	);
}

export async function fetchTelegramChats(accountId?: string, limit = 50): Promise<TelegramChatListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return ApiClient.instance.get<TelegramChatListResponse>(
		`/api/v1/telegram/chats?${params.toString()}`,
		'Telegram chats request failed'
	);
}

export async function fetchTelegramMessages(
	accountId?: string,
	providerChatId?: string,
	limit = 50
): Promise<TelegramMessageListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	if (providerChatId?.trim()) {
		params.set('provider_chat_id', providerChatId.trim());
	}
	return ApiClient.instance.get<TelegramMessageListResponse>(
		`/api/v1/telegram/messages?${params.toString()}`,
		'Telegram messages request failed'
	);
}

export async function fetchTelegramRuntimeStatus(accountId: string): Promise<TelegramRuntimeStatus> {
	const params = new URLSearchParams({ account_id: accountId.trim() });
	return ApiClient.instance.get<TelegramRuntimeStatus>(
		`/api/v1/telegram/runtime/status?${params.toString()}`,
		'Telegram runtime status request failed'
	);
}

export async function startTelegramRuntime(
	request: TelegramRuntimeStartRequest
): Promise<TelegramRuntimeStatus> {
	return ApiClient.instance.post<TelegramRuntimeStatus>(
		'/api/v1/telegram/runtime/start',
		request,
		'Telegram runtime start failed'
	);
}

export async function syncTelegramChats(
	request: TelegramChatSyncRequest
): Promise<TelegramChatSyncResponse> {
	return ApiClient.instance.post<TelegramChatSyncResponse>(
		'/api/v1/telegram/sync/chats',
		request,
		'Telegram chat sync failed'
	);
}

export async function syncTelegramHistory(
	request: TelegramHistorySyncRequest
): Promise<TelegramHistorySyncResponse> {
	return ApiClient.instance.post<TelegramHistorySyncResponse>(
		'/api/v1/telegram/sync/history',
		request,
		'Telegram history sync failed'
	);
}

export async function downloadTelegramMedia(
	request: TelegramMediaDownloadRequest
): Promise<TelegramMediaDownloadResponse> {
	return ApiClient.instance.post<TelegramMediaDownloadResponse>(
		'/api/v1/telegram/media/download',
		request,
		'Telegram media download failed'
	);
}

export async function fetchTelegramQrLoginStatus(setupId: string): Promise<TelegramQrLoginStatusResponse> {
	return ApiClient.instance.get<TelegramQrLoginStatusResponse>(
		`/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}`,
		'Telegram QR login status request failed'
	);
}

export async function cancelTelegramQrLogin(setupId: string): Promise<TelegramQrLoginCancelResponse> {
	return ApiClient.instance.delete<TelegramQrLoginCancelResponse>(
		`/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}`,
		'Telegram QR login cancel request failed'
	);
}

export async function startTelegramQrLogin(
	request: TelegramQrLoginStartRequest
): Promise<TelegramQrLoginStatusResponse> {
	return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
		'/api/v1/telegram/login/qr/start',
		request,
		'Telegram QR login start failed'
	);
}

export async function submitTelegramQrLoginPassword(
	setupId: string,
	request: TelegramQrLoginPasswordRequest
): Promise<TelegramQrLoginStatusResponse> {
	return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
		`/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}/password`,
		request,
		'Telegram QR login password submit failed'
	);
}

export async function setupTelegramAccount(
	request: TelegramLiveAccountSetupRequest
): Promise<TelegramAccountSetupResponse> {
	return ApiClient.instance.post<TelegramAccountSetupResponse>(
		'/api/v1/telegram/accounts',
		request,
		'Telegram account setup request failed'
	);
}

export async function setupTelegramFixtureAccount(
	request: TelegramAccountSetupRequest
): Promise<TelegramAccountSetupResponse> {
	return ApiClient.instance.post<TelegramAccountSetupResponse>(
		'/api/v1/telegram/accounts/fixture',
		request,
		'Telegram account setup request failed'
	);
}

export async function fetchTelegramAccounts(includeRemoved = false): Promise<TelegramAccountListResponse> {
	const params = new URLSearchParams();
	if (includeRemoved) {
		params.set('include_removed', 'true');
	}
	const query = params.toString();
	return ApiClient.instance.get<TelegramAccountListResponse>(
		`/api/v1/telegram/accounts${query ? `?${query}` : ''}`,
		'Telegram account list request failed'
	);
}

export async function logoutTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
	return ApiClient.instance.post<TelegramAccountLifecycleResponse>(
		`/api/v1/telegram/accounts/${encodeURIComponent(accountId)}/logout`,
		{},
		'Telegram account logout request failed'
	);
}

export async function removeTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
	return ApiClient.instance.delete<TelegramAccountLifecycleResponse>(
		`/api/v1/telegram/accounts/${encodeURIComponent(accountId)}`,
		'Telegram account remove request failed'
	);
}

export async function saveTelegramCall(request: TelegramCallRequest): Promise<TelegramCall> {
	return ApiClient.instance.post<TelegramCall>('/api/v1/calls', request, 'Telegram call save failed');
}

export async function ingestTelegramFixtureMessage(
	request: TelegramFixtureMessageRequest
): Promise<TelegramMessageIngestResponse> {
	return ApiClient.instance.post<TelegramMessageIngestResponse>(
		'/api/v1/telegram/messages',
		request,
		'Telegram fixture message request failed'
	);
}

export async function sendTelegramManualMessage(
	request: TelegramManualSendRequest
): Promise<TelegramManualSendResponse> {
	return ApiClient.instance.post<TelegramManualSendResponse>(
		'/api/v1/telegram/messages/send',
		request,
		'Telegram manual send failed'
	);
}

export async function dryRunTelegramSend(
	request: TelegramSendDryRunRequest
): Promise<TelegramSendDryRunResponse> {
	return ApiClient.instance.post<TelegramSendDryRunResponse>(
		'/api/v1/policies/telegram-send/dry-run',
		request,
		'Telegram send dry-run failed'
	);
}

export async function saveCallTranscriptFixture(
	callId: string,
	request: CallTranscriptFixtureRequest
): Promise<CallTranscript> {
	return ApiClient.instance.post<CallTranscript>(
		`/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
		request,
		'Call transcript save failed'
	);
}

export async function fetchCallTranscript(callId: string): Promise<CallTranscriptResponse> {
	return ApiClient.instance.get<CallTranscriptResponse>(
		`/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
		'Call transcript request failed'
	);
}

export async function fetchAutomationPolicies(): Promise<AutomationPolicyListResponse> {
	return ApiClient.instance.get<AutomationPolicyListResponse>(
		'/api/v1/policies',
		'Automation policy request failed'
	);
}

export async function fetchAutomationTemplates(): Promise<AutomationTemplateListResponse> {
	return ApiClient.instance.get<AutomationTemplateListResponse>(
		'/api/v1/policies/templates',
		'Automation template request failed'
	);
}

export async function saveAutomationPolicy(request: AutomationPolicyRequest): Promise<AutomationPolicy> {
	return ApiClient.instance.post<AutomationPolicy>('/api/v1/policies', request, 'Automation policy save failed');
}

export async function saveAutomationTemplate(request: AutomationTemplateRequest): Promise<AutomationTemplate> {
	return ApiClient.instance.post<AutomationTemplate>(
		'/api/v1/policies/templates',
		request,
		'Automation template save failed'
	);
}
