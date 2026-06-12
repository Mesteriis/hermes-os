import {
	fetchTelegramCapabilities,
	fetchTelegramChats,
	fetchTelegramMessages,
	fetchTelegramRuntimeStatus,
	startTelegramRuntime,
	syncTelegramChats,
	syncTelegramHistory,
	fetchAutomationTemplates,
	fetchAutomationPolicies,
	fetchTelegramCalls,
	fetchCallTranscript,
	fetchTelegramQrLoginStatus,
	ingestTelegramFixtureMessage,
	sendTelegramManualMessage,
	downloadTelegramMedia,
	saveAutomationPolicy,
	saveAutomationTemplate,
	dryRunTelegramSend,
	saveTelegramCall,
	saveCallTranscriptFixture,
	setupTelegramAccount,
	setupTelegramFixtureAccount,
	startTelegramQrLogin,
	submitTelegramQrLoginPassword,
	type TelegramCapabilitiesResponse,
	type TelegramChat,
	type TelegramMessage,
	type TelegramRuntimeStatus,
	type AutomationTemplate,
	type AutomationPolicy,
	type TelegramCall,
	type CallTranscript,
	type TelegramAccountSetupResponse,
	type TelegramLiveAccountSetupRequest,
	type TelegramProviderKind,
	type TelegramQrLoginStatusResponse,
	type TelegramManualSendResponse,
	type TelegramSendDryRunResponse,
	type TelegramMediaDownloadRequest,
	type TelegramMediaDownloadResponse
} from '$lib/api';
import { formatDateTime } from './formatting';

export const TELEGRAM_SELECTED_CHAT_MESSAGE_LIMIT = 5000;
export const TELEGRAM_CHAT_METADATA_LIMIT = 5000;

export async function loadTelegramWorkspace(
	selectedTelegramChatId: string,
	selectedTelegramCallId: string
): Promise<{
	capabilities: TelegramCapabilitiesResponse | null;
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
			chatResponse,
			messageResponse,
			templateResponse,
			policyResponse,
			callResponse
		] = await Promise.all([
			fetchTelegramCapabilities(),
			fetchTelegramChats(undefined, TELEGRAM_CHAT_METADATA_LIMIT),
			fetchTelegramMessages(),
			fetchAutomationTemplates(),
			fetchAutomationPolicies(),
			fetchTelegramCalls()
		]);

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
			} catch { /* transcript optional */ }
		}
		const runtimeStatuses = await loadTelegramRuntimeStatuses(chats);

		return {
			capabilities: capabilityResponse,
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

async function loadTelegramRuntimeStatuses(chats: TelegramChat[]): Promise<Record<string, TelegramRuntimeStatus>> {
	const accountIds = Array.from(new Set(chats.map((chat) => chat.account_id).filter(Boolean)));
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

export async function saveTelegramAccountFromWizard(params: {
	accountForm: {
		account_id: string;
		provider_kind: TelegramProviderKind;
		display_name: string;
		external_account_id: string;
		api_id: string;
		api_hash: string;
		bot_token: string;
		session_encryption_key: string;
		tdlib_data_path: string;
		transcription_enabled: boolean;
	};
	authMethod: string;
	qrLogin: TelegramQrLoginStatusResponse | null;
	isFixtureSetup: boolean;
}): Promise<{
	message: string;
	error: string;
	accountId: string;
	providerKind: string;
}> {
	const { accountForm, authMethod, qrLogin, isFixtureSetup } = params;
	const providerKind =
		authMethod === 'bot_token'
			? 'telegram_bot'
			: authMethod === 'phone' || authMethod === 'qr'
				? 'telegram_user'
				: accountForm.provider_kind;
	const isQrAuthorizedAccount = authMethod === 'qr' && qrLogin?.status === 'ready';

	try {
		let result: TelegramAccountSetupResponse;
		if (isFixtureSetup) {
			result = await setupTelegramFixtureAccount({
				account_id: accountForm.account_id,
				provider_kind: providerKind,
				display_name: accountForm.display_name,
				external_account_id: accountForm.external_account_id,
				tdlib_data_path: accountForm.tdlib_data_path || undefined,
				transcription_enabled: authMethod === 'qr' ? false : accountForm.transcription_enabled
			});
		} else {
			const request: TelegramLiveAccountSetupRequest = {
				account_id: accountForm.account_id,
				provider_kind: providerKind,
				display_name: accountForm.display_name,
				external_account_id: accountForm.external_account_id,
				tdlib_data_path: accountForm.tdlib_data_path || undefined,
				transcription_enabled: authMethod === 'qr' ? false : accountForm.transcription_enabled
			};
			if (providerKind === 'telegram_user' && isQrAuthorizedAccount) {
				request.qr_authorized = true;
				if (accountForm.session_encryption_key) {
					request.session_encryption_key = accountForm.session_encryption_key;
				}
			} else if (providerKind === 'telegram_user') {
				const apiId = accountForm.api_id.trim();
				const apiHash = accountForm.api_hash.trim();
				if (apiId) {
					request.api_id = Number(apiId);
				}
				if (apiHash) {
					request.api_hash = apiHash;
				}
				if (accountForm.session_encryption_key) {
					request.session_encryption_key = accountForm.session_encryption_key;
				}
			} else if (providerKind === 'telegram_bot' && accountForm.bot_token) {
				request.bot_token = accountForm.bot_token;
			}
			result = await setupTelegramAccount(request);
		}
		const runtimeLabel =
			authMethod === 'qr' && qrLogin?.status === 'ready'
				? 'saved after QR authorization'
				: result.runtime === 'live_blocked'
					? 'saved as live-blocked'
					: 'saved';
		const message = `${providerKindLabel(result.provider_kind)} account ${result.account_id} ${runtimeLabel}`;
		return {
			message,
			error: '',
			accountId: result.account_id,
			providerKind: result.provider_kind
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram account setup failed',
			accountId: accountForm.account_id,
			providerKind
		};
	}
}

export async function saveReadyTelegramQrAccountFromWizard(
	authMethod: string,
	qrLogin: TelegramQrLoginStatusResponse | null
): Promise<{ shouldSave: boolean }> {
	if (authMethod !== 'qr' || qrLogin?.status !== 'ready') {
		return { shouldSave: false };
	}
	return { shouldSave: true };
}

export async function startTelegramQrLoginFromWizard(params: {
	accountForm: {
		account_id: string;
		display_name: string;
		api_id: string;
		api_hash: string;
		session_encryption_key: string;
		tdlib_data_path?: string;
	};
	capabilities: TelegramCapabilitiesResponse | null;
	externalAccountId: string;
}): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	const { accountForm, capabilities, externalAccountId } = params;

	if (capabilities && !capabilities.tdjson_runtime_available) {
		return {
			qrLogin: null,
			message: '',
			error: 'TDLib JSON runtime is not available in the running backend'
		};
	}

	if (capabilities && !capabilities.telegram_app_credentials_configured) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram app credentials are not configured in the backend environment'
		};
	}

	try {
		const result = await startTelegramQrLogin({
			account_id: accountForm.account_id,
			display_name: accountForm.display_name,
			external_account_id: externalAccountId,
			session_encryption_key: accountForm.session_encryption_key || undefined,
			tdlib_data_path: accountForm.tdlib_data_path || undefined,
			transcription_enabled: false
		});
		const message =
			result.status === 'waiting_qr_scan'
				? 'Scan the Telegram QR code to continue'
				: result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login start failed'
		};
	}
}

export async function submitTelegramQrPasswordFromWizard(
	qrLogin: TelegramQrLoginStatusResponse,
	password: string
): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	if (qrLogin.status !== 'waiting_password') {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram QR login is not waiting for a password'
		};
	}

	if (!password) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram 2-step verification password is required'
		};
	}

	try {
		const result = await submitTelegramQrLoginPassword(
			qrLogin.setup_id,
			{ password }
		);
		const message = result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login password submit failed'
		};
	}
}

export function shouldPollTelegramQrLoginStatus(
	status: TelegramQrLoginStatusResponse['status'] | null | undefined
) {
	return status === 'waiting_qr_scan' || status === 'waiting_password';
}

export function submitTelegramQrStepFromWizard(qrLogin: TelegramQrLoginStatusResponse | null) {
	if (qrLogin?.status === 'waiting_password') {
		return 'password';
	}
	return 'start';
}

export async function refreshTelegramQrLoginStatus(
	qrLogin: TelegramQrLoginStatusResponse
): Promise<{
	qrLogin: TelegramQrLoginStatusResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await fetchTelegramQrLoginStatus(
			qrLogin.setup_id
		);
		const message = result.message ?? `Telegram QR login status: ${result.status}`;
		return { qrLogin: result, message, error: '' };
	} catch (error) {
		return {
			qrLogin: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram QR login status request failed'
		};
	}
}

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

export async function saveTelegramAutomationTemplate(params: {
	template_id: string;
	name: string;
	body_template: string;
	required_variables_text: string;
}): Promise<{
	message: string;
	error: string;
	templateId: string;
}> {
	try {
		const template = await saveAutomationTemplate({
			template_id: params.template_id,
			name: params.name,
			body_template: params.body_template,
			required_variables: params.required_variables_text
				.split(',')
				.map((item) => item.trim())
				.filter(Boolean)
		});
		return {
			message: `Template ${template.template_id} saved`,
			error: '',
			templateId: template.template_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Automation template save failed',
			templateId: params.template_id
		};
	}
}

export async function saveTelegramAutomationPolicy(params: {
	policy_id: string;
	template_id: string;
	name: string;
	enabled: boolean;
	account_id: string;
	allowed_chat_ids_text: string;
	trigger_kind: string;
	max_sends_per_hour: number;
	quiet_hours_text: string;
	expires_at: string;
	conditions_text: string;
}): Promise<{
	message: string;
	error: string;
	policyId: string;
}> {
	try {
		const policy = await saveAutomationPolicy({
			policy_id: params.policy_id,
			template_id: params.template_id,
			name: params.name,
			enabled: params.enabled,
			account_id: params.account_id,
			allowed_chat_ids: params.allowed_chat_ids_text
				.split(',')
				.map((item) => item.trim())
				.filter(Boolean),
			trigger_kind: params.trigger_kind,
			max_sends_per_hour: Number(params.max_sends_per_hour),
			quiet_hours: parseJsonObject(params.quiet_hours_text, 'quiet hours'),
			expires_at: params.expires_at.trim() || null,
			conditions: parseJsonObject(params.conditions_text, 'conditions')
		});
		return {
			message: `Policy ${policy.policy_id} saved`,
			error: '',
			policyId: policy.policy_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Automation policy save failed',
			policyId: params.policy_id
		};
	}
}

export async function runTelegramAutomationDryRun(params: {
	policy_id: string;
	provider_chat_id: string;
	variables_text: string;
	source_context_text: string;
}): Promise<{
	result: TelegramSendDryRunResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await dryRunTelegramSend({
			command_id: `telegram-dry-run-${crypto.randomUUID()}`,
			policy_id: params.policy_id,
			provider_chat_id: params.provider_chat_id,
			variables: parseStringMap(params.variables_text, 'variables'),
			source_context: parseJsonObject(params.source_context_text, 'source context')
		});
		return {
			result,
			message: `Dry-run accepted with preview hash ${result.rendered_preview_hash}`,
			error: ''
		};
	} catch (error) {
		return {
			result: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram send dry-run failed'
		};
	}
}

export async function saveTelegramCallFixture(params: {
	call_id: string;
	account_id: string;
	provider_call_id: string;
	provider_chat_id: string;
	direction: 'incoming' | 'outgoing';
	call_state: 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed';
	started_at: string;
	ended_at: string;
	transcription_policy_id: string;
	metadata_text: string;
}): Promise<{
	message: string;
	error: string;
	callId: string;
}> {
	try {
		const call = await saveTelegramCall({
			call_id: params.call_id,
			account_id: params.account_id,
			provider_call_id: params.provider_call_id,
			provider_chat_id: params.provider_chat_id,
			direction: params.direction,
			call_state: params.call_state,
			started_at: params.started_at.trim() || null,
			ended_at: params.ended_at.trim() || null,
			transcription_policy_id: params.transcription_policy_id.trim() || null,
			metadata: parseJsonObject(params.metadata_text, 'call metadata')
		});
		return {
			message: `Call ${call.call_id} saved`,
			error: '',
			callId: call.call_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram call save failed',
			callId: params.call_id
		};
	}
}

export async function saveCallTranscriptFixtureFromUi(params: {
	transcript_id: string;
	account_id: string;
	provider_chat_id: string;
	source_audio_ref: string;
	language_code: string;
	always_on_policy: boolean;
	selectedCallId: string;
}): Promise<{
	transcript: CallTranscript | null;
	message: string;
	error: string;
}> {
	if (!params.selectedCallId) {
		return { transcript: null, message: '', error: '' };
	}
	try {
		const transcript = await saveCallTranscriptFixture(
			params.selectedCallId,
			{
				transcript_id: params.transcript_id,
				account_id: params.account_id,
				provider_chat_id: params.provider_chat_id,
				source_audio_ref: params.source_audio_ref,
				language_code: params.language_code || undefined,
				always_on_policy: params.always_on_policy
			}
		);
		return {
			transcript,
			message: `Transcript ${transcript.transcript_id} saved`,
			error: ''
		};
	} catch (error) {
		return {
			transcript: null,
			message: '',
			error: error instanceof Error ? error.message : 'Call transcript save failed'
		};
	}
}

export async function loadSelectedCallTranscript(
	callId: string
): Promise<{
	transcript: CallTranscript | null;
	error: string;
}> {
	if (!callId) {
		return { transcript: null, error: '' };
	}
	try {
		const response = await fetchCallTranscript(callId);
		return { transcript: response.transcript, error: '' };
	} catch (error) {
		return {
			transcript: null,
			error: error instanceof Error ? error.message : 'Call transcript request failed'
		};
	}
}

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

export function telegramMessageTime(message: TelegramMessage) {
	return formatDateTime(message.occurred_at ?? message.projected_at);
}

export type TelegramChatFilter =
	| 'all'
	| 'unread'
	| 'mentions'
	| 'pinned'
	| 'projects'
	| 'bots'
	| 'archived';

export type TelegramThreadTab = 'messages' | 'files' | 'links' | 'topics' | 'pinned' | 'timeline';
export type TelegramRailTab = 'context' | 'members' | 'about';

export type TelegramChatFilterCount = {
	filter: TelegramChatFilter;
	count: number;
};

export type TelegramChatGroupFilter = {
	id: string;
	label: string;
	source: 'local' | 'telegram';
	count: number;
	icon: string;
};

export type TelegramAttachmentHint = {
	id: string;
	kind: 'document' | 'photo' | 'video' | 'audio' | 'voice' | 'file';
	fileName: string;
	mimeType: string | null;
	sizeBytes: number | null;
	tdlibFileId: number | null;
	providerAttachmentId: string;
	downloadState: 'remote' | 'downloading' | 'downloaded' | 'unknown';
	localPath: string | null;
	messageId: string;
};

export type TelegramLinkHint = {
	url: string;
	label: string;
	messageId: string;
	occurredAt: string | null;
};

export function telegramMessagesForChat(
	messages: TelegramMessage[],
	providerChatId: string | null | undefined
): TelegramMessage[] {
	if (!providerChatId) return [];
	return messages.filter((message) => message.provider_chat_id === providerChatId);
}

export function telegramMessagesChronological(messages: TelegramMessage[]): TelegramMessage[] {
	return messages.slice().sort((left, right) => {
		const leftTime = new Date(left.occurred_at ?? left.projected_at).getTime();
		const rightTime = new Date(right.occurred_at ?? right.projected_at).getTime();
		return leftTime - rightTime;
	});
}

export function telegramOldestTdlibMessageId(messages: TelegramMessage[]): number | null {
	const ids = messages
		.map((message) => telegramTdlibMessageId(message.provider_message_id))
		.filter((value): value is number => value !== null);
	return ids.length ? Math.min(...ids) : null;
}

function telegramTdlibMessageId(providerMessageId: string): number | null {
	const id = providerMessageId.split(':').at(-1)?.trim() ?? '';
	if (!id) return null;
	const parsed = Number.parseInt(id, 10);
	return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

export function telegramChatPreview(chat: TelegramChat, messages: TelegramMessage[]): string {
	const latestMessage = telegramMessagesForChat(messages, chat.provider_chat_id)
		.slice()
		.sort((left, right) => {
			const leftTime = new Date(left.occurred_at ?? left.projected_at).getTime();
			const rightTime = new Date(right.occurred_at ?? right.projected_at).getTime();
			return rightTime - leftTime;
		})[0];
	if (!latestMessage) {
		return `${chat.account_id} · ${chat.sync_state}`;
	}
	const sender = latestMessage.sender_display_name ?? latestMessage.sender;
	const text = latestMessage.text.trim();
	return text ? `${sender}: ${text}` : `${sender}: ${latestMessage.delivery_state}`;
}

export function telegramChatUnreadCount(chat: TelegramChat): number {
	return metadataNumber(chat.metadata, [
		'unread_count',
		'unread_message_count',
		'unread_messages',
		'tdlib_raw.unread_count'
	]);
}

export function telegramChatMentionCount(chat: TelegramChat, messages: TelegramMessage[]): number {
	const metadataCount = metadataNumber(chat.metadata, [
		'mention_count',
		'mentions_count',
		'unread_mention_count',
		'tdlib_raw.unread_mention_count'
	]);
	if (metadataCount > 0) return metadataCount;
	return telegramMessagesForChat(messages, chat.provider_chat_id).filter((message) =>
		message.text.includes('@')
	).length;
}

export function telegramChatIsPinned(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['pinned', 'is_pinned', 'tdlib_raw.positions.0.is_pinned']);
}

export function telegramChatIsArchived(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['archived', 'is_archived', 'tdlib_raw.is_archived']);
}

export function telegramChatIsProject(chat: TelegramChat): boolean {
	return metadataBoolean(chat.metadata, ['project', 'is_project', 'hermes_project']) ||
		metadataString(chat.metadata, ['category', 'folder', 'label']).toLowerCase().includes('project');
}

export function telegramChatIsBot(chat: TelegramChat): boolean {
	return chat.chat_kind === 'bot' ||
		Boolean(chat.username?.toLowerCase().endsWith('bot')) ||
		/\bbot\b/i.test(chat.title);
}

export function telegramChatFilterCounts(
	chats: TelegramChat[],
	messages: TelegramMessage[]
): TelegramChatFilterCount[] {
	return [
		{ filter: 'all', count: chats.length },
		{ filter: 'unread', count: chats.filter((chat) => telegramChatUnreadCount(chat) > 0).length },
		{
			filter: 'mentions',
			count: chats.filter((chat) => telegramChatMentionCount(chat, messages) > 0).length
		},
		{ filter: 'pinned', count: chats.filter(telegramChatIsPinned).length },
		{ filter: 'projects', count: chats.filter(telegramChatIsProject).length },
		{ filter: 'bots', count: chats.filter(telegramChatIsBot).length },
		{ filter: 'archived', count: chats.filter(telegramChatIsArchived).length }
	];
}

export function telegramChatGroupFilters(chats: TelegramChat[]): TelegramChatGroupFilter[] {
	const localGroups: TelegramChatGroupFilter[] = [
		{
			id: 'local:all',
			label: 'All Chats',
			source: 'local',
			count: chats.length,
			icon: 'tabler:messages'
		},
		{
			id: 'local:private',
			label: 'Direct',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'private').length,
			icon: 'tabler:user'
		},
		{
			id: 'local:group',
			label: 'Groups',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'group').length,
			icon: 'tabler:users-group'
		},
		{
			id: 'local:channel',
			label: 'Channels',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'channel').length,
			icon: 'tabler:speakerphone'
		},
		{
			id: 'local:bot',
			label: 'Bots',
			source: 'local',
			count: chats.filter((chat) => chat.chat_kind === 'bot' || telegramChatIsBot(chat)).length,
			icon: 'tabler:robot'
		}
	];

	const telegramFolders = new Map<string, TelegramChatGroupFilter>();
	for (const chat of chats) {
		for (const label of telegramFolderLabels(chat)) {
			const id = `telegram:${label}`;
			const current = telegramFolders.get(id);
			telegramFolders.set(id, {
				id,
				label,
				source: 'telegram',
				count: (current?.count ?? 0) + 1,
				icon: label === 'Archived' ? 'tabler:archive' : 'tabler:folder'
			});
		}
	}

	return [...localGroups, ...Array.from(telegramFolders.values()).sort((left, right) => left.label.localeCompare(right.label))];
}

export function filterTelegramChatsByGroup(
	chats: TelegramChat[],
	groupFilterId: string
): TelegramChat[] {
	if (!groupFilterId || groupFilterId === 'local:all') return chats;
	if (groupFilterId === 'local:private') return chats.filter((chat) => chat.chat_kind === 'private');
	if (groupFilterId === 'local:group') return chats.filter((chat) => chat.chat_kind === 'group');
	if (groupFilterId === 'local:channel') return chats.filter((chat) => chat.chat_kind === 'channel');
	if (groupFilterId === 'local:bot') {
		return chats.filter((chat) => chat.chat_kind === 'bot' || telegramChatIsBot(chat));
	}
	if (groupFilterId.startsWith('telegram:')) {
		const label = groupFilterId.slice('telegram:'.length);
		return chats.filter((chat) => telegramFolderLabels(chat).includes(label));
	}
	return chats;
}

export function filterTelegramChats(
	chats: TelegramChat[],
	messages: TelegramMessage[],
	query: string,
	filter: TelegramChatFilter
): TelegramChat[] {
	const normalizedQuery = query.trim().toLowerCase();
	return chats.filter((chat) => {
		if (!matchesTelegramChatFilter(chat, messages, filter)) return false;
		if (!normalizedQuery) return true;
		const searchable = [
			chat.title,
			chat.username ?? '',
			chat.account_id,
			chat.provider_chat_id,
			chat.chat_kind,
			telegramChatPreview(chat, messages)
		]
			.join(' ')
			.toLowerCase();
		return searchable.includes(normalizedQuery);
	});
}

export function telegramAttachmentHintsForMessages(
	messages: TelegramMessage[]
): TelegramAttachmentHint[] {
	return messages.flatMap(telegramMessageAttachmentHints);
}

export function telegramMessageAttachmentHints(message: TelegramMessage): TelegramAttachmentHint[] {
	const explicit = explicitAttachmentHints(message);
	if (explicit.length) return explicit;

	const tdlibRaw = metadataRecord(message.metadata, ['tdlib_raw']);
	const content = tdlibRaw ? valueRecord(tdlibRaw.content) : null;
	if (!content) return [];

	const contentType = valueString(content['@type']);
	const baseId = `${message.message_id}:${contentType || 'attachment'}`;
	if (contentType === 'messageDocument') {
		const document = valueRecord(content.document);
		const file = document ? valueRecord(document.document) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'document', {
				fileName: valueString(document?.file_name) || 'document',
				mimeType: valueString(document?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messagePhoto') {
		const photo = valueRecord(content.photo);
		const sizes = valueArray(photo?.sizes);
		const largest = valueRecord(sizes.at(-1));
		const file = largest ? valueRecord(largest.photo) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'photo', {
				fileName: `photo-${message.provider_message_id}.jpg`,
				mimeType: 'image/jpeg',
				file
			})
		];
	}
	if (contentType === 'messageVideo') {
		const video = valueRecord(content.video);
		const file = video ? valueRecord(video.video) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'video', {
				fileName: valueString(video?.file_name) || `video-${message.provider_message_id}.mp4`,
				mimeType: valueString(video?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messageAudio') {
		const audio = valueRecord(content.audio);
		const file = audio ? valueRecord(audio.audio) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'audio', {
				fileName: valueString(audio?.file_name) || valueString(audio?.title) || 'audio',
				mimeType: valueString(audio?.mime_type),
				file
			})
		];
	}
	if (contentType === 'messageVoiceNote') {
		const voice = valueRecord(content.voice_note);
		const file = voice ? valueRecord(voice.voice) : null;
		return [
			attachmentFromTdlibFile(message, baseId, 'voice', {
				fileName: `voice-${message.provider_message_id}.ogg`,
				mimeType: valueString(voice?.mime_type),
				file
			})
		];
	}
	return [];
}

export function telegramLinkHintsForMessages(messages: TelegramMessage[]): TelegramLinkHint[] {
	const seen = new Set<string>();
	const links: TelegramLinkHint[] = [];
	for (const message of messages) {
		for (const url of message.text.match(/https?:\/\/[^\s<>()]+/g) ?? []) {
			const normalized = url.replace(/[.,!?;:]+$/, '');
			if (seen.has(`${message.message_id}:${normalized}`)) continue;
			seen.add(`${message.message_id}:${normalized}`);
			links.push({
				url: normalized,
				label: normalized.replace(/^https?:\/\//, ''),
				messageId: message.message_id,
				occurredAt: message.occurred_at ?? message.projected_at
			});
		}
	}
	return links;
}

export function telegramPinnedMessages(messages: TelegramMessage[]): TelegramMessage[] {
	return messages.filter((message) =>
		metadataBoolean(message.metadata, ['pinned', 'is_pinned', 'tdlib_raw.is_pinned'])
	);
}

function matchesTelegramChatFilter(
	chat: TelegramChat,
	messages: TelegramMessage[],
	filter: TelegramChatFilter
): boolean {
	switch (filter) {
		case 'unread':
			return telegramChatUnreadCount(chat) > 0;
		case 'mentions':
			return telegramChatMentionCount(chat, messages) > 0;
		case 'pinned':
			return telegramChatIsPinned(chat);
		case 'projects':
			return telegramChatIsProject(chat);
		case 'bots':
			return telegramChatIsBot(chat);
		case 'archived':
			return telegramChatIsArchived(chat);
		case 'all':
		default:
			return true;
	}
}

function telegramFolderLabels(chat: TelegramChat): string[] {
	const labels = new Set<string>();
	for (const key of ['folder', 'folder_name', 'telegram_folder', 'chat_folder', 'chat_folder_title', 'tdlib_folder']) {
		const value = valueString(chat.metadata[key]);
		if (value) labels.add(value);
	}

	const tdlibRaw = metadataRecord(chat.metadata, ['tdlib_raw']);
	for (const position of valueArray(tdlibRaw?.positions)) {
		const list = valueRecord(valueRecord(position)?.list);
		const listType = valueString(list?.['@type']);
		if (listType === 'chatListArchive') labels.add('Archived');
		if (listType === 'chatListMain') labels.add('Main');
		if (listType === 'chatListFolder') {
			const folderId = valueNumber(list?.chat_folder_id);
			labels.add(folderId == null ? 'Folder' : `Folder ${folderId}`);
		}
	}

	return Array.from(labels);
}

function explicitAttachmentHints(message: TelegramMessage): TelegramAttachmentHint[] {
	const attachments = metadataArray(message.metadata, ['attachments', 'files', 'media']);
	return attachments
		.map((value, index) => {
			const attachment = valueRecord(value);
			if (!attachment) return null;
			const fileName = valueString(attachment.filename) || valueString(attachment.file_name);
			if (!fileName) return null;
			const sizeBytes = valueNumber(attachment.size_bytes) ?? valueNumber(attachment.size);
			const tdlibFileId =
				valueNumber(attachment.tdlib_file_id) ??
				valueNumber(attachment.file_id) ??
				valueNumber(attachment.id);
			const providerAttachmentId =
				valueString(attachment.attachment_id) ||
				(tdlibFileId !== null ? `tdlib-file:${tdlibFileId}` : `${message.message_id}:attachment:${index}`);
			return {
				id: providerAttachmentId,
				kind: telegramAttachmentKind(valueString(attachment.kind) || valueString(attachment.content_type)),
				fileName,
				mimeType: valueString(attachment.content_type) || valueString(attachment.mime_type),
				sizeBytes,
				tdlibFileId,
				providerAttachmentId,
				downloadState: telegramDownloadState(attachment),
				localPath: valueString(attachment.storage_path) || valueString(attachment.local_path),
				messageId: message.message_id
			} satisfies TelegramAttachmentHint;
		})
		.filter((value): value is TelegramAttachmentHint => value !== null);
}

function attachmentFromTdlibFile(
	message: TelegramMessage,
	id: string,
	kind: TelegramAttachmentHint['kind'],
	params: {
		fileName: string;
		mimeType: string | null;
		file: Record<string, unknown> | null;
	}
): TelegramAttachmentHint {
	const file = params.file;
	const local = file ? valueRecord(file.local) : null;
	const remote = file ? valueRecord(file.remote) : null;
	const localPath = valueString(local?.path);
	const isDownloading = valueBoolean(local?.is_downloading_active);
	const isDownloaded = valueBoolean(local?.is_downloading_completed) || Boolean(localPath);
	const sizeBytes = valueNumber(file?.size) ?? valueNumber(file?.expected_size);
	const tdlibFileId = valueNumber(file?.id);
	const providerAttachmentId =
		tdlibFileId !== null ? `tdlib-file:${tdlibFileId}` : valueString(remote?.unique_id) || id;
	return {
		id: providerAttachmentId,
		kind,
		fileName: params.fileName,
		mimeType: params.mimeType,
		sizeBytes,
		tdlibFileId,
		providerAttachmentId,
		downloadState: isDownloaded ? 'downloaded' : isDownloading ? 'downloading' : 'remote',
		localPath,
		messageId: message.message_id
	};
}

function telegramAttachmentKind(value: string | null): TelegramAttachmentHint['kind'] {
	const normalized = value?.toLowerCase() ?? '';
	if (normalized.includes('image') || normalized.includes('photo')) return 'photo';
	if (normalized.includes('video')) return 'video';
	if (normalized.includes('audio')) return 'audio';
	if (normalized.includes('voice')) return 'voice';
	if (normalized.includes('document')) return 'document';
	return 'file';
}

function telegramDownloadState(
	attachment: Record<string, unknown>
): TelegramAttachmentHint['downloadState'] {
	const state = valueString(attachment.download_state) || valueString(attachment.status);
	if (state === 'downloaded' || state === 'complete' || state === 'completed') return 'downloaded';
	if (state === 'downloading' || state === 'in_progress') return 'downloading';
	if (state === 'remote' || state === 'pending') return 'remote';
	if (valueString(attachment.storage_path) || valueString(attachment.local_path)) return 'downloaded';
	return 'unknown';
}

function metadataNumber(metadata: Record<string, unknown>, paths: string[]): number {
	for (const path of paths) {
		const value = metadataPath(metadata, path);
		const parsed = valueNumber(value);
		if (parsed !== null) return parsed;
	}
	return 0;
}

function metadataString(metadata: Record<string, unknown>, paths: string[]): string {
	for (const path of paths) {
		const value = valueString(metadataPath(metadata, path));
		if (value) return value;
	}
	return '';
}

function metadataBoolean(metadata: Record<string, unknown>, paths: string[]): boolean {
	for (const path of paths) {
		const value = valueBoolean(metadataPath(metadata, path));
		if (value !== null) return value;
	}
	return false;
}

function metadataArray(metadata: Record<string, unknown>, paths: string[]): unknown[] {
	for (const path of paths) {
		const value = valueArray(metadataPath(metadata, path));
		if (value.length) return value;
	}
	return [];
}

function metadataRecord(
	metadata: Record<string, unknown>,
	paths: string[]
): Record<string, unknown> | null {
	for (const path of paths) {
		const value = valueRecord(metadataPath(metadata, path));
		if (value) return value;
	}
	return null;
}

function metadataPath(metadata: Record<string, unknown>, path: string): unknown {
	return path.split('.').reduce<unknown>((current, segment) => {
		if (Array.isArray(current)) {
			const index = Number.parseInt(segment, 10);
			return Number.isFinite(index) ? current[index] : undefined;
		}
		const record = valueRecord(current);
		if (!record) return undefined;
		return record[segment];
	}, metadata);
}

function valueRecord(value: unknown): Record<string, unknown> | null {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null;
}

function valueArray(value: unknown): unknown[] {
	return Array.isArray(value) ? value : [];
}

function valueString(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function valueNumber(value: unknown): number | null {
	if (typeof value === 'number' && Number.isFinite(value)) return value;
	if (typeof value === 'string' && value.trim()) {
		const parsed = Number.parseInt(value.trim(), 10);
		return Number.isFinite(parsed) ? parsed : null;
	}
	return null;
}

function valueBoolean(value: unknown): boolean | null {
	if (typeof value === 'boolean') return value;
	if (typeof value === 'string') {
		if (value === 'true') return true;
		if (value === 'false') return false;
	}
	return null;
}

function telegramChatKindValue(value: string): 'private' | 'group' | 'channel' | 'bot' {
	if (value === 'group' || value === 'channel' || value === 'bot') {
		return value;
	}
	return 'private';
}

function providerKindLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

function parseJsonObject(value: string, field: string): Record<string, unknown> {
	const trimmed = value.trim();
	if (!trimmed) {
		return {};
	}

	const parsed = JSON.parse(trimmed) as unknown;
	if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
		throw new Error(`${field} must be a JSON object`);
	}
	return parsed as Record<string, unknown>;
}

function parseStringMap(value: string, field: string): Record<string, string> {
	const parsed = parseJsonObject(value, field);
	return Object.fromEntries(
		Object.entries(parsed).map(([key, rawValue]) => {
			if (typeof rawValue !== 'string') {
				throw new Error(`${field}.${key} must be a string`);
			}
			return [key, rawValue];
		})
	);
}
