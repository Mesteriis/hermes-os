export type TelegramProviderKind = 'telegram_user' | 'telegram_bot';

export type TelegramAccountLifecycleState = 'active' | 'logged_out' | 'removed' | string;

export type TelegramAccountSetupRequest = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	display_name: string;
	external_account_id: string;
	tdlib_data_path?: string;
	transcription_enabled: boolean;
};

export type TelegramLiveAccountSetupRequest = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	display_name: string;
	external_account_id: string;
	api_id?: number;
	api_hash?: string;
	bot_token?: string;
	session_encryption_key?: string;
	tdlib_data_path?: string;
	qr_authorized?: boolean;
	transcription_enabled: boolean;
};

export type TelegramQrLoginStartRequest = {
	account_id: string;
	display_name: string;
	external_account_id: string;
	api_id?: number;
	api_hash?: string;
	session_encryption_key?: string;
	tdlib_data_path?: string;
	transcription_enabled: boolean;
};

export type TelegramQrLoginPasswordRequest = {
	password: string;
};

export type TelegramQrLoginCancelResponse = {
	setup_id: string;
	cancelled: boolean;
};

export type TelegramQrLoginStatus =
	| 'waiting_qr_scan'
	| 'waiting_password'
	| 'ready'
	| 'expired'
	| 'failed'
	| 'runtime_unavailable';

export type TelegramQrLoginStatusResponse = {
	setup_id: string;
	account_id: string;
	status: TelegramQrLoginStatus;
	qr_link: string | null;
	qr_svg: string | null;
	telegram_user_id: string | null;
	telegram_username: string | null;
	suggested_account_id: string | null;
	suggested_display_name: string | null;
	suggested_external_account_id: string | null;
	expires_at: string | null;
	poll_after_ms: number;
	message: string | null;
};

export type TelegramCredentialBinding = {
	secret_purpose: string;
	secret_ref: string;
	secret_kind: string;
	store_kind: string;
};

export type TelegramAccountSetupResponse = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	runtime: string;
	transcription_enabled: boolean;
	credential_bindings: TelegramCredentialBinding[];
};

export type TelegramAccount = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	display_name: string;
	external_account_id: string;
	runtime: string;
	lifecycle_state: TelegramAccountLifecycleState;
	transcription_enabled: boolean;
	tdlib_data_path: string | null;
	created_at: string;
	updated_at: string;
};

export type TelegramAccountListResponse = {
	items: TelegramAccount[];
};

export type TelegramAccountLifecycleResponse = {
	account: TelegramAccount;
	stopped_runtime_actor: boolean;
};

export type TelegramCapabilityStatus = {
	capability: string;
	status: 'available' | 'blocked' | string;
	closure_gate: boolean;
	reason: string;
};

export type TelegramCapabilitiesResponse = {
	version: string;
	runtime_mode: string;
	telegram_app_credentials_configured: boolean;
	tdjson_runtime_available: boolean;
	qr_login_ready: boolean;
	capabilities: TelegramCapabilityStatus[];
	unsupported_features: string[];
};

export type TelegramRuntimeStatus = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	runtime_kind: string;
	status: 'stopped' | 'running' | 'blocked' | 'degraded' | 'error' | string;
	fixture_runtime: boolean;
	tdjson_runtime_available: boolean;
	telegram_app_credentials_configured: boolean;
	live_send_available: boolean;
	last_error: string | null;
	updated_at: string;
};

export type TelegramRuntimeStartRequest = {
	account_id: string;
};

export type TelegramChatSyncRequest = {
	account_id: string;
	limit?: number;
};

export type TelegramChatSyncResponse = {
	account_id: string;
	runtime_kind: string;
	status: string;
	synced_count: number;
	items: TelegramChat[];
};

export type TelegramHistorySyncRequest = {
	account_id: string;
	provider_chat_id: string;
	from_message_id?: number;
	mode?: 'latest' | 'older' | 'full';
	limit?: number;
};

export type TelegramHistorySyncResponse = {
	account_id: string;
	provider_chat_id: string;
	runtime_kind: string;
	status: string;
	synced_count: number;
	has_more: boolean;
	next_from_message_id: number | null;
	items: TelegramMessage[];
};

export type TelegramMediaDownloadRequest = {
	account_id: string;
	provider_chat_id: string;
	provider_message_id: string;
	tdlib_file_id: number;
	provider_attachment_id?: string;
	filename?: string;
	content_type?: string;
	priority?: number;
};

export type TelegramMediaDownloadResponse = {
	account_id: string;
	provider_chat_id: string;
	provider_message_id: string;
	runtime_kind: string;
	status: string;
	tdlib_file_id: number;
	local_path: string | null;
	size_bytes: number | null;
	expected_size_bytes: number | null;
	downloaded_size_bytes: number | null;
	is_downloading_active: boolean;
	is_downloading_completed: boolean;
	attachment_id: string | null;
	blob_id: string | null;
	scan_status: string | null;
};

export type TelegramChat = {
	telegram_chat_id: string;
	account_id: string;
	provider_chat_id: string;
	chat_kind: 'private' | 'group' | 'channel' | 'bot';
	title: string;
	username: string | null;
	sync_state: 'fixture' | 'syncing' | 'synced' | 'degraded' | 'error';
	last_message_at: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type TelegramChatListResponse = {
	items: TelegramChat[];
};

export type TelegramMessage = {
	message_id: string;
	raw_record_id: string;
	account_id: string;
	provider_message_id: string;
	provider_chat_id: string | null;
	chat_title: string;
	sender: string;
	sender_display_name: string | null;
	text: string;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: TelegramProviderKind;
	delivery_state: string;
	metadata: Record<string, unknown>;
};

export type TelegramMessageListResponse = {
	items: TelegramMessage[];
};

export type TelegramFixtureMessageRequest = {
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
	delivery_state: 'received' | 'sent' | 'send_dry_run' | 'send_blocked';
};

export type TelegramMessageIngestResponse = {
	raw_record_id: string;
	message_id: string;
};

export type TelegramManualSendRequest = {
	command_id: string;
	account_id: string;
	provider_chat_id: string;
	text: string;
};

export type TelegramManualSendResponse = {
	raw_record_id: string;
	message_id: string;
	account_id: string;
	provider_chat_id: string;
	delivery_state: string;
	status: string;
	runtime_kind: string;
	rendered_preview_hash: string;
};

export type AutomationTemplate = {
	template_id: string;
	name: string;
	body_template: string;
	required_variables: string[];
	created_at: string;
	updated_at: string;
};

export type AutomationTemplateListResponse = {
	items: AutomationTemplate[];
};

export type AutomationPolicy = {
	policy_id: string;
	template_id: string;
	name: string;
	enabled: boolean;
	account_id: string;
	allowed_chat_ids: string[];
	trigger_kind: string;
	max_sends_per_hour: number;
	quiet_hours: Record<string, unknown>;
	expires_at: string | null;
	conditions: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type AutomationPolicyListResponse = {
	items: AutomationPolicy[];
};

export type AutomationTemplateRequest = {
	template_id: string;
	name: string;
	body_template: string;
	required_variables: string[];
};

export type AutomationPolicyRequest = {
	policy_id: string;
	template_id: string;
	name: string;
	enabled: boolean;
	account_id: string;
	allowed_chat_ids: string[];
	trigger_kind: string;
	max_sends_per_hour: number;
	quiet_hours: Record<string, unknown>;
	expires_at?: string | null;
	conditions: Record<string, unknown>;
};

export type TelegramSendDryRunRequest = {
	command_id: string;
	policy_id: string;
	provider_chat_id: string;
	variables: Record<string, string>;
	source_context: Record<string, unknown>;
};

export type TelegramSendDryRunResponse = {
	outbound_message_id: string;
	policy_id: string;
	template_id: string;
	account_id: string;
	provider_chat_id: string;
	rendered_text: string;
	rendered_preview_hash: string;
	status: string;
	event_id: string;
};

export type TelegramCall = {
	call_id: string;
	account_id: string;
	provider_call_id: string;
	provider_chat_id: string;
	direction: 'incoming' | 'outgoing';
	call_state: 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed';
	started_at: string | null;
	ended_at: string | null;
	transcription_policy_id: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type TelegramCallListResponse = {
	items: TelegramCall[];
};

export type TelegramCallRequest = {
	call_id: string;
	account_id: string;
	provider_call_id: string;
	provider_chat_id: string;
	direction: 'incoming' | 'outgoing';
	call_state: 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed';
	started_at?: string | null;
	ended_at?: string | null;
	transcription_policy_id?: string | null;
	metadata: Record<string, unknown>;
};

export type CallTranscript = {
	transcript_id: string;
	call_id: string;
	account_id: string;
	provider_chat_id: string;
	transcript_status: 'queued' | 'running' | 'succeeded' | 'failed';
	stt_provider: string;
	source_audio_ref: string | null;
	language_code: string | null;
	transcript_text: string;
	segments: unknown[];
	provenance: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CallTranscriptResponse = {
	transcript: CallTranscript | null;
};

export type CallTranscriptFixtureRequest = {
	transcript_id: string;
	account_id: string;
	provider_chat_id: string;
	source_audio_ref: string;
	language_code?: string;
	always_on_policy: boolean;
};
