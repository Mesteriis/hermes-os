import type { LayoutSettings } from '$lib/layout';

export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		persons: boolean;
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

export type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam';

export type CommunicationMessageSummaryV2 = {
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
	workflow_state: WorkflowState;
	importance_score: number | null;
	ai_category: string | null;
	ai_summary: string | null;
	ai_summary_generated_at: string | null;
	message_metadata: Record<string, unknown>;
	attachment_count: number;
};

export type MailMessagesResponse = {
	items: CommunicationMessageSummaryV2[];
};

export type MailMessageDetailItemV2 = {
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
	workflow_state: WorkflowState;
	importance_score: number | null;
	ai_category: string | null;
	ai_summary: string | null;
	ai_summary_generated_at: string | null;
	message_metadata: Record<string, unknown>;
};

export type MailMessageDetailResponse = {
	message: MailMessageDetailItemV2;
	attachments: CommunicationAttachment[];
};

export type WorkflowStateCountItem = {
	state: string;
	count: number;
};

export type WorkflowStateCountsResponse = {
	counts: WorkflowStateCountItem[];
};

export type WorkflowStateTransitionRequest = {
	workflow_state: WorkflowState;
};


export type EmailThread = {
	thread_id: string;
	account_id: string;
	subject: string;
	message_count: number;
	participant_count: number;
	first_message_at: string | null;
	last_message_at: string | null;
	has_open_action: boolean;
	has_attachments: boolean;
	dominant_workflow_state: string;
};

export type ThreadMessage = {
	message_id: string;
	account_id: string;
	subject: string;
	sender: string;
	sender_display_name: string | null;
	body_text: string;
	occurred_at: string | null;
	projected_at: string;
	workflow_state: string;
	importance_score: number | null;
	ai_category: string | null;
	ai_summary: string | null;
	delivery_state: string;
	attachment_count: number;
};

export type ThreadListResponse = { items: EmailThread[] };
export type ThreadMessagesResponse = { items: ThreadMessage[] };

export type EmailRule = {
	rule_id: string;
	name: string;
	description_nl: string;
	conditions_json: Record<string, unknown>[];
	actions_json: Record<string, unknown>[];
	mode: 'suggest' | 'ask_before_execute' | 'auto_execute' | 'dry_run';
	enabled: boolean;
	match_count: number;
	last_matched_at: string | null;
	created_at: string;
	updated_at: string;
};

export type EmailTemplate = {
	template_id: string;
	name: string;
	subject_template: string;
	body_template: string;
	variables: string[];
	language: string | null;
	created_at: string;
	updated_at: string;
};

export type EmailPersona = {
	persona_id: string;
	name: string;
	account_id: string;
	display_name: string;
	signature: string;
	default_language: string | null;
	default_tone: string | null;
	is_default: boolean;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type MessageAnalyzeResponse = {
	message_id: string;
	analyzed: boolean;
	category: string | null;
	summary: string | null;
	importance_score: number | null;
	workflow_state: string;
};


export type EmailDraft = {
	draft_id: string;
	account_id: string;
	persona_id: string | null;
	to_recipients: string[];
	cc_recipients: string[];
	bcc_recipients: string[];
	subject: string;
	body_text: string;
	body_html: string | null;
	in_reply_to: string | null;
	references: string[];
	status: 'draft' | 'scheduled' | 'sending' | 'sent' | 'failed';
	scheduled_send_at: string | null;
	send_attempts: number;
	last_error: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type MailboxHealth = {
	total_messages: number;
	unread: number;
	needs_action: number;
	waiting: number;
	done: number;
	archived: number;
	spam: number;
	important: number;
	with_attachments: number;
	average_importance: number;
	oldest_message_days: number | null;
};

export type SenderStats = {
	sender: string;
	message_count: number;
	avg_importance: number;
	last_message_days: number | null;
};

export type DraftListResponse = { items: EmailDraft[] };
export type EmailSearchResponse = {
	results: { object_id: string; object_kind: string; title: string }[];
};
export type WorkflowStateTransitionResponse = {
	message_id: string;
	workflow_state: string;
	previous_state: string;
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

export type GraphEvidenceSourceKind = 'person' | 'message' | 'document' | 'raw_record';

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

export type PersonIdentityReviewState =
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type PersonIdentityCandidate = {
	identity_candidate_id: string;
	candidate_kind: 'merge_persons' | 'attach_email_address' | 'split_person';
	left_person_id: string;
	right_person_id: string | null;
	email_address: string | null;
	evidence_summary: string;
	confidence: number;
	review_state: PersonIdentityReviewState;
	generated_at: string;
	reviewed_at: string | null;
	updated_at: string;
};

export type PersonIdentityCandidateListResponse = {
	items: PersonIdentityCandidate[];
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

export type TaskCandidateListResponse = {
	items: TaskCandidate[];
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
	person_id?: string;
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

export type WhatsappWebProviderKind = 'whatsapp_web';

export type WhatsappCapabilityStatus = {
	capability: string;
	status: 'available' | 'blocked' | string;
	closure_gate: boolean;
	reason: string;
};

export type WhatsappCapabilitiesResponse = {
	version: string;
	runtime_mode: string;
	capabilities: WhatsappCapabilityStatus[];
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
	apiSecret: string,
	limit = 50
) : Promise<PersonIdentityCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/identity-candidates?${params.toString()}`,
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
	apiSecret: string
): Promise<V1Status> {
	return getJson(baseUrl, apiSecret, '/api/v1/status', 'V1 status request failed');
}

export async function fetchApplicationSettings(
	baseUrl: string,
	apiSecret: string
): Promise<ApplicationSettingsResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/settings', 'Settings request failed');
}

export async function saveApplicationSetting(
	baseUrl: string,
	apiSecret: string,
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	return putJson(
		baseUrl, apiSecret, `/api/v1/settings/${encodeURIComponent(settingKey)}`,
		{ value },
		'Setting update failed'
	);
}

export function findFrontendLayoutSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_LAYOUT_SETTING_KEY) ?? null;
}

export async function saveFrontendLayoutSetting(
	baseUrl: string,
	apiSecret: string,
	value: LayoutSettings
): Promise<ApplicationSetting> {
	return saveApplicationSetting(baseUrl, apiSecret, FRONTEND_LAYOUT_SETTING_KEY, value);
}

export async function fetchProviderAccounts(
	baseUrl: string,
	apiSecret: string
): Promise<ProviderAccountListResponse> {
	return getJson(
		baseUrl, apiSecret, '/api/v1/settings/accounts',
		'Provider accounts request failed'
	);
}

export async function fetchCommunicationMessages(
	baseUrl: string,
	apiSecret: string,
	limit = 50
): Promise<CommunicationMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	);
}

export async function fetchCommunicationMessage(
	baseUrl: string,
	apiSecret: string,
	messageId: string
): Promise<CommunicationMessageDetail> {
	return getJson(
		baseUrl, apiSecret, `/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Communication message detail request failed'
	);
}

export async function fetchGraphSummary(
	baseUrl: string,
	apiSecret: string
): Promise<GraphSummary> {
	return getJson(baseUrl, apiSecret, '/api/v1/graph/summary', 'Graph summary request failed');
}

export async function fetchGraphNodes(
	baseUrl: string,
	apiSecret: string,
	limit = 20
): Promise<GraphNode[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/graph/nodes?${params.toString()}`,
		'Graph node picker request failed'
	);
}

export async function searchGraphNodes(
	baseUrl: string,
	apiSecret: string,
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
		baseUrl, apiSecret, `/api/v1/graph/search?${params.toString()}`,
		'Graph search request failed'
	);
}

export async function fetchGraphNeighborhood(
	baseUrl: string,
	apiSecret: string,
	nodeId: string,
	depth = 1
): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({
		node_id: nodeId,
		depth: String(depth)
	});

	return getJson(
		baseUrl, apiSecret, `/api/v1/graph/neighborhood?${params.toString()}`,
		'Graph neighborhood request failed'
	);
}

export async function fetchProjects(
	baseUrl: string,
	apiSecret: string,
	limit = 25
): Promise<ProjectListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/projects?${params.toString()}`,
		'Projects request failed'
	);
}

export async function fetchProjectDetail(
	baseUrl: string,
	apiSecret: string,
	projectId: string
): Promise<ProjectDetail> {
	return getJson(
		baseUrl, apiSecret, `/api/v1/projects/${encodeURIComponent(projectId)}`,
		'Project detail request failed'
	);
}

export async function fetchTaskCandidates(
	baseUrl: string,
	apiSecret: string,
	limit = 50
): Promise<TaskCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/task-candidates?${params.toString()}`,
		'Task candidates request failed'
	);
}

export async function fetchDocumentProcessingJobs(
	baseUrl: string,
	apiSecret: string,
	limit = 50
): Promise<DocumentProcessingJobsResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/document-processing/jobs?${params.toString()}`,
		'Document processing jobs request failed'
	);
}

export async function fetchDocumentProcessing(
	baseUrl: string,
	apiSecret: string,
	documentId: string
): Promise<DocumentProcessingRecord> {
	return getJson(
		baseUrl, apiSecret, `/api/v1/documents/${encodeURIComponent(documentId)}/processing`,
		'Document processing request failed'
	);
}

export async function retryDocumentProcessingJob(
	baseUrl: string,
	apiSecret: string,
	jobId: string,
	request: DocumentProcessingRetryRequest
): Promise<DocumentProcessingRetryResponse> {
	return postJson(
		baseUrl, apiSecret, `/api/v1/document-processing/jobs/${encodeURIComponent(jobId)}/retry`,
		request,
		'Document processing retry request failed'
	);
}

export async function reviewTaskCandidate(
	baseUrl: string,
	apiSecret: string,
	taskCandidateId: string,
	reviewState: TaskCandidateReviewState
) {
	return putJson(
		baseUrl, apiSecret, `/api/v1/task-candidates/${encodeURIComponent(taskCandidateId)}/review`,
		{
			command_id: `task-candidate-review-${crypto.randomUUID()}`,
			review_state: reviewState
		}
	);
}

export async function reviewIdentityCandidate(
	baseUrl: string,
	apiSecret: string,
	identityCandidateId: string,
	reviewState: PersonIdentityReviewState,
	commandId = `person-identity-review-${crypto.randomUUID()}`
) {
	return putJson(
		baseUrl, apiSecret, `/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
		{
			command_id: commandId,
			review_state: reviewState
		}
	);
}

export async function startGmailOAuthSetup(
	baseUrl: string,
	apiSecret: string,
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return postJson(baseUrl, apiSecret, '/api/v1/email-accounts/gmail/oauth/start', request);
}

export async function completeGmailOAuthSetup(
	baseUrl: string,
	apiSecret: string,
	request: GmailOAuthCompleteRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, apiSecret, '/api/v1/email-accounts/gmail/oauth/complete', request);
}

export async function setupImapAccount(
	baseUrl: string,
	apiSecret: string,
	request: ImapAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, apiSecret, '/api/v1/email-accounts/imap', request);
}

export async function setupTelegramFixtureAccount(
	baseUrl: string,
	apiSecret: string,
	request: TelegramAccountSetupRequest
): Promise<TelegramAccountSetupResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/telegram/accounts/fixture',
		request,
		'Telegram account setup request failed'
	);
}

export async function setupTelegramAccount(
	baseUrl: string,
	apiSecret: string,
	request: TelegramLiveAccountSetupRequest
): Promise<TelegramAccountSetupResponse> {
	return postJson(
		baseUrl,
		apiSecret,
		'/api/v1/telegram/accounts',
		request,
		'Telegram account setup request failed'
	);
}

export async function startTelegramQrLogin(
	baseUrl: string,
	apiSecret: string,
	request: TelegramQrLoginStartRequest
): Promise<TelegramQrLoginStatusResponse> {
	return postJson(
		baseUrl,
		apiSecret,
		'/api/v1/telegram/login/qr/start',
		request,
		'Telegram QR login start failed'
	);
}

export async function fetchTelegramQrLoginStatus(
	baseUrl: string,
	apiSecret: string,
	setupId: string
): Promise<TelegramQrLoginStatusResponse> {
	return getJson(
		baseUrl,
		apiSecret,
		`/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}`,
		'Telegram QR login status request failed'
	);
}

export async function submitTelegramQrLoginPassword(
	baseUrl: string,
	apiSecret: string,
	setupId: string,
	request: TelegramQrLoginPasswordRequest
): Promise<TelegramQrLoginStatusResponse> {
	return postJson(
		baseUrl,
		apiSecret,
		`/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}/password`,
		request,
		'Telegram QR login password submit failed'
	);
}

export async function fetchTelegramCapabilities(
	baseUrl: string,
	apiSecret: string
): Promise<TelegramCapabilitiesResponse> {
	return getJson(
		baseUrl, apiSecret, '/api/v1/telegram/capabilities',
		'Telegram capabilities request failed'
	);
}

export async function fetchWhatsappCapabilities(
	baseUrl: string,
	apiSecret: string
): Promise<WhatsappCapabilitiesResponse> {
	return getJson(
		baseUrl, apiSecret, '/api/v1/whatsapp/capabilities',
		'WhatsApp capabilities request failed'
	);
}

export async function setupWhatsappWebFixtureAccount(
	baseUrl: string,
	apiSecret: string,
	request: WhatsappWebAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/whatsapp/accounts/fixture',
		request,
		'WhatsApp Web account setup request failed'
	);
}

export async function fetchWhatsappWebSessions(
	baseUrl: string,
	apiSecret: string,
	accountId?: string,
	limit = 50
): Promise<WhatsappWebSessionListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl, apiSecret, `/api/v1/whatsapp/sessions?${params.toString()}`,
		'WhatsApp Web sessions request failed'
	);
}

export async function fetchWhatsappWebMessages(
	baseUrl: string,
	apiSecret: string,
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
		baseUrl, apiSecret, `/api/v1/whatsapp/messages?${params.toString()}`,
		'WhatsApp Web messages request failed'
	);
}

export async function ingestWhatsappWebFixtureMessage(
	baseUrl: string,
	apiSecret: string,
	request: WhatsappWebFixtureMessageRequest
): Promise<WhatsappWebMessageIngestResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/whatsapp/messages',
		request,
		'WhatsApp Web fixture message request failed'
	);
}

export async function fetchTelegramChats(
	baseUrl: string,
	apiSecret: string,
	accountId?: string,
	limit = 50
): Promise<TelegramChatListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl, apiSecret, `/api/v1/telegram/chats?${params.toString()}`,
		'Telegram chats request failed'
	);
}

export async function fetchTelegramMessages(
	baseUrl: string,
	apiSecret: string,
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
		baseUrl, apiSecret, `/api/v1/telegram/messages?${params.toString()}`,
		'Telegram messages request failed'
	);
}

export async function ingestTelegramFixtureMessage(
	baseUrl: string,
	apiSecret: string,
	request: TelegramFixtureMessageRequest
): Promise<TelegramMessageIngestResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/telegram/messages',
		request,
		'Telegram fixture message request failed'
	);
}

export async function fetchAutomationTemplates(
	baseUrl: string,
	apiSecret: string
): Promise<AutomationTemplateListResponse> {
	return getJson(
		baseUrl, apiSecret, '/api/v1/policies/templates',
		'Automation template request failed'
	);
}

export async function saveAutomationTemplate(
	baseUrl: string,
	apiSecret: string,
	request: AutomationTemplateRequest
): Promise<AutomationTemplate> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/policies/templates',
		request,
		'Automation template save failed'
	);
}

export async function fetchAutomationPolicies(
	baseUrl: string,
	apiSecret: string
): Promise<AutomationPolicyListResponse> {
	return getJson(
		baseUrl, apiSecret, '/api/v1/policies',
		'Automation policy request failed'
	);
}

export async function saveAutomationPolicy(
	baseUrl: string,
	apiSecret: string,
	request: AutomationPolicyRequest
): Promise<AutomationPolicy> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/policies',
		request,
		'Automation policy save failed'
	);
}

export async function dryRunTelegramSend(
	baseUrl: string,
	apiSecret: string,
	request: TelegramSendDryRunRequest
): Promise<TelegramSendDryRunResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/policies/telegram-send/dry-run',
		request,
		'Telegram send dry-run failed'
	);
}

export async function fetchTelegramCalls(
	baseUrl: string,
	apiSecret: string,
	accountId?: string,
	limit = 50
): Promise<TelegramCallListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) {
		params.set('account_id', accountId.trim());
	}
	return getJson(
		baseUrl, apiSecret, `/api/v1/calls?${params.toString()}`,
		'Telegram call request failed'
	);
}

export async function saveTelegramCall(
	baseUrl: string,
	apiSecret: string,
	request: TelegramCallRequest
): Promise<TelegramCall> {
	return postJson(baseUrl, apiSecret, '/api/v1/calls', request, 'Telegram call save failed');
}

export async function saveCallTranscriptFixture(
	baseUrl: string,
	apiSecret: string,
	callId: string,
	request: CallTranscriptFixtureRequest
): Promise<CallTranscript> {
	return postJson(
		baseUrl, apiSecret, `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
		request,
		'Call transcript save failed'
	);
}

export async function fetchCallTranscript(
	baseUrl: string,
	apiSecret: string,
	callId: string
): Promise<CallTranscriptResponse> {
	return getJson(
		baseUrl, apiSecret, `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
		'Call transcript request failed'
	);
}

export async function fetchAiStatus(
	baseUrl: string,
	apiSecret: string
): Promise<AiStatus> {
	return getJson(baseUrl, apiSecret, '/api/v1/ai/status', 'AI status request failed');
}

export async function fetchAiAgents(
	baseUrl: string,
	apiSecret: string
): Promise<AiAgentListResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/ai/agents', 'AI agents request failed');
}

export async function fetchAiRuns(
	baseUrl: string,
	apiSecret: string,
	limit = 25
): Promise<AiRunListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(
		baseUrl, apiSecret, `/api/v1/ai/runs?${params.toString()}`,
		'AI run history request failed'
	);
}

export async function requestAiAnswer(
	baseUrl: string,
	apiSecret: string,
	request: AiAnswerRequest
): Promise<AiAnswerResponse> {
	return postJson(baseUrl, apiSecret, '/api/v1/ai/answers', request, 'AI answer request failed');
}

export async function refreshAiTaskCandidates(
	baseUrl: string,
	apiSecret: string,
	request: AiTaskCandidateRefreshRequest
): Promise<AiTaskCandidateRefreshResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/ai/task-candidates/refresh',
		request,
		'AI task candidate refresh request failed'
	);
}

export async function requestAiMeetingPrep(
	baseUrl: string,
	apiSecret: string,
	request: AiMeetingPrepRequest
): Promise<AiMeetingPrepResponse> {
	return postJson(
		baseUrl, apiSecret, '/api/v1/ai/meeting-prep',
		request,
		'AI meeting prep request failed'
	);
}


export async function fetchMailMessages(
	baseUrl: string,
	apiSecret: string,
	accountId?: string,
	workflowState?: WorkflowState,
	channelKind?: string,
	limit = 50
): Promise<MailMessagesResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (workflowState?.trim()) params.set('workflow_state', workflowState.trim());
	if (channelKind?.trim()) params.set('channel_kind', channelKind.trim());
	return getJson(
		baseUrl, apiSecret, `/api/v1/communications/messages?${params.toString()}`,
		'Mail messages request failed'
	);
}

export async function fetchMailMessage(
	baseUrl: string,
	apiSecret: string,
	messageId: string
): Promise<MailMessageDetailResponse> {
	return getJson(
		baseUrl, apiSecret, `/api/v1/communications/messages/${encodeURIComponent(messageId)}`,
		'Mail message detail request failed'
	);
}

export async function transitionMessageWorkflowState(
	baseUrl: string,
	apiSecret: string,
	messageId: string,
	workflowState: WorkflowState
): Promise<WorkflowStateTransitionResponse> {
	return putJson(
		baseUrl, apiSecret, `/api/v1/communications/messages/${encodeURIComponent(messageId)}/workflow-state`,
		{ workflow_state: workflowState },
		'Workflow state transition failed'
	);
}

export async function fetchMessageStateCounts(
	baseUrl: string,
	apiSecret: string,
	accountId?: string
): Promise<WorkflowStateCountsResponse> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	const qs = params.toString();
	return getJson(
		baseUrl, apiSecret, `/api/v1/communications/messages/states${qs ? '?' + qs : ''}`,
		'Message state counts request failed'
	);
}


export async function fetchThreads(
	baseUrl: string, apiSecret: string, accountId?: string, limit = 50
): Promise<ThreadListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	return getJson(baseUrl, apiSecret, `/api/v1/communications/threads?${params.toString()}`, 'Threads request failed');
}

export async function fetchThreadMessages(
	baseUrl: string, apiSecret: string, accountId: string, subject: string, limit = 50
): Promise<ThreadMessagesResponse> {
	const params = new URLSearchParams({ account_id: accountId, subject, limit: String(Math.trunc(limit)) });
	return getJson(baseUrl, apiSecret, `/api/v1/communications/threads/messages?${params.toString()}`, 'Thread messages failed');
}

export async function analyzeMessage(
	baseUrl: string, apiSecret: string, messageId: string
): Promise<MessageAnalyzeResponse> {
	return postJson(baseUrl, apiSecret, `/api/v1/communications/messages/${encodeURIComponent(messageId)}/analyze`, {}, 'Message analysis failed');
}

export async function searchEmails(
	baseUrl: string, apiSecret: string, query: string, limit = 20
): Promise<EmailSearchResponse> {
	const params = new URLSearchParams({ q: query, limit: String(Math.trunc(limit)) });
	return getJson(baseUrl, apiSecret, `/api/v1/communications/search?${params.toString()}`, 'Email search failed');
}


export async function fetchDrafts(
	baseUrl: string, apiSecret: string, accountId?: string, status?: string
): Promise<DraftListResponse> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	if (status?.trim()) params.set('status', status.trim());
	const qs = params.toString();
	return getJson(baseUrl, apiSecret, `/api/v1/communications/drafts${qs ? '?' + qs : ''}`, 'Drafts request failed');
}

export async function createDraft(
	baseUrl: string, apiSecret: string, draft: Record<string, unknown>
): Promise<EmailDraft> {
	return postJson(baseUrl, apiSecret, '/api/v1/communications/drafts', draft, 'Draft creation failed');
}

export async function fetchMailboxHealth(
	baseUrl: string, apiSecret: string, accountId?: string
): Promise<MailboxHealth> {
	const params = new URLSearchParams();
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	const qs = params.toString();
	return getJson(baseUrl, apiSecret, `/api/v1/communications/analytics/health${qs ? '?' + qs : ''}`, 'Health request failed');
}

export async function fetchTopSenders(
	baseUrl: string, apiSecret: string, accountId?: string, limit = 20
): Promise<SenderStats[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (accountId?.trim()) params.set('account_id', accountId.trim());
	return getJson(baseUrl, apiSecret, `/api/v1/communications/analytics/senders?${params.toString()}`, 'Senders request failed');
}


export type EnrichedPerson = {
	person_id: string;
	display_name: string;
	email_address: string;
	language: string | null;
	tone: string | null;
	trust_score: number | null;
	avg_response_hours: number | null;
	preferred_channel: string | null;
	last_interaction_at: string | null;
	interaction_count: number;
	frequent_topics: string[];
	writing_style: string | null;
	person_metadata: Record<string, unknown>;
	is_favorite: boolean;
	notes: string | null;
	linked_projects: string[];
	linked_documents: string[];
	created_at: string;
	updated_at: string;
};

export type PersonListResponse = {
	items: EnrichedPerson[];
};

export async function fetchPersons(
	baseUrl: string, apiSecret: string, limit = 50, favoritesOnly = false
): Promise<PersonListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (favoritesOnly) params.set('favorites_only', 'true');
	return getJson(baseUrl, apiSecret, `/api/v1/persons?${params.toString()}`, 'Persons request failed');
}

export async function fetchPerson(
	baseUrl: string, apiSecret: string, personId: string
): Promise<EnrichedPerson> {
	return getJson(baseUrl, apiSecret, `/api/v1/persons/${encodeURIComponent(personId)}`, 'Person request failed');
}

export type Organization = {
	organization_id: string;
	display_name: string;
	legal_name: string | null;
	org_type: string | null;
	status: string;
	country: string | null;
	city: string | null;
	address: string | null;
	website: string | null;
	industry: string | null;
	description: string | null;
	primary_language: string | null;
	timezone: string | null;
	trust_score: number | null;
	health_status: string | null;
	priority: string | null;
	notes: string | null;
	tags: string[];
	org_metadata: Record<string, unknown>;
	last_interaction_at: string | null;
	interaction_count: number;
	registration_number: string | null;
	country_of_registration: string | null;
	vat: string | null;
	cif: string | null;
	nif: string | null;
	tax_id: string | null;
	legal_address: string | null;
	registry_source: string | null;
	registry_last_verified: string | null;
	communication_style: string | null;
	verbosity: string | null;
	formality: string | null;
	secondary_languages: string[] | null;
	preferred_tone: string | null;
	official_style_required: boolean | null;
	last_health_check: string | null;
	watchlist: boolean;
	created_at: string;
	updated_at: string;
};

export type OrganizationListResponse = {
	items: Organization[];
};

export async function fetchOrganizations(
	baseUrl: string, apiSecret: string, limit = 50
): Promise<OrganizationListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return getJson(baseUrl, apiSecret, `/api/v1/organizations?${params.toString()}`, 'Organizations request failed');
}

export async function fetchOrganization(
	baseUrl: string, apiSecret: string, orgId: string
): Promise<Organization> {
	return getJson(baseUrl, apiSecret, `/api/v1/organizations/${encodeURIComponent(orgId)}`, 'Organization request failed');
}


async function getJson<TResponse>(
	baseUrl: string,
	apiSecret: string,
	path: string,
	fallbackMessage: string
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		headers: {
			'X-Hermes-Secret': apiSecret
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
	apiSecret: string,
	path: string,
	body: unknown,
	fallbackMessage = 'Account setup request failed'
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			'X-Hermes-Secret': apiSecret
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
	apiSecret: string,
	path: string,
	body: unknown,
	fallbackMessage = 'PUT request failed'
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json',
			'X-Hermes-Secret': apiSecret
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

// ── Calendar Types ────────────────────────────────────────────────────────

export type CalendarAccount = {
	account_id: string;
	provider: string;
	account_name: string;
	email: string | null;
	credentials_reference: string | null;
	sync_status: string;
	capabilities: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CalendarAccountsResponse = { items: CalendarAccount[] };

export type CalendarSource = {
	source_id: string;
	account_id: string;
	provider_calendar_id: string | null;
	name: string;
	color: string | null;
	timezone: string | null;
	visibility: string;
	read_only: boolean;
	sync_enabled: boolean;
	capabilities: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CalendarSourcesResponse = { items: CalendarSource[] };

export type CalendarEvent = {
	event_id: string;
	source_event_id: string | null;
	account_id: string | null;
	source_id: string | null;
	title: string;
	description: string | null;
	location: string | null;
	start_at: string;
	end_at: string;
	timezone: string | null;
	all_day: boolean;
	recurrence_rule: string | null;
	status: string;
	visibility: string;
	event_type: string | null;
	importance_score: number | null;
	readiness_score: number | null;
	sync_status: string;
	created_at: string;
	updated_at: string;
};

export type CalendarEventsResponse = { items: CalendarEvent[] };

export type EventParticipant = {
	id: string;
	event_id: string;
	person_id: string | null;
	email: string;
	display_name: string | null;
	role: string;
	response_status: string;
	organization_id: string | null;
	timezone: string | null;
	confidence: number;
	created_at: string;
};

export type EventParticipantsResponse = { items: EventParticipant[] };

export type EventRelation = {
	id: string;
	event_id: string;
	entity_type: string;
	entity_id: string;
	relation_type: string;
	source: string;
	confidence: number;
	created_at: string;
};

export type EventRelationsResponse = { items: EventRelation[] };

export type EventContextPack = {
	id: string;
	event_id: string;
	summary: string | null;
	participants_summary: string | null;
	documents: unknown[];
	tasks: unknown[];
	open_questions: unknown[];
	risks: unknown[];
	suggested_agenda: unknown[];
	suggested_actions: unknown[];
	generated_at: string;
	model: string | null;
	created_at: string;
	updated_at: string;
};

export type EventAgenda = {
	id: string;
	event_id: string;
	items: unknown[];
	source: string;
	created_by: string | null;
	created_at: string;
	updated_at: string;
};

export type EventChecklist = {
	id: string;
	event_id: string;
	items: unknown[];
	source: string;
	created_at: string;
	updated_at: string;
};

export type MeetingNote = {
	id: string;
	event_id: string;
	content: string;
	format: string;
	source: string;
	linked_note_id: string | null;
	created_at: string;
	updated_at: string;
};

export type MeetingNotesResponse = { items: MeetingNote[] };

export type MeetingOutcome = {
	id: string;
	event_id: string;
	outcome_type: string;
	title: string;
	description: string | null;
	owner_person_id: string | null;
	due_date: string | null;
	source: string;
	confidence: number;
	linked_entity_id: string | null;
	created_at: string;
	updated_at: string;
};

export type MeetingOutcomesResponse = { items: MeetingOutcome[] };

export type DeadlineEvent = {
	id: string;
	source_entity_type: string | null;
	source_entity_id: string | null;
	title: string;
	due_at: string;
	severity: string;
	status: string;
	linked_calendar_event_id: string | null;
	created_at: string;
	updated_at: string;
};

export type DeadlinesResponse = { items: DeadlineEvent[] };

export type FocusBlock = {
	id: string;
	title: string;
	start_at: string;
	end_at: string;
	purpose: string | null;
	linked_project_id: string | null;
	protection_level: string;
	status: string;
	created_at: string;
	updated_at: string;
};

export type FocusBlocksResponse = { items: FocusBlock[] };

export type CalendarRule = {
	rule_id: string;
	name: string;
	natural_language_description: string | null;
	compiled_dsl: Record<string, unknown>;
	enabled: boolean;
	approval_mode: string;
	last_run_at: string | null;
	created_at: string;
	updated_at: string;
};

export type CalendarRulesResponse = { items: CalendarRule[] };

// ── Calendar API Functions ─────────────────────────────────────────────────

export async function fetchCalendarAccounts(
	baseUrl: string, apiSecret: string, provider?: string
): Promise<CalendarAccountsResponse> {
	const params = new URLSearchParams();
	if (provider) params.set('provider', provider);
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/accounts?${params.toString()}`, 'Calendar accounts request failed');
}

export async function createCalendarAccount(
	baseUrl: string, apiSecret: string,
	body: { provider: string; account_name: string; email?: string }
): Promise<CalendarAccount> {
	return postJson(baseUrl, apiSecret, '/api/v1/calendar/accounts', body, 'Create calendar account failed');
}

export async function fetchCalendarSources(
	baseUrl: string, apiSecret: string, accountId: string
): Promise<CalendarSourcesResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/accounts/${encodeURIComponent(accountId)}/sources`, 'Calendar sources request failed');
}

export async function fetchCalendarEvents(
	baseUrl: string, apiSecret: string,
	params: { account_id?: string; source_id?: string; from?: string; to?: string; status?: string; event_type?: string; limit?: number }
): Promise<CalendarEventsResponse> {
	const sp = new URLSearchParams();
	if (params.account_id) sp.set('account_id', params.account_id);
	if (params.source_id) sp.set('source_id', params.source_id);
	if (params.from) sp.set('from', params.from);
	if (params.to) sp.set('to', params.to);
	if (params.status) sp.set('status', params.status);
	if (params.event_type) sp.set('event_type', params.event_type);
	if (params.limit) sp.set('limit', String(params.limit));
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events?${sp.toString()}`, 'Calendar events request failed');
}

export async function createCalendarEvent(
	baseUrl: string, apiSecret: string,
	body: { title: string; start_at: string; end_at: string; description?: string; location?: string; event_type?: string; account_id?: string; source_id?: string; timezone?: string; all_day?: boolean }
): Promise<CalendarEvent> {
	return postJson(baseUrl, apiSecret, '/api/v1/calendar/events', body, 'Create event failed');
}

export async function deleteCalendarEvent(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<{ deleted: boolean }> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}/api/v1/calendar/events/${encodeURIComponent(eventId)}`, {
		method: 'DELETE',
		headers: { 'X-Hermes-Secret': apiSecret }
	});
	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as { message?: string } | null;
		throw new Error(error?.message ?? `Delete event failed: ${response.status}`);
	}
	return (await response.json()) as { deleted: boolean };
}

export async function fetchEventParticipants(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<EventParticipantsResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/participants`, 'Participants request failed');
}

export async function fetchEventContextPack(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<EventContextPack | null> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/context-pack`, 'Context pack request failed');
}

export async function fetchEventAgenda(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<EventAgenda | null> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/agenda`, 'Agenda request failed');
}

export async function fetchEventChecklist(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<EventChecklist | null> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/checklist`, 'Checklist request failed');
}

export async function fetchEventBrief(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/brief`, 'Brief request failed');
}

export async function fetchMeetingNotes(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<MeetingNotesResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/notes`, 'Notes request failed');
}

export async function fetchMeetingOutcomes(
	baseUrl: string, apiSecret: string, eventId: string
): Promise<MeetingOutcomesResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/events/${encodeURIComponent(eventId)}/outcomes`, 'Outcomes request failed');
}

export async function fetchDeadlines(
	baseUrl: string, apiSecret: string, status?: string
): Promise<DeadlinesResponse> {
	const params = new URLSearchParams();
	if (status) params.set('status', status);
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/deadlines?${params.toString()}`, 'Deadlines request failed');
}

export async function fetchFocusBlocks(
	baseUrl: string, apiSecret: string
): Promise<FocusBlocksResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/calendar/focus-blocks', 'Focus blocks request failed');
}

export async function fetchCalendarWatchtower(
	baseUrl: string, apiSecret: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, '/api/v1/calendar/watchtower', 'Watchtower request failed');
}

export async function fetchWeeklyBrief(
	baseUrl: string, apiSecret: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, '/api/v1/calendar/weekly-brief', 'Weekly brief request failed');
}

export async function fetchCalendarRules(
	baseUrl: string, apiSecret: string
): Promise<CalendarRulesResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/calendar/rules', 'Rules request failed');
}

export async function postCalendarBrain(
	baseUrl: string, apiSecret: string, question: string
): Promise<Record<string, unknown>> {
	return postJson(baseUrl, apiSecret, '/api/v1/calendar/brain', { q: question }, 'Brain query failed');
}

export async function searchCalendarEvents(
	baseUrl: string, apiSecret: string, q: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, `/api/v1/calendar/search?q=${encodeURIComponent(q)}`, 'Calendar search failed');
}

// ── Task Types ───────────────────────────────────────────────────────────

export type Task = {
	task_id: string;
	task_candidate_id: string | null;
	title: string;
	description: string | null;
	source_kind: string;
	source_id: string;
	source_type: string;
	project_id: string | null;
	status: string;
	hermes_status: string;
	priority_score: number | null;
	risk_score: number | null;
	readiness_score: number | null;
	area: string | null;
	why: string | null;
	outcome: string | null;
	due_at: string | null;
	completed_at: string | null;
	archived_at: string | null;
	waiting_reason: string | null;
	energy_type: string | null;
	confidentiality: string;
	tags: unknown[];
	task_metadata: Record<string, unknown>;
	linked_person_id: string | null;
	linked_organization_id: string | null;
	created_from_event_id: string | null;
	created_by_actor_id: string | null;
	created_at: string;
	updated_at: string;
};

export type TaskRecordsResponse = { items: Task[] };

export type TaskContextPack = {
	id: string;
	task_id: string;
	summary: string | null;
	source_summary: string | null;
	open_questions: unknown[];
	blockers: unknown[];
	risks: unknown[];
	suggested_next_action: string | null;
	generated_at: string;
	model: string | null;
};

export type TaskEvidence = {
	id: string; task_id: string; source_type: string;
	source_id: string; quote: string | null; confidence: number;
	created_at: string;
};

export type TaskEvidenceResponse = { items: TaskEvidence[] };

export type TaskRelation = {
	id: string; task_id: string; entity_type: string;
	entity_id: string; relation_type: string;
	source: string; confidence: number; created_at: string;
};

export type TaskRelationsResponse = { items: TaskRelation[] };

export type TaskChecklist = {
	id: string; task_id: string; items: unknown[];
	source: string; created_at: string; updated_at: string;
};

export type TaskSubtask = {
	id: string; parent_task_id: string; child_task_id: string;
	sort_order: number; created_at: string;
};

export type TaskSubtasksResponse = { items: TaskSubtask[] };

export type TaskProviderAccount = {
	account_id: string; provider: string; account_name: string;
	credentials_reference: string | null; sync_mode: string;
	capabilities: Record<string, unknown>;
};

export type TaskProvidersResponse = { items: TaskProviderAccount[] };

export type TaskRule = {
	rule_id: string; name: string; natural_language_description: string | null;
	compiled_dsl: Record<string, unknown>; enabled: boolean;
	approval_mode: string; last_run_at: string | null;
};

export type TaskRulesResponse = { items: TaskRule[] };

export type TaskTemplate = {
	template_id: string; name: string; description: string | null;
	default_fields: Record<string, unknown>; default_checklist: unknown[];
	default_priority: string; default_energy_type: string | null;
};

export type TaskTemplatesResponse = { items: TaskTemplate[] };

// ── Task API Functions ────────────────────────────────────────────────────

export async function fetchTaskRecords(
	baseUrl: string, apiSecret: string,
	params: { status?: string; project_id?: string; source_type?: string; limit?: number }
): Promise<TaskRecordsResponse> {
	const sp = new URLSearchParams();
	if (params.status) sp.set('status', params.status);
	if (params.project_id) sp.set('project_id', params.project_id);
	if (params.source_type) sp.set('source_type', params.source_type);
	if (params.limit) sp.set('limit', String(params.limit));
	return getJson(baseUrl, apiSecret, `/api/v1/tasks?${sp.toString()}`, 'Tasks request failed');
}

export async function fetchTask(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<Task> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}`, 'Task request failed');
}

export async function createTask(
	baseUrl: string, apiSecret: string,
	body: { title: string; description?: string; source_type?: string; project_id?: string; hermes_status?: string; priority_score?: number; due_at?: string; area?: string; linked_person_id?: string }
): Promise<Task> {
	return postJson(baseUrl, apiSecret, '/api/v1/tasks', body, 'Create task failed');
}

export async function updateTask(
	baseUrl: string, apiSecret: string, taskId: string,
	body: Record<string, unknown>
): Promise<Task> {
	return putJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}`, body, 'Update task failed');
}

export async function setTaskStatus(
	baseUrl: string, apiSecret: string, taskId: string, status: string
): Promise<{ status: string }> {
	return postJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/status`, { status }, 'Set status failed');
}

export async function archiveTask(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<{ archived: boolean }> {
	return postJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/archive`, {}, 'Archive failed');
}

export async function fetchTaskContextPack(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<TaskContextPack | null> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/context-pack`, 'Context pack failed');
}

export async function fetchTaskEvidence(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<TaskEvidenceResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/evidence`, 'Evidence failed');
}

export async function fetchTaskRelations(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<TaskRelationsResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/relations`, 'Relations failed');
}

export async function fetchTaskChecklist(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<TaskChecklist | null> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/checklist`, 'Checklist failed');
}

export async function fetchTaskSubtasks(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<TaskSubtasksResponse> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/subtasks`, 'Subtasks failed');
}

export async function analyzeTask(
	baseUrl: string, apiSecret: string, taskId: string
): Promise<Record<string, unknown>> {
	return postJson(baseUrl, apiSecret, `/api/v1/tasks/${encodeURIComponent(taskId)}/analyze`, {}, 'Analyze failed');
}

export async function fetchTaskProviders(
	baseUrl: string, apiSecret: string
): Promise<TaskProvidersResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/tasks/providers', 'Providers failed');
}

export async function fetchTaskRules(
	baseUrl: string, apiSecret: string
): Promise<TaskRulesResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/tasks/rules', 'Rules failed');
}

export async function fetchTaskTemplates(
	baseUrl: string, apiSecret: string
): Promise<TaskTemplatesResponse> {
	return getJson(baseUrl, apiSecret, '/api/v1/tasks/templates', 'Templates failed');
}

export async function fetchTaskWatchtower(
	baseUrl: string, apiSecret: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, '/api/v1/tasks/watchtower', 'Watchtower failed');
}

export async function fetchTaskDailyBrief(
	baseUrl: string, apiSecret: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, '/api/v1/tasks/daily-brief', 'Daily brief failed');
}

export async function searchTasks(
	baseUrl: string, apiSecret: string, q: string
): Promise<Record<string, unknown>> {
	return getJson(baseUrl, apiSecret, `/api/v1/tasks/search?q=${encodeURIComponent(q)}`, 'Task search failed');
}
