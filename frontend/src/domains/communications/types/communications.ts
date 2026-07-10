// --- Re-exported API types from Svelte reference ---
import type { MailCertificate } from './certificates'
import type { CommunicationAiState } from './aiState'
export type {
  EmailAccountCapabilities,
  EmailAccountListResponse,
  EmailAccountView,
  EmailProviderAccount
} from './mailAccounts'
export type LocalMessageState = 'active' | 'trash' | 'all'
export type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'

export type CommunicationMessageSummary = {
  message_id: string
  raw_record_id: string
  observation_id?: string | null
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  workflow_state: WorkflowState
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  ai_summary_generated_at: string | null
  ai_state?: CommunicationAiState | null
  message_metadata: Record<string, unknown>
  attachment_count: number
  local_state: LocalMessageState
  local_state_changed_at: string | null
}

export type CommunicationMessagesResponse = {
  items: CommunicationMessageSummary[]
  next_cursor: string | null
  has_more: boolean
}
export type CommunicationAttachment = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  blob_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  disposition: 'attachment' | 'inline' | 'unknown'
  scan_status: 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed'
  scan_engine: string | null
  scan_checked_at: string | null
  scan_summary: string | null
  scan_metadata: Record<string, unknown>
  storage_kind: string
  storage_path: string
  created_at: string
  updated_at: string
}

export type CommunicationMessageDetailItem = {
  message_id: string
  raw_record_id: string
  observation_id?: string | null
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text: string
  body_html: string | null
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  workflow_state: WorkflowState
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  ai_summary_generated_at: string | null
  ai_state?: CommunicationAiState | null
  message_metadata: Record<string, unknown>
  local_state: LocalMessageState
  local_state_changed_at: string | null
  local_state_reason: string | null
}

export type CommunicationMessageDetailResponse = {
  message: CommunicationMessageDetailItem
  attachments: CommunicationAttachment[]
}

export type WorkflowStateCountItem = {
  state: string
  count: number
}

export type WorkflowStateCountsResponse = {
  counts: WorkflowStateCountItem[]
}

export type WorkflowStateTransitionRequest = {
  workflow_state: WorkflowState
}

export type LocalMessageStateResponse = {
  message_id: string
  local_state: LocalMessageState
  provider_deleted?: boolean
}

export type MailSyncSettings = {
  account_id: string
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
  updated_at: string
}

export type MailSyncSettingsUpdate = {
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
}

export type MailSyncStatus = {
  account_id: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  last_started_at: string | null
  last_updated_at: string | null
  last_completed_at: string | null
  next_run_at: string | null
  last_error_code: string | null
  last_error_message: string | null
  consecutive_failures: number
  last_fetched_messages: number
  last_projected_messages: number
  last_upserted_personas: number
  last_upserted_organizations: number
}

export type MailSyncStatusListResponse = {
  items: MailSyncStatus[]
}

export type MailSyncRunResponse = {
  run_id: string
  account_id: string
  trigger: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  fetched_messages: number
  projected_messages: number
  upserted_personas: number
  upserted_organizations: number
  checkpoint_before_present: boolean
  checkpoint_after_present: boolean
  checkpoint_saved: boolean
  failure_reason: { code: string; message: string } | null
  started_at: string
  completed_at: string | null
  next_run_at: string | null
}

export type CommunicationThread = {
  thread_id: string
  account_id: string
  subject: string
  message_count: number
  participant_count: number
  first_message_at: string | null
  last_message_at: string | null
  last_activity_at: string
  has_open_action: boolean
  has_attachments: boolean
  dominant_workflow_state: string
}

export type CommunicationThreadSummary = Pick<
  CommunicationThread,
  | 'thread_id'
  | 'subject'
  | 'message_count'
  | 'participant_count'
  | 'last_activity_at'
  | 'has_open_action'
  | 'has_attachments'
  | 'dominant_workflow_state'
>

export type ThreadMessage = {
  message_id: string
  provider_record_id: string
  account_id: string
  subject: string
  sender: string
  sender_display_name: string | null
  body_text: string
  occurred_at: string | null
  projected_at: string
  workflow_state: string
  importance_score: number | null
  ai_category: string | null
  ai_summary: string | null
  delivery_state: string
  attachment_count: number
  attachments: CommunicationAttachment[]
}

export type ThreadListResponse = {
  items: CommunicationThread[]
  next_cursor: string | null
  has_more: boolean
}
export type ThreadMessagesResponse = { items: ThreadMessage[] }

export type ProviderCall = {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at: string | null
  ended_at: string | null
  transcription_policy_id: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type ProviderCallListResponse = {
  items: ProviderCall[]
}

export type ProviderCallTranscript = {
  transcript_id: string
  call_id: string
  account_id: string
  provider_chat_id: string
  transcript_status: string
  stt_provider: string
  source_audio_ref: string | null
  language_code: string | null
  transcript_text: string
  segments: unknown
  provenance: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type ProviderCallTranscriptResponse = {
  transcript: ProviderCallTranscript | null
}

export type MessageAnalyzeResponse = {
  message_id: string
  analyzed: boolean
  category: string | null
  summary: string | null
  summary_contract: {
    key_points: string[]
    action_items: string[]
    risks: string[]
    deadlines: string[]
    event_candidates: CommunicationKnowledgeCandidate[]
    persona_candidates: CommunicationKnowledgeCandidate[]
    organization_candidates: CommunicationKnowledgeCandidate[]
    document_candidates: CommunicationKnowledgeCandidate[]
    agreement_candidates: CommunicationKnowledgeCandidate[]
  }
  importance_score: number | null
  workflow_state: string
  source: string
  confidence: number | null
  evidence: string[]
}

export type CommunicationKnowledgeCandidate = {
  title: string
  evidence: string
}

export type WorkflowActionKind =
  | 'reply'
  | 'create_task'
  | 'create_note'
  | 'create_document'
  | 'create_event'
  | 'link_document'
  | 'create_persona'
  | 'archive'

export type WorkflowActionSource = {
  kind: 'communication_message'
  id: string
}

export type WorkflowActionRequest = {
  command_id: string
  action: WorkflowActionKind
  source?: WorkflowActionSource
  input?: {
    title?: string
    body?: string
    email?: string
    display_name?: string
    starts_at?: string
    ends_at?: string
    due_at?: string
    document_id?: string
  }
}

export type WorkflowActionResponse = {
  command_id: string
  event_id: string
  action: WorkflowActionKind
  status: 'created' | 'updated' | 'linked' | 'opened' | 'archived' | 'noop'
  target: {
    kind: 'compose' | 'message' | 'task' | 'document' | 'calendar_event' | 'persona'
    id: string | null
  }
  provenance: {
    source_kind?: string
    source_id?: string
    confidence: number | null
    evidence: string[]
  }
}

export type MailboxHealth = {
  total_messages: number
  unread: number
  needs_action: number
  waiting: number
  done: number
  archived: number
  spam: number
  important: number
  with_attachments: number
  average_importance: number
  oldest_message_days: number | null
}

export type SenderStats = {
  sender: string
  message_count: number
  avg_importance: number
  last_message_days: number | null
}

export type SenderStatsListResponse = {
  items: SenderStats[]
  next_cursor: string | null
  has_more: boolean
}

export type WorkflowStateTransitionResponse = {
  message_id: string
  workflow_state: string
  previous_state: string
}

export type MessageExplainResponse = {
  reasons: string[]
}

export type SmartCcResponse = {
  suggestions: string[]
}

export type MessagePinToggleResponse = {
  message_id: string
  pinned: boolean
}

export type ConversationPinToggleResponse = {
  conversation_id: string
  provider_chat_id: string
  channel_kind: string
  action: string
  status: string
  command_id: string
  provider: string
  active: boolean
}

export type CommunicationProviderMessageCommandResponse = {
  message_id: string
  raw_record_id: string
  conversation_id: string
  provider_chat_id: string
  provider_message_id: string | null
  channel_kind: string
  status: string
  command_id: string
  provider: string
}

export type MessageImportantToggleResponse = {
  message_id: string
  important: boolean
}

export type MessageExportResponse = {
  content_type: string
  content: string
  filename: string
}
export type MessageExportFormat = 'md' | 'eml' | 'json'

export type MessageAuthResult = {
  result: string
  domain?: string | null
  ip?: string | null
  selector?: string | null
  policy?: string | null
}

export type MessageAuthCheckResponse = {
  auth: {
    spf: MessageAuthResult | null
    dkim: MessageAuthResult | null
    dmarc: MessageAuthResult | null
    raw_headers: string[]
  }
  risk: {
    has_spf: boolean
    spf_pass: boolean
    has_dkim: boolean
    dkim_pass: boolean
    has_dmarc: boolean
    dmarc_pass: boolean
    is_spoofed: boolean
    risk_summary: string
  }
}

export type SignatureDetection = {
  has_signature: boolean
  signature_type: string | null
  signer_info: string | null
  is_valid: boolean | null
  cert_expiry_warning: string | null
}

export type LanguageDetection = {
  language: string
  confidence: number
  script: string | null
}

export type TranslationResponse = {
  translated: boolean
  text?: string
  target?: string
  model?: string
  reason?: string
}

export type AiReplyResponse = {
  subject?: string
  body?: string
  tone?: string
  language?: string
  generated?: boolean
  reason?: string
}

export type AiReplyVariantsRequest = {
  languages?: string[]
  tones?: string[]
}

export type AiReplyVariantsResponse = {
  variants: AiReplyResponse[]
}

export type ExtractedTask = {
  title: string
  due_date: string | null
  assignee: string | null
  priority: string | null
  source: string
}

export type ExtractedNote = {
  title: string
  content: string
  tags: string[]
  source: string
}

export type ExtractTasksResponse = { tasks: ExtractedTask[] }
export type ExtractNotesResponse = { notes: ExtractedNote[] }

export type CommunicationSearchResponse = {
  results: { object_id: string; object_kind: string; title: string }[]
}

export type SubscriptionSource = {
  sender: string
  message_count: number
  first_seen: string
  last_seen: string
  is_newsletter: boolean
  has_unsubscribe: boolean
}

export type SubscriptionListResponse = {
  items: SubscriptionSource[]
  next_cursor: string | null
  has_more: boolean
}

export type DuplicateAttachmentGroup = {
  sha256: string
  filenames: string[]
  message_ids: string[]
  count: number
}

export type CommunicationMessageInsight = {
  messageId: string
  explain: MessageExplainResponse | null
  smartCc: SmartCcResponse | null
  auth: MessageAuthCheckResponse | null
  signature: SignatureDetection | null
  language: LanguageDetection | null
  aiReply: AiReplyResponse | null
  tasks: ExtractedTask[]
  notes: ExtractedNote[]
  translation: TranslationResponse | null
}

export type CommunicationResourceSnapshot = {
  subscriptions: SubscriptionSource[]
  duplicates: DuplicateAttachmentGroup[]
  invoices: unknown[]
  legalDocuments: unknown[]
  certificates: MailCertificate[]
  expiringCertificates: MailCertificate[]
  personas: unknown[]
  templates: unknown[]
  blockers: unknown[]
}

export type CommunicationResourceSummary = {
  subscriptions: number
  duplicates: number
  invoices: number
  legalDocuments: number
  certificates: number
  expiringCertificates: number
  personas: number
  templates: number
  blockers: number
}

export type { CommunicationPersona } from './personas'
export type {
  MailCertificate,
  MailCertificateCreateRequest,
  MailCertificateListResponse
} from './certificates'
export type {
  BulkMessageAction,
  BulkMessageActionRequest,
  BulkMessageActionResponse,
  DraftDeleteResponse,
  DraftListResponse,
  DraftUpsertRequest,
  CommunicationDraft,
  CommunicationOutboxItem,
  CommunicationOutboxStatus,
  OutboxListResponse,
  RedirectMessageRequest,
  SendCommunicationRequest,
  SendCommunicationResponse
} from './mailOperations'
export type {
  CommunicationTemplate,
  RichTemplateDeleteResponse,
  RichTemplateMailMergePreviewRequest,
  RichTemplateMailMergePreviewResponse,
  RichTemplateRenderRequest,
  RichTemplateRenderResponse,
  RichTemplateUpsertRequest,
  RichTemplateUpsertResponse
} from './templates'

export type CommunicationArchitectureBlocker = {
  section: string
  feature: string
  reason: string
  resolution: string
}

// --- UI-specific types (from Svelte state/services) ---

export type ComposeMode = 'compose' | 'reply' | 'forward'

export type ComposeFormModel = {
  mode: ComposeMode
  draftId: string
  accountId: string
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
  bodyHtml: string | null
  bodyFormat: 'plain' | 'html'
  scheduledSendAt: string
  undoSendSeconds: number | null
  inReplyTo: string | null
}

export type CommunicationAccountOption = {
  account_id: string
  label: string
  provider_kind: string
  email: string
  can_send: boolean
  send_unavailable_reason: string
}

export type SendCapability = {
  canSend: boolean
  reason: string
}

export type RenderedMessageContent = {
  html: string
  isHtml: boolean
}
export type OriginalMailSrcdocOptions = {
  messageId: string
  bodyHtml: string
}

export type NavigatorMode = 'threads' | 'contacts'
export type InspectorMode = 'context' | 'contact' | 'organization' | null
export type MessageContextTab = 'message' | 'attachments' | 'headers' | 'related' | 'timeline'
export type ProjectItem = {
  project_id: string
  name: string
}

export type TaskItem = {
  task_id: string
  title: string
}
export type CommunicationSectionId =
  | 'unified'
  | 'inbox'
  | 'waiting'
  | 'needs_reply'
  | 'done'
  | 'archived'
  | 'mentions'
  | 'mail'
  | 'telegram'
  | 'whatsapp'
  | 'calls'
  | 'meetings'

export type CommunicationListMessage = {
  messageId: string
  subject: string
  sender: string
  senderDisplayName: string | null
  preview: string
  occurredAt: string | null
  projectedAt: string
  workflowState: WorkflowState
  importanceScore: number | null
  aiCategory: string | null
  attachmentCount: number
  conversationId: string | null
}
