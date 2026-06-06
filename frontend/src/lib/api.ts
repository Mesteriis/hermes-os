import type { LayoutSettings } from '$lib/layout';

export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		contacts: boolean;
		search: boolean;
		documents: boolean;
		account_setup: boolean;
	};
};

export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json';

export type ApplicationSetting = {
	setting_key: string;
	category: string;
	value_kind: SettingValueKind;
	value: boolean | number | string | Record<string, unknown> | unknown[];
	label: string;
	description: string;
	metadata: Record<string, unknown>;
	is_editable: boolean;
	updated_by_actor_id: string | null;
	created_at: string;
	updated_at: string;
};

export type ApplicationSettingsResponse = {
	items: ApplicationSetting[];
};

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout';

export type ProviderAccount = {
	account_id: string;
	provider_kind:
		| 'gmail'
		| 'icloud'
		| 'imap'
		| 'telegram_user'
		| 'telegram_bot'
		| 'whatsapp_web'
		| string;
	display_name: string;
	external_account_id: string;
	config: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type ProviderAccountListResponse = {
	items: ProviderAccount[];
};

export type CommunicationMessageSummary = {
	message_id: string;
	raw_record_id: string;
	account_id: string;
	provider_record_id: string;
	subject: string;
	sender: string;
	recipients: string[];
	body_text_preview: string;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: string;
	conversation_id: string | null;
	sender_display_name: string | null;
	delivery_state: string;
	message_metadata: Record<string, unknown>;
	attachment_count: number;
};

export type CommunicationMessageDetailItem = {
	message_id: string;
	raw_record_id: string;
	account_id: string;
	provider_record_id: string;
	subject: string;
	sender: string;
	recipients: string[];
	body_text: string;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: string;
	conversation_id: string | null;
	sender_display_name: string | null;
	delivery_state: string;
	message_metadata: Record<string, unknown>;
};

export type CommunicationAttachment = {
	attachment_id: string;
	message_id: string;
	raw_record_id: string;
	blob_id: string;
	provider_attachment_id: string;
	filename: string | null;
	content_type: string;
	size_bytes: number;
	sha256: string;
	disposition: 'attachment' | 'inline' | 'unknown';
	scan_status: 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed';
	scan_engine: string | null;
	scan_checked_at: string | null;
	scan_summary: string | null;
	scan_metadata: Record<string, unknown>;
	storage_kind: string;
	storage_path: string;
	created_at: string;
	updated_at: string;
};

export type CommunicationMessagesResponse = {
	items: CommunicationMessageSummary[];
};

export type CommunicationMessageDetail = {
	message: CommunicationMessageDetailItem;
	attachments: CommunicationAttachment[];
};

export type GraphNodeKind = 'person' | 'email_address' | 'message' | 'document' | 'project';

export type GraphRelationshipType =
	| 'person_has_email_address'
	| 'person_sent_message'
	| 'person_received_message'
	| 'email_address_sent_message'
	| 'email_address_received_message'
	| 'project_has_message'
	| 'project_has_document'
	| 'project_involves_person'
	| 'project_involves_email_address';

export type GraphReviewState =
	| 'system_accepted'
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type GraphEvidenceSourceKind = 'contact' | 'message' | 'document' | 'raw_record';

export type GraphNode = {
	node_id: string;
	node_kind: GraphNodeKind;
	stable_key: string;
	label: string;
	properties: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type GraphEdge = {
	edge_id: string;
	source_node_id: string;
	target_node_id: string;
	relationship_type: GraphRelationshipType;
	confidence: number;
	review_state: GraphReviewState;
	properties: Record<string, unknown>;
	valid_from: string | null;
	valid_to: string | null;
	created_at: string;
	updated_at: string;
};

export type GraphCount = {
	key: string;
	count: number;
};

export type GraphSummary = {
	node_counts: GraphCount[];
	edge_counts: GraphCount[];
	evidence_count: number;
	latest_projection_at: string | null;
	is_empty: boolean;
};

export type GraphEvidenceSummary = {
	edge_id: string;
	source_kind: GraphEvidenceSourceKind;
	source_id: string;
	excerpt: string | null;
	metadata: Record<string, unknown>;
};

export type GraphNeighborhood = {
	selected_node: GraphNode;
	nodes: GraphNode[];
	edges: GraphEdge[];
	evidence: GraphEvidenceSummary[];
	edge_limit: number;
	truncated: boolean;
	evidence_limit: number;
	evidence_truncated: boolean;
};

export type ProjectRecord = {
	project_id: string;
	name: string;
	kind: string;
	status: 'planning' | 'active' | 'on_hold' | 'completed' | 'archived';
	description: string;
	owner_display_name: string;
	progress_percent: number;
	start_date: string | null;
	target_date: string | null;
	created_at: string;
	updated_at: string;
};

export type ProjectStats = {
	message_count: number;
	document_count: number;
	people_count: number;
	graph_connection_count: number;
	latest_activity_at: string | null;
};

export type ProjectSummary = {
	project: ProjectRecord;
	stats: ProjectStats;
	graph_node_id: string;
};

export type ProjectTimelineItem = {
	item_kind: 'message' | 'document' | string;
	item_id: string;
	title: string;
	subtitle: string;
	occurred_at: string;
};

export type ProjectPersonSummary = {
	display_name: string;
	email_address: string;
	interaction_count: number;
	last_interaction_at: string | null;
};

export type ProjectMessageSummary = {
	message_id: string;
	subject: string;
	sender: string;
	occurred_at: string;
};

export type ProjectDocumentSummary = {
	document_id: string;
	document_kind: string;
	title: string;
	imported_at: string;
};

export type ContactIdentityReviewState =
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type ContactIdentityCandidate = {
	identity_candidate_id: string;
	candidate_kind: 'merge_contacts' | 'attach_email_address' | 'split_contact';
	left_contact_id: string;
	right_contact_id: string | null;
	email_address: string | null;
	evidence_summary: string;
	confidence: number;
	review_state: ContactIdentityReviewState;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type ContactIdentityCandidateListResponse = {
	items: ContactIdentityCandidate[];
};

export type TaskCandidateReviewState =
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type TaskCandidate = {
	task_candidate_id: string;
	source_kind: 'message' | 'document';
	source_id: string;
	project_id: string | null;
	title: string;
	due_text: string | null;
	assignee_label: string | null;
	confidence: number;
	review_state: TaskCandidateReviewState;
	evidence_excerpt: string;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type ActiveTask = {
	task_id: string;
	task_candidate_id: string;
	title: string;
	source_kind: 'message' | 'document';
	source_id: string;
	project_id: string | null;
	status: 'active';
	created_at: string;
	updated_at: string;
};

export type TaskCandidateListResponse = {
	items: TaskCandidate[];
};

export type TaskListResponse = {
	items: ActiveTask[];
};

export type DocumentProcessingStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'skipped';

export type DocumentProcessingStep = 'extract_text' | 'ocr';

export type DocumentProcessingArtifactKind = 'extracted_text' | 'ocr_text';

export type DocumentProcessingJob = {
	job_id: string;
	document_id: string;
	step: DocumentProcessingStep;
	status: DocumentProcessingStatus;
	attempts: number;
	max_attempts: number;
	last_error_summary: string | null;
	queued_at: string;
	started_at: string | null;
	finished_at: string | null;
	created_at: string;
	updated_at: string;
};

export type DocumentProcessingArtifact = {
	artifact_id: string;
	document_id: string;
	job_id: string;
	artifact_kind: DocumentProcessingArtifactKind;
	content_sha256: string;
	text_content: string | null;
	storage_kind: string | null;
	storage_path: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
};

export type DocumentProcessingRecord = {
	document_id: string;
	jobs: DocumentProcessingJob[];
	artifacts: DocumentProcessingArtifact[];
};

export type DocumentProcessingJobsResponse = {
	items: DocumentProcessingJob[];
};

export type DocumentProcessingRetryRequest = {
	command_id: string;
};

export type DocumentProcessingRetryResponse = {
	job_id: string;
	status: DocumentProcessingStatus;
	event_id: string;
};

export type AiStatus = {
	runtime: string;
	status: string;
	version: string | null;
	chat_model: string;
	embedding_model: string;
	embedding_dimension: number;
	chat_model_available: boolean;
	embedding_model_available: boolean;
};

export type AiAgent = {
	agent_id: 'HESTIA' | 'HERMES' | 'MNEMOSYNE' | 'ATHENA' | string;
	display_name: string;
	role: string;
	default_model: string;
	status: string;
};

export type AiAgentListResponse = {
	items: AiAgent[];
};

export type AiCitation = {
	source_kind: string;
	source_id: string;
	title: string;
	excerpt: string;
	score: number;
	graph_node_id?: string;
};

export type AiRun = {
	run_id: string;
	agent_id: string;
	status: 'requested' | 'completed' | 'failed' | string;
	chat_model: string;
	embedding_model: string;
	prompt_template_version: string;
	model_config: Record<string, unknown>;
	query: string;
	answer: string | null;
	citations: AiCitation[] | unknown[];
	error_summary: string | null;
	actor_id: string;
	causation_id: string | null;
	correlation_id: string | null;
	requested_event_id: string | null;
	completed_event_id: string | null;
	failed_event_id: string | null;
	started_at: string;
	completed_at: string | null;
	duration_ms: number | null;
	created_at: string;
	updated_at: string;
};

export type AiRunListResponse = {
	items: AiRun[];
};

export type AiAnswerRequest = {
	command_id: string;
	query: string;
	agent_id?: string;
	correlation_id?: string;
};

export type AiAnswerResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	answer: string;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};

export type AiTaskCandidateRefreshRequest = {
	command_id: string;
	query: string;
	correlation_id?: string;
};

export type AiTaskCandidateRefreshResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	created_count: number;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};

export type AiMeetingPrepRequest = {
	command_id: string;
	topic: string;
	project_id?: string;
	contact_id?: string;
	correlation_id?: string;
};

export type AiMeetingPrepResponse = {
	run_id: string;
	agent_id: string;
	status: string;
	briefing: string;
	citations: AiCitation[];
	model: string;
	embedding_model: string;
	created_at: string;
	duration_ms: number;
};

export type TelegramProviderKind = 'telegram_user' | 'telegram_bot';

export type TelegramAccountSetupRequest = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	display_name: string;
	external_account_id: string;
	tdlib_data_path?: string;
	transcription_enabled: boolean;
};

export type TelegramAccountSetupResponse = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	runtime: string;
	transcription_enabled: boolean;
};

export type V4CapabilityStatus = {
	capability: string;
	status: 'available' | 'blocked' | string;
	closure_gate: boolean;
	reason: string;
};

export type V4CapabilitiesResponse = {
	version: string;
	runtime_mode: string;
	capabilities: V4CapabilityStatus[];
	unsupported_features: string[];
};

export type WhatsappWebProviderKind = 'whatsapp_web';

export type V5CapabilityStatus = {
	capability: string;
	status: 'available' | 'blocked' | string;
	closure_gate: boolean;
	reason: string;
};

export type V5CapabilitiesResponse = {
	version: string;
	runtime_mode: string;
	capabilities: V5CapabilityStatus[];
	unsupported_features: string[];
};

export type WhatsappWebSession = {
	session_id: string;
	account_id: string;
	device_name: string;
	companion_runtime: 'fixture' | 'manual_webview' | 'blocked';
	link_state: 'fixture' | 'qr_pending' | 'linked' | 'degraded' | 'revoked' | 'blocked';
	local_state_path: string;
	last_sync_at: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type WhatsappWebSessionListResponse = {
	items: WhatsappWebSession[];
};

export type WhatsappWebAccountSetupRequest = {
	account_id: string;
	provider_kind: WhatsappWebProviderKind;
	display_name: string;
	external_account_id: string;
	device_name: string;
	local_state_path: string;
};

export type WhatsappWebAccountSetupResponse = {
	account_id: string;
	provider_kind: WhatsappWebProviderKind;
	runtime: string;
	session: WhatsappWebSession;
};

export type WhatsappWebMessage = {
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
	channel_kind: WhatsappWebProviderKind;
	delivery_state: string;
	metadata: Record<string, unknown>;
};

export type WhatsappWebMessageListResponse = {
	items: WhatsappWebMessage[];
};

export type WhatsappWebFixtureMessageRequest = {
	account_id: string;
	provider_chat_id: string;
	provider_message_id: string;
	chat_title: string;
	sender_id: string;
	sender_display_name: string;
	text: string;
	import_batch_id: string;
	occurred_at: string;
	delivery_state: 'received' | 'sent' | 'send_dry_run' | 'send_blocked';
};

export type WhatsappWebMessageIngestResponse = {
	raw_record_id: string;
	message_id: string;
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

export async function fetchIdentityCandidates(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
) : Promise<ContactIdentityCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/identity-candidates?${params.toString()}`,
		'Identity candidate request failed'
	);
}

export type ProjectDetail = {
	project: ProjectRecord;
	stats: ProjectStats;
	graph_node_id: string;
	timeline: ProjectTimelineItem[];
	key_people: ProjectPersonSummary[];
	recent_messages: ProjectMessageSummary[];
	documents: ProjectDocumentSummary[];
};

export type ProjectListResponse = {
	items: ProjectSummary[];
};

export type GmailOAuthStartRequest = {
	account_id: string;
	display_name: string;
	external_account_id: string;
	client_id: string;
	client_secret?: string;
	redirect_uri: string;
};

export type GmailOAuthStartResponse = {
	setup_id: string;
	authorization_url: string;
	state: string;
	redirect_uri: string;
};

export type GmailOAuthCompleteRequest = {
	setup_id: string;
	state: string;
	authorization_code: string;
};

export type EmailAccountSetupResponse = {
	account_id: string;
	secret_ref: string;
	secret_kind: 'oauth_token' | 'app_password' | 'password';
	store_kind: 'encrypted_vault' | 'database_encrypted_vault';
};

export type ImapAccountSetupRequest = {
	account_id: string;
	provider_kind: 'icloud' | 'imap';
	display_name: string;
	external_account_id: string;
	host: string;
	port: number;
	tls: boolean;
	mailbox: string;
	username: string;
	password: string;
	secret_kind: 'app_password' | 'password';
};

export async function fetchV1Status(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V1Status> {
	return getJson(baseUrl, token, actorId, '/api/v1/status', 'V1 status request failed');
}

export async function fetchApplicationSettings(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<ApplicationSettingsResponse> {
	return getJson(baseUrl, token, actorId, '/api/v2/settings', 'Settings request failed');
}

export async function saveApplicationSetting(
	baseUrl: string,
	token: string,
	actorId: string,
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	return putJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/settings/${encodeURIComponent(settingKey)}`,
		{ value },
		'Setting update failed'
	);
}

export function findFrontendLayoutSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_LAYOUT_SETTING_KEY) ?? null;
}

export async function saveFrontendLayoutSetting(
	baseUrl: string,
	token: string,
	actorId: string,
	value: LayoutSettings
): Promise<ApplicationSetting> {
	return saveApplicationSetting(baseUrl, token, actorId, FRONTEND_LAYOUT_SETTING_KEY, value);
}

export async function fetchProviderAccounts(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<ProviderAccountListResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		'/api/v2/settings/accounts',
		'Provider accounts request failed'
	);
}

export async function fetchCommunicationMessages(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
): Promise<CommunicationMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	);
}

export async function fetchCommunicationMessage(
	baseUrl: string,
	token: string,
	actorId: string,
	messageId: string
): Promise<CommunicationMessageDetail> {
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Communication message detail request failed'
	);
}

export async function fetchGraphSummary(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<GraphSummary> {
	return getJson(baseUrl, token, actorId, '/api/v2/graph/summary', 'Graph summary request failed');
}

export async function fetchGraphNodes(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 20
): Promise<GraphNode[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/graph/nodes?${params.toString()}`,
		'Graph node picker request failed'
	);
}

export async function searchGraphNodes(
	baseUrl: string,
	token: string,
	actorId: string,
	query: string,
	limit = 20
): Promise<GraphNode[]> {
	const normalizedQuery = query.trim();
	if (!normalizedQuery) {
		return [];
	}

	const params = new URLSearchParams({
		q: normalizedQuery,
		limit: String(Math.trunc(limit))
	});

	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/graph/search?${params.toString()}`,
		'Graph search request failed'
	);
}

export async function fetchGraphNeighborhood(
	baseUrl: string,
	token: string,
	actorId: string,
	nodeId: string,
	depth = 1
): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({
		node_id: nodeId,
		depth: String(depth)
	});

	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/graph/neighborhood?${params.toString()}`,
		'Graph neighborhood request failed'
	);
}

export async function fetchProjects(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 25
): Promise<ProjectListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/projects?${params.toString()}`,
		'Projects request failed'
	);
}

export async function fetchProjectDetail(
	baseUrl: string,
	token: string,
	actorId: string,
	projectId: string
): Promise<ProjectDetail> {
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/projects/${encodeURIComponent(projectId)}`,
		'Project detail request failed'
	);
}

export async function fetchTaskCandidates(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
): Promise<TaskCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/task-candidates?${params.toString()}`,
		'Task candidates request failed'
	);
}

export async function fetchTasks(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
): Promise<TaskListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/tasks?${params.toString()}`,
		'Tasks request failed'
	);
}

export async function fetchDocumentProcessingJobs(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 50
): Promise<DocumentProcessingJobsResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/document-processing/jobs?${params.toString()}`,
		'Document processing jobs request failed'
	);
}

export async function fetchDocumentProcessing(
	baseUrl: string,
	token: string,
	actorId: string,
	documentId: string
): Promise<DocumentProcessingRecord> {
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/documents/${encodeURIComponent(documentId)}/processing`,
		'Document processing request failed'
	);
}

export async function retryDocumentProcessingJob(
	baseUrl: string,
	token: string,
	actorId: string,
	jobId: string,
	request: DocumentProcessingRetryRequest
): Promise<DocumentProcessingRetryResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/document-processing/jobs/${encodeURIComponent(jobId)}/retry`,
		request,
		'Document processing retry request failed'
	);
}

export async function reviewTaskCandidate(
	baseUrl: string,
	token: string,
	actorId: string,
	taskCandidateId: string,
	reviewState: TaskCandidateReviewState
) {
	return putJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/task-candidates/${encodeURIComponent(taskCandidateId)}/review`,
		{
			command_id: `task-candidate-review-${crypto.randomUUID()}`,
			review_state: reviewState
		}
	);
}

export async function reviewIdentityCandidate(
	baseUrl: string,
	token: string,
	actorId: string,
	identityCandidateId: string,
	reviewState: ContactIdentityReviewState,
	commandId = `contact-identity-review-${crypto.randomUUID()}`
) {
	return putJson(
		baseUrl,
		token,
		actorId,
		`/api/v2/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
		{
			command_id: commandId,
			review_state: reviewState
		}
	);
}

export async function startGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/start', request);
}

export async function completeGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthCompleteRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/complete', request);
}

export async function setupImapAccount(
	baseUrl: string,
	token: string,
	actorId: string,
	request: ImapAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/imap', request);
}

export async function setupTelegramFixtureAccount(
	baseUrl: string,
	token: string,
	actorId: string,
	request: TelegramAccountSetupRequest
): Promise<TelegramAccountSetupResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/telegram/accounts/fixture',
		request,
		'Telegram account setup request failed'
	);
}

export async function fetchV4Capabilities(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V4CapabilitiesResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/capabilities',
		'V4 capabilities request failed'
	);
}

export async function fetchV5Capabilities(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V5CapabilitiesResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		'/api/v5/capabilities',
		'V5 capabilities request failed'
	);
}

export async function setupWhatsappWebFixtureAccount(
	baseUrl: string,
	token: string,
	actorId: string,
	request: WhatsappWebAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v5/whatsapp/accounts/fixture',
		request,
		'WhatsApp Web account setup request failed'
	);
}

export async function fetchWhatsappWebSessions(
	baseUrl: string,
	token: string,
	actorId: string,
	accountId?: string,
	limit = 50
): Promise<WhatsappWebSessionListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v5/whatsapp/sessions?${params.toString()}`,
		'WhatsApp Web sessions request failed'
	);
}

export async function fetchWhatsappWebMessages(
	baseUrl: string,
	token: string,
	actorId: string,
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
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v5/whatsapp/messages?${params.toString()}`,
		'WhatsApp Web messages request failed'
	);
}

export async function ingestWhatsappWebFixtureMessage(
	baseUrl: string,
	token: string,
	actorId: string,
	request: WhatsappWebFixtureMessageRequest
): Promise<WhatsappWebMessageIngestResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v5/whatsapp/messages',
		request,
		'WhatsApp Web fixture message request failed'
	);
}

export async function fetchTelegramChats(
	baseUrl: string,
	token: string,
	actorId: string,
	accountId?: string,
	limit = 50
): Promise<TelegramChatListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v4/telegram/chats?${params.toString()}`,
		'Telegram chats request failed'
	);
}

export async function fetchTelegramMessages(
	baseUrl: string,
	token: string,
	actorId: string,
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
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v4/telegram/messages?${params.toString()}`,
		'Telegram messages request failed'
	);
}

export async function ingestTelegramFixtureMessage(
	baseUrl: string,
	token: string,
	actorId: string,
	request: TelegramFixtureMessageRequest
): Promise<TelegramMessageIngestResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/telegram/messages',
		request,
		'Telegram fixture message request failed'
	);
}

export async function fetchAutomationTemplates(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<AutomationTemplateListResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/policies/templates',
		'Automation template request failed'
	);
}

export async function saveAutomationTemplate(
	baseUrl: string,
	token: string,
	actorId: string,
	request: AutomationTemplateRequest
): Promise<AutomationTemplate> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/policies/templates',
		request,
		'Automation template save failed'
	);
}

export async function fetchAutomationPolicies(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<AutomationPolicyListResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/policies',
		'Automation policy request failed'
	);
}

export async function saveAutomationPolicy(
	baseUrl: string,
	token: string,
	actorId: string,
	request: AutomationPolicyRequest
): Promise<AutomationPolicy> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/policies',
		request,
		'Automation policy save failed'
	);
}

export async function dryRunTelegramSend(
	baseUrl: string,
	token: string,
	actorId: string,
	request: TelegramSendDryRunRequest
): Promise<TelegramSendDryRunResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v4/policies/telegram-send/dry-run',
		request,
		'Telegram send dry-run failed'
	);
}

export async function fetchTelegramCalls(
	baseUrl: string,
	token: string,
	actorId: string,
	accountId?: string,
	limit = 50
): Promise<TelegramCallListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v4/calls?${params.toString()}`,
		'Telegram call request failed'
	);
}

export async function saveTelegramCall(
	baseUrl: string,
	token: string,
	actorId: string,
	request: TelegramCallRequest
): Promise<TelegramCall> {
	return postJson(baseUrl, token, actorId, '/api/v4/calls', request, 'Telegram call save failed');
}

export async function saveCallTranscriptFixture(
	baseUrl: string,
	token: string,
	actorId: string,
	callId: string,
	request: CallTranscriptFixtureRequest
): Promise<CallTranscript> {
	return postJson(
		baseUrl,
		token,
		actorId,
		`/api/v4/calls/${encodeURIComponent(callId)}/transcript`,
		request,
		'Call transcript save failed'
	);
}

export async function fetchCallTranscript(
	baseUrl: string,
	token: string,
	actorId: string,
	callId: string
): Promise<CallTranscriptResponse> {
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v4/calls/${encodeURIComponent(callId)}/transcript`,
		'Call transcript request failed'
	);
}

export async function fetchAiStatus(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<AiStatus> {
	return getJson(baseUrl, token, actorId, '/api/v3/ai/status', 'AI status request failed');
}

export async function fetchAiAgents(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<AiAgentListResponse> {
	return getJson(baseUrl, token, actorId, '/api/v3/agents', 'AI agents request failed');
}

export async function fetchAiRuns(
	baseUrl: string,
	token: string,
	actorId: string,
	limit = 25
): Promise<AiRunListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl,
		token,
		actorId,
		`/api/v3/ai/runs?${params.toString()}`,
		'AI run history request failed'
	);
}

export async function requestAiAnswer(
	baseUrl: string,
	token: string,
	actorId: string,
	request: AiAnswerRequest
): Promise<AiAnswerResponse> {
	return postJson(baseUrl, token, actorId, '/api/v3/ai/answers', request, 'AI answer request failed');
}

export async function refreshAiTaskCandidates(
	baseUrl: string,
	token: string,
	actorId: string,
	request: AiTaskCandidateRefreshRequest
): Promise<AiTaskCandidateRefreshResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v3/ai/task-candidates/refresh',
		request,
		'AI task candidate refresh request failed'
	);
}

export async function requestAiMeetingPrep(
	baseUrl: string,
	token: string,
	actorId: string,
	request: AiMeetingPrepRequest
): Promise<AiMeetingPrepResponse> {
	return postJson(
		baseUrl,
		token,
		actorId,
		'/api/v3/ai/meeting-prep',
		request,
		'AI meeting prep request failed'
	);
}

async function getJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	fallbackMessage: string
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		headers: {
			Authorization: `Bearer ${token}`,
			'X-Hermes-Actor-Id': actorId
		}
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}

async function postJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	body: unknown,
	fallbackMessage = 'Account setup request failed'
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json',
			'X-Hermes-Actor-Id': actorId
		},
		body: JSON.stringify(body)
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}

async function putJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	body: unknown,
	fallbackMessage = 'PUT request failed'
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'PUT',
		headers: {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json',
			'X-Hermes-Actor-Id': actorId
		},
		body: JSON.stringify(body)
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `${fallbackMessage}: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}
