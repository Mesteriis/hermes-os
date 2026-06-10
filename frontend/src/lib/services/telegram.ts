import {
	fetchTelegramCapabilities,
	fetchTelegramChats,
	fetchTelegramMessages,
	fetchAutomationTemplates,
	fetchAutomationPolicies,
	fetchTelegramCalls,
	fetchCallTranscript,
	fetchTelegramQrLoginStatus,
	ingestTelegramFixtureMessage,
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
	type AutomationTemplate,
	type AutomationPolicy,
	type TelegramCall,
	type CallTranscript,
	type TelegramProviderKind,
	type TelegramQrLoginStatusResponse,
	type TelegramSendDryRunResponse
} from '$lib/api';
import { formatDateTime } from './formatting';

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
			fetchTelegramChats(),
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

		let transcript: CallTranscript | null = null;
		if (nextCallId) {
			try {
				const response = await fetchCallTranscript(nextCallId);
				transcript = response.transcript;
			} catch { /* transcript optional */ }
		}

		return {
			capabilities: capabilityResponse,
			chats,
			messages: messageResponse.items,
			templates: templateResponse.items,
			policies: policyResponse.items,
			calls,
			transcript,
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
			selectedChatId: selectedTelegramChatId,
			selectedCallId: selectedTelegramCallId,
			error: error instanceof Error ? error.message : 'Unknown Telegram workspace error'
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

	try {
		const result = isFixtureSetup
			? await setupTelegramFixtureAccount({
					account_id: accountForm.account_id,
					provider_kind: providerKind,
					display_name: accountForm.display_name,
					external_account_id: accountForm.external_account_id,
					tdlib_data_path:
						authMethod === 'qr' ? undefined : accountForm.tdlib_data_path || undefined,
					transcription_enabled:
						authMethod === 'qr' ? false : accountForm.transcription_enabled
				})
			: await setupTelegramAccount({
					account_id: accountForm.account_id,
					provider_kind: providerKind,
					display_name: accountForm.display_name,
					external_account_id: accountForm.external_account_id,
					api_id:
						providerKind === 'telegram_user' && accountForm.api_id.trim()
							? Number(accountForm.api_id.trim())
							: undefined,
					api_hash:
						providerKind === 'telegram_user'
							? accountForm.api_hash.trim() || undefined
							: undefined,
					bot_token:
						providerKind === 'telegram_bot'
							? accountForm.bot_token || undefined
							: undefined,
					session_encryption_key:
						providerKind === 'telegram_user'
							? accountForm.session_encryption_key || undefined
							: undefined,
					tdlib_data_path:
						authMethod === 'qr' ? undefined : accountForm.tdlib_data_path || undefined,
					transcription_enabled:
						authMethod === 'qr' ? false : accountForm.transcription_enabled
				});
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

	const apiIdValue = accountForm.api_id.trim();
	const apiHashValue = accountForm.api_hash.trim();
	const appCredentialsConfigured = capabilities?.telegram_app_credentials_configured ?? false;
	if (!appCredentialsConfigured && (!apiIdValue || !apiHashValue)) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram API ID and API hash are required for QR login in this dev session'
		};
	}
	const parsedApiId = Number(apiIdValue);
	if (apiIdValue && (!Number.isInteger(parsedApiId) || parsedApiId <= 0)) {
		return {
			qrLogin: null,
			message: '',
			error: 'Telegram API ID must be greater than zero'
		};
	}
	const apiId = apiIdValue ? parsedApiId : undefined;

	try {
		const result = await startTelegramQrLogin({
			account_id: accountForm.account_id,
			display_name: accountForm.display_name,
			external_account_id: externalAccountId,
			api_id: apiId,
			api_hash: apiHashValue || undefined,
			session_encryption_key: accountForm.session_encryption_key || undefined,
			tdlib_data_path: undefined,
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
