import type { FrontendThemeSettings, LayoutSettings, SidebarSettings } from '$lib/layout';

export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		persons: boolean;
		search: boolean;
		documents: boolean;
		account_setup: boolean;
	};
	vault_status: VaultStatus;
};

export type VaultMode = 'uninitialized' | 'locked' | 'unlocked';

export type VaultStatus = {
	state: VaultMode;
	needs_entropy: boolean;
	needs_biometric: boolean;
	needs_recovery: boolean;
	version: number;
	recoverable: boolean;
	entropy_progress: number;
};

export type VaultEntropyEvent = {
	x: number;
	y: number;
	dx: number;
	dy: number;
	timestamp_ms: number;
	velocity: number;
	acceleration: number;
	interval_ms: number;
};

export type VaultRecoveryExportResponse = {
	path: string;
	recovery_phrase: string;
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
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar';
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale';
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme';
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state';

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
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
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
	body_html: string | null;
	occurred_at: string | null;
	projected_at: string;
	channel_kind: string;
	conversation_id: string | null;
	sender_display_name: string | null;
	delivery_state: string;
	message_metadata: Record<string, unknown>;
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
	local_state_reason: string | null;
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
export type LocalMessageState = 'active' | 'trash' | 'all';

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
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
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
	body_html: string | null;
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
	local_state: LocalMessageState;
	local_state_changed_at: string | null;
	local_state_reason: string | null;
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

export type LocalMessageStateResponse = {
	message_id: string;
	local_state: LocalMessageState;
	provider_deleted?: boolean;
};

export type MailSyncSettings = {
	account_id: string;
	sync_enabled: boolean;
	batch_size: number;
	poll_interval_seconds: number;
	updated_at: string;
};

export type MailSyncSettingsUpdate = {
	sync_enabled: boolean;
	batch_size: number;
	poll_interval_seconds: number;
};

export type MailSyncStatus = {
	account_id: string;
	status: string;
	phase: string;
	progress_mode: 'none' | 'determinate' | 'indeterminate' | string;
	progress_percent: number | null;
	processed_messages: number;
	estimated_total_messages: number | null;
	current_batch_size: number;
	last_started_at: string | null;
	last_completed_at: string | null;
	next_run_at: string | null;
	last_error_code: string | null;
	last_error_message: string | null;
	last_fetched_messages: number;
	last_projected_messages: number;
	last_upserted_persons: number;
	last_upserted_organizations: number;
};

export type MailSyncStatusListResponse = {
	items: MailSyncStatus[];
};

export type MailSyncFailureReason = {
	code: string;
	message: string;
};

export type MailSyncRunResponse = {
	run_id: string;
	account_id: string;
	trigger: string;
	status: string;
	phase: string;
	progress_mode: 'none' | 'determinate' | 'indeterminate' | string;
	progress_percent: number | null;
	processed_messages: number;
	estimated_total_messages: number | null;
	current_batch_size: number;
	fetched_messages: number;
	projected_messages: number;
	upserted_persons: number;
	upserted_organizations: number;
	checkpoint_before_present: boolean;
	checkpoint_after_present: boolean;
	checkpoint_saved: boolean;
	failure_reason: MailSyncFailureReason | null;
	started_at: string;
	completed_at: string | null;
	next_run_at: string | null;
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
	source: string;
	confidence: number | null;
	evidence: string[];
};

export type WorkflowActionKind =
	| 'reply'
	| 'create_task'
	| 'create_note'
	| 'create_document'
	| 'create_event'
	| 'link_document'
	| 'create_contact'
	| 'archive';

export type WorkflowActionSource = {
	kind: 'communication_message';
	id: string;
};

export type WorkflowActionRequest = {
	command_id: string;
	action: WorkflowActionKind;
	source?: WorkflowActionSource;
	input?: {
		title?: string;
		body?: string;
		email?: string;
		display_name?: string;
		starts_at?: string;
		ends_at?: string;
		due_at?: string;
		document_id?: string;
	};
};

export type WorkflowActionResponse = {
	command_id: string;
	event_id: string;
	action: WorkflowActionKind;
	status: 'created' | 'updated' | 'linked' | 'opened' | 'archived' | 'noop';
	target: {
		kind: 'compose' | 'message' | 'task' | 'document' | 'calendar_event' | 'person';
		id: string | null;
	};
	provenance: {
		source_kind?: string;
		source_id?: string;
		confidence: number | null;
		evidence: string[];
	};
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

export type DraftDeleteResponse = {
	deleted: boolean;
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

export type SendEmailRequest = {
	account_id: string;
	to: string[];
	cc?: string[];
	bcc?: string[];
	subject: string;
	body_text: string;
	body_html?: string | null;
	in_reply_to?: string | null;
	references?: string[];
	confirmed_provider_write: boolean;
};

export type SendEmailResponse = {
	message_id: string;
	accepted: string[];
	accepted_recipients: string[];
	transport: 'smtp' | 'local' | string;
	status: 'sent' | 'queued' | string;
	failure_reason: string | null;
};

export type MessageExplainResponse = {
	reasons: string[];
};

export type SmartCcResponse = {
	suggestions: string[];
};

export type MessagePinToggleResponse = {
	message_id: string;
	pinned: boolean;
};

export type MessageExportResponse = {
	content_type: string;
	content: string;
	filename: string;
};

export type MailAuthResult = {
	result: string;
	domain?: string | null;
	ip?: string | null;
	selector?: string | null;
	policy?: string | null;
};

export type MailAuthResults = {
	spf: MailAuthResult | null;
	dkim: MailAuthResult | null;
	dmarc: MailAuthResult | null;
	raw_headers: string[];
};

export type MailAuthRisk = {
	has_spf: boolean;
	spf_pass: boolean;
	has_dkim: boolean;
	dkim_pass: boolean;
	has_dmarc: boolean;
	dmarc_pass: boolean;
	is_spoofed: boolean;
	risk_summary: string;
};

export type MessageAuthCheckResponse = {
	auth: MailAuthResults;
	risk: MailAuthRisk;
};

export type SignatureDetection = {
	has_signature: boolean;
	signature_type: string | null;
	signer_info: string | null;
	is_valid: boolean | null;
	cert_expiry_warning: string | null;
};

export type LanguageDetection = {
	language: string;
	confidence: number;
	script: string | null;
};

export type TranslationResponse = {
	translated: boolean;
	text?: string;
	target?: string;
	model?: string;
	reason?: string;
};

export type AiReplyResponse = {
	subject?: string;
	body?: string;
	tone?: string;
	language?: string;
	generated?: boolean;
	reason?: string;
};

export type ExtractedTask = {
	title: string;
	due_date: string | null;
	assignee: string | null;
	priority: string | null;
	source: string;
};

export type ExtractedNote = {
	title: string;
	content: string;
	tags: string[];
	source: string;
};

export type ExtractTasksResponse = {
	tasks: ExtractedTask[];
};

export type ExtractNotesResponse = {
	notes: ExtractedNote[];
};

export type SubscriptionSource = {
	sender: string;
	message_count: number;
	first_seen: string;
	last_seen: string;
	is_newsletter: boolean;
	has_unsubscribe: boolean;
};

export type DuplicateAttachmentGroup = {
	sha256: string;
	filenames: string[];
	message_ids: string[];
	count: number;
};

export type InvoiceStatus =
	| 'received'
	| 'recognized'
	| 'needs_review'
	| 'approved'
	| 'paid'
	| 'closed'
	| 'rejected';

export type InvoiceRecord = {
	invoice_id: string;
	message_id: string | null;
	amount: number | null;
	currency: string | null;
	invoice_number: string | null;
	issue_date: string | null;
	due_date: string | null;
	counterparty: string | null;
	tax_id: string | null;
	status: InvoiceStatus;
	linked_project_id: string | null;
	linked_person_id: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type InvoiceListResponse = {
	items: InvoiceRecord[];
};

export type LegalDocument = {
	document_id: string;
	message_id: string | null;
	document_type: string;
	title: string;
	parties: string[];
	effective_date: string | null;
	expiry_date: string | null;
	amount: number | null;
	currency: string | null;
	status: string;
	linked_project_id: string | null;
	risks: string[];
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type LegalDocumentListResponse = {
	items: LegalDocument[];
};

export type CertificateRecord = {
	cert_id: string;
	owner_name: string;
	issuer: string;
	serial_number: string | null;
	fingerprint_sha256: string | null;
	valid_from: string | null;
	valid_until: string | null;
	cert_type: string;
	provider: string;
	storage_kind: string;
	storage_ref: string | null;
	trust_status: string;
	is_revoked: boolean;
	usage: string[];
	linked_message_id: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type CertificateListResponse = {
	items: CertificateRecord[];
};

export type RichTemplateListResponse = {
	templates: EmailTemplate[];
};

export type RenderTemplateResponse = {
	rendered: boolean;
	template_id: string;
	variables: Record<string, string>;
};

export type MailArchitectureBlocker = {
	section: string;
	feature: string;
	reason: string;
	resolution: string;
};

export type MailMessageInsight = {
	messageId: string;
	explain: MessageExplainResponse | null;
	smartCc: SmartCcResponse | null;
	auth: MessageAuthCheckResponse | null;
	signature: SignatureDetection | null;
	language: LanguageDetection | null;
	aiReply: AiReplyResponse | null;
	tasks: ExtractedTask[];
	notes: ExtractedNote[];
	translation: TranslationResponse | null;
};

export type MailResourceSnapshot = {
	subscriptions: SubscriptionSource[];
	duplicates: DuplicateAttachmentGroup[];
	invoices: InvoiceRecord[];
	legalDocuments: LegalDocument[];
	certificates: CertificateRecord[];
	expiringCertificates: CertificateRecord[];
	personas: EmailPersona[];
	templates: EmailTemplate[];
	blockers: MailArchitectureBlocker[];
};

export type MailResourceSummary = {
	subscriptions: number;
	duplicates: number;
	invoices: number;
	legalDocuments: number;
	certificates: number;
	expiringCertificates: number;
	personas: number;
	templates: number;
	blockers: number;
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

export type AiProviderKind = 'built_in' | 'cli' | 'api' | string;

export type AiProviderAccount = {
	provider_id: string;
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	status: 'ready' | 'disabled' | 'needs_setup' | 'unavailable' | string;
	consent_state: 'not_required' | 'required' | 'granted' | string;
	consented_at: string | null;
	config: Record<string, unknown>;
	capabilities: string[];
	created_at: string;
	updated_at: string;
};

export type AiProviderPreset = {
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	privacy: 'local' | 'cli' | 'remote' | string;
	base_url: string | null;
	command_preset: string | null;
	capabilities: string[];
};

export type AiCapabilitySlot = {
	slot: string;
	label: string;
	description: string;
	requires_embedding_dimension: number | null;
};

export type AiModelCatalogItem = {
	model_key: string;
	provider_id: string;
	display_name: string;
	category: string;
	privacy: 'local' | 'cli' | 'remote' | string;
	capabilities: string[];
	context_window: number | null;
	embedding_dimension: number | null;
	is_available: boolean;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type AiModelRoute = {
	capability_slot: string;
	provider_id: string;
	model_key: string;
	created_at: string;
	updated_at: string;
};

export type AiPromptTemplate = {
	prompt_id: string;
	name: string;
	entity_scope: string;
	capability_slot: string;
	description: string | null;
	is_system: boolean;
	active_version_id: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type AiPromptVersion = {
	prompt_version_id: string;
	prompt_id: string;
	version_label: string;
	body_template: string;
	variables: string[];
	status: 'active' | 'draft' | string;
	created_by_actor_id: string;
	created_at: string;
	updated_at: string;
};

export type AiPromptEvalRun = {
	eval_run_id: string;
	prompt_id: string;
	prompt_version_id: string;
	provider_id: string;
	model_key: string;
	source_refs: Record<string, unknown>[];
	variables: Record<string, unknown>;
	output_text: string;
	score: number | null;
	notes: string | null;
	actor_id: string;
	created_at: string;
};

export type AiSettingsOverviewResponse = {
	providers: AiProviderAccount[];
	models: AiModelCatalogItem[];
	routes: AiModelRoute[];
	prompts: AiPromptTemplate[];
	eval_runs: AiPromptEvalRun[];
	capability_slots: AiCapabilitySlot[];
	provider_presets: AiProviderPreset[];
};

export type AiProviderListResponse = { items: AiProviderAccount[] };
export type AiModelListResponse = { items: AiModelCatalogItem[] };
export type AiPromptListResponse = { items: AiPromptTemplate[] };

export type AiProviderCreateRequest = {
	provider_id?: string;
	provider_kind: AiProviderKind;
	provider_key: string;
	display_name: string;
	base_url?: string;
	command_preset?: string;
	config?: Record<string, unknown>;
	capabilities?: string[];
	enabled?: boolean;
	remote_context_consent?: boolean;
	api_key?: string;
};

export type AiProviderPatchRequest = {
	display_name?: string;
	base_url?: string;
	config?: Record<string, unknown>;
	enabled?: boolean;
	api_key?: string;
};

export type AiProviderConsentRequest = {
	consented: boolean;
};

export type AiProviderCommandResponse = {
	provider_id: string;
	command: 'test' | 'sync_models' | string;
	status: string;
	message: string;
};

export type AiModelRouteUpdateRequest = {
	provider_id: string;
	model_key: string;
};

export type AiPromptCreateRequest = {
	prompt_id?: string;
	name: string;
	entity_scope: string;
	capability_slot: string;
	description?: string;
	metadata?: Record<string, unknown>;
};

export type AiPromptVersionCreateRequest = {
	prompt_version_id?: string;
	version_label?: string;
	body_template: string;
	variables?: string[];
	metadata?: Record<string, unknown>;
};

export type AiPromptActivateRequest = {
	prompt_version_id: string;
};

export type AiPromptTestRequest = {
	prompt_version_id?: string;
	provider_id: string;
	model_key: string;
	variables?: Record<string, unknown>;
	source_refs?: Record<string, unknown>[];
	score?: number;
	notes?: string;
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
	external_account_id?: string;
	client_id?: string;
	client_secret?: string;
	redirect_uri: string;
	app_return_url?: string;
	scopes?: string[];
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
	external_account_id?: string;
};

export type EmailAccountSetupResponse = {
	account_id: string;
	secret_ref: string;
	secret_kind: 'oauth_token' | 'app_password' | 'password';
	store_kind: 'encrypted_vault' | 'database_encrypted_vault' | 'host_vault';
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
