import type { CommunicationAttachment, LocalMessageState, WorkflowState } from './communication';

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

export type MessageImportantToggleResponse = {
	message_id: string;
	important: boolean;
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
