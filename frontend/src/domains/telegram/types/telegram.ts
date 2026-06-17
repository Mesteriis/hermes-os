// --- Provider kinds ---
export type TelegramProviderKind = 'telegram_user' | 'telegram_bot'

// --- Account lifecycle ---
export type TelegramAccountLifecycleState = 'active' | 'logged_out' | 'removed' | string

export type TelegramAccount = {
  account_id: string
  provider_kind: TelegramProviderKind
  display_name: string
  external_account_id: string
  runtime: string
  lifecycle_state: TelegramAccountLifecycleState
  transcription_enabled: boolean
  tdlib_data_path: string | null
  created_at: string
  updated_at: string
}

export type TelegramAccountListResponse = {
  items: TelegramAccount[]
}

export type TelegramAccountSetupResponse = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime: string
  transcription_enabled: boolean
  credential_bindings: { secret_purpose: string; secret_ref: string; secret_kind: string; store_kind: string }[]
}

export type TelegramAccountLifecycleResponse = {
  account: TelegramAccount
  stopped_runtime_actor: boolean
}

// --- Capabilities ---
export type TelegramCapabilityState = 'available' | 'blocked' | 'degraded' | 'unsupported'
export type TelegramActionClass = 'read' | 'local_write' | 'provider_write' | 'destructive' | 'export' | 'secret_access' | 'automation'

export type TelegramOperationCapability = {
  operation: string
  category: string
  status: TelegramCapabilityState
  action_class: TelegramActionClass
  reason: string
  confirmation_required: boolean
  closure_gate: boolean
}

export type TelegramCapabilityAccountScope = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime_kind: string
  lifecycle_state: TelegramAccountLifecycleState
}

export type TelegramCapabilitiesResponse = {
  version: string
  runtime_mode: string
  account_scope?: TelegramCapabilityAccountScope | null
  telegram_app_credentials_configured: boolean
  tdjson_runtime_available: boolean
  qr_login_ready: boolean
  bot_runtime_available: boolean
  capabilities: TelegramOperationCapability[]
  unsupported_features: string[]
}

// --- Runtime ---
export type TelegramRuntimeStatus = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime_kind: string
  status: 'stopped' | 'running' | 'blocked' | 'degraded' | 'error' | string
  fixture_runtime: boolean
  tdjson_path: string | null
  tdjson_runtime_available: boolean
  tdjson_probe_error: string | null
  telegram_api_id_configured: boolean
  telegram_api_hash_configured: boolean
  telegram_app_credentials_configured: boolean
  live_send_available: boolean
  runtime_blockers: string[]
  last_error: string | null
  last_sync_scope?: string | null
  last_sync_status?: string | null
  last_synced_count?: number | null
  last_sync_has_more?: boolean | null
  last_sync_provider_chat_id?: string | null
  last_command_id?: string | null
  last_command_status?: string | null
  last_command_kind?: string | null
  last_command_provider_chat_id?: string | null
  last_command_message_id?: string | null
  last_command_telegram_chat_id?: string | null
  updated_at: string
}

// --- Chats ---
export type TelegramChatKind = 'private' | 'group' | 'channel' | 'bot'
export type TelegramChatSyncState = 'fixture' | 'syncing' | 'synced' | 'degraded' | 'error'

export type TelegramChat = {
  telegram_chat_id: string
  account_id: string
  provider_chat_id: string
  chat_kind: TelegramChatKind
  title: string
  username: string | null
  sync_state: TelegramChatSyncState
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type TelegramChatListResponse = {
  items: TelegramChat[]
}

export type TelegramChatDetailResponse = {
  item: TelegramChat
}

export type TelegramChatGroupFilterListResponse = {
  items: TelegramChatGroupFilter[]
}

export type {
  TelegramChatMember,
  TelegramChatMemberListResponse,
  TelegramChatMembersSyncResponse
} from './telegramMembers'

export type {
  TelegramChatActionRequest,
  TelegramChatActionResponse,
  TelegramChatFolderReassignRequest,
  TelegramChatFolderReassignResponse,
  TelegramChatLifecycleCommandResponse
} from './telegramChatActions'

// --- Messages ---
export type TelegramMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string | null
  chat_title: string
  sender: string
  sender_display_name: string | null
  text: string
  occurred_at: string | null
  projected_at: string
  channel_kind: TelegramProviderKind
  delivery_state: string
  metadata: Record<string, unknown>
}

export type TelegramMessageListResponse = {
  items: TelegramMessage[]
}

export type TelegramMessageSearchResponse = {
  query: string
  items: TelegramMessage[]
  total: number
}
export type TelegramChatSearchResponse = {
  query: string
  items: TelegramChat[]
  total: number
}
export type TelegramMediaItem = {
  message_id: string
  provider_message_id: string
  provider_chat_id: string
  file_name: string
  kind: string
  mime_type: string | null
  size_bytes: number | null
  occurred_at: string | null
  download_state: string
  tdlib_file_id: number | null
  provider_attachment_id: string | null
  local_path: string | null
  expected_size_bytes?: number | null
  downloaded_size_bytes?: number | null
  is_downloading_active?: boolean | null
  is_downloading_completed?: boolean | null
  last_error?: string | null
}
export type TelegramMediaSearchResponse = {
  query?: string | null
  source?: 'projection' | 'provider_refresh' | string
  provider_search_attempted?: boolean
  provider_search_error?: string | null
  items: TelegramMediaItem[]
}
// --- Sync ---
export type TelegramChatSyncRequest = {
  account_id: string
  limit?: number
}

export type TelegramChatSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  items: TelegramChat[]
}

export type TelegramHistorySyncRequest = {
  account_id: string
  provider_chat_id: string
  from_message_id?: number
  mode?: 'latest' | 'older' | 'full'
  limit?: number
}

export type TelegramHistorySyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  next_from_message_id: number | null
  items: TelegramMessage[]
}

// --- QR login ---
export type TelegramQrLoginStatus =
  | 'waiting_qr_scan'
  | 'waiting_password'
  | 'ready'
  | 'expired'
  | 'failed'
  | 'runtime_unavailable'

export type TelegramQrLoginStatusResponse = {
  setup_id: string
  account_id: string
  status: TelegramQrLoginStatus
  qr_link: string | null
  qr_svg: string | null
  telegram_user_id: string | null
  telegram_username: string | null
  suggested_account_id: string | null
  suggested_display_name: string | null
  suggested_external_account_id: string | null
  expires_at: string | null
  poll_after_ms: number
  message: string | null
}

// --- UI types ---
export type TelegramChatFilter = 'all' | 'unread' | 'mentions' | 'pinned' | 'projects' | 'bots' | 'archived'
export type TelegramThreadTab = 'messages' | 'files' | 'links' | 'voice' | 'topics' | 'pinned' | 'timeline'
export type TelegramRailTab = 'context' | 'members' | 'about'

export type TelegramChatFilterCount = {
  filter: TelegramChatFilter
  count: number
}

export type TelegramChatGroupFilter = {
  id: string
  label: string
  source: 'local' | 'telegram'
  count: number
  icon: string
  provider_folder_id?: number | null
}

export type TelegramAttachmentHint = {
  id: string
  kind: 'document' | 'photo' | 'video' | 'audio' | 'voice' | 'sticker' | 'animation' | 'video_note' | 'file'
  fileName: string
  mimeType: string | null
  sizeBytes: number | null
  tdlibFileId: number | null
  providerAttachmentId: string
  downloadState: 'remote' | 'downloading' | 'downloaded' | 'failed' | 'unknown'
  localPath: string | null
  expectedSizeBytes?: number | null
  downloadedSizeBytes?: number | null
  isDownloadingActive?: boolean | null
  isDownloadingCompleted?: boolean | null
  lastError?: string | null
  messageId: string
  providerMessageId?: string | null
}

// --- Additional request/response types for API ---
export type TelegramRuntimeStartRequest = {
  account_id: string
}

export type TelegramRuntimeStopRequest = {
  account_id: string
}

export type TelegramRuntimeRestartRequest = {
  account_id: string
}

export type TelegramQrLoginStartRequest = {
  account_id: string
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  transcription_enabled: boolean
}

export type TelegramQrLoginPasswordRequest = {
  password: string
}

export type TelegramMediaDownloadRequest = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  tdlib_file_id: number
  provider_attachment_id?: string
  filename?: string
  content_type?: string
  priority?: number
}

export type TelegramMediaDownloadResponse = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  runtime_kind: string
  status: string
  tdlib_file_id: number
  local_path: string | null
  size_bytes: number | null
  expected_size_bytes: number | null
  downloaded_size_bytes: number | null
  is_downloading_active: boolean
  is_downloading_completed: boolean
  attachment_id: string | null
  blob_id: string | null
  scan_status: string | null
}

export type TelegramManualSendResponse = {
  message_id: string
  provider_message_id: string
  provider_chat_id: string
  status: string
}

export type TelegramSendDryRunResponse = {
  allowed: boolean
  reason: string | null
}

export type TelegramMessageIngestResponse = {
  raw_record_id: string
  message_id: string
}

export type TelegramCallListResponse = {
  items: TelegramCall[]
}

export type TelegramCall = {
  call_id: string
  account_id: string
  provider_chat_id: string
  status: string
  occurred_at: string | null
}

export type TelegramCallTranscript = {
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
  provenance: unknown
  created_at: string
  updated_at: string
}

export type TelegramCallTranscriptResponse = {
  transcript: TelegramCallTranscript | null
}

// --- Lifecycle types (ADR-0091) ---

export type TelegramTombstoneReasonClass =
  | 'deleted_by_owner'
  | 'deleted_by_counterparty'
  | 'deleted_by_provider'
  | 'moderation_removed'
  | 'account_removed'
  | 'retention_policy'
  | 'unknown'

export type TelegramTombstoneActorClass = 'owner' | 'provider' | 'automation' | 'system' | 'unknown'

export type TelegramCommandKind =
  | 'send_text'
  | 'send_media'
  | 'edit'
  | 'delete'
  | 'restore_visibility'
  | 'mark_read'
  | 'mark_unread'
  | 'pin'
  | 'unpin'
  | 'archive'
  | 'unarchive'
  | 'mute'
  | 'unmute'
  | 'folder_add'
  | 'folder_remove'
  | 'react'
  | 'unreact'
  | 'reply'
  | 'forward'
  | 'join'
  | 'leave'
  | 'topic_create'
  | 'topic_close'
  | 'topic_reopen'
  | 'admin_action'

export type TelegramCommandStatus = 'queued' | 'executing' | 'completed' | 'failed' | 'retrying' | 'cancelled' | 'dead_letter'
export type TelegramConfirmationDecision = 'pending' | 'confirmed' | 'rejected' | 'not_required'

export type TelegramLifecycleResponse = {
  operation: string
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  status: string
  timestamp: string
  version_number: number | null
  tombstone_id: string | null
}

export type {
  TelegramDeleteRequest,
  TelegramEditRequest,
  TelegramForwardRequest,
  TelegramPinRequest,
  TelegramReplyRequest,
  TelegramRestoreVisibilityRequest,
} from './telegramLifecycleRequests'

export type TelegramMessageVersion = {
  version_id: string
  message_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string
  version_number: number
  body_text: string | null
  edit_timestamp: string
  source_event: string | null
  raw_diff_payload: Record<string, unknown>
  provenance: Record<string, unknown>
  created_at: string
}

export type TelegramMessageVersionListResponse = {
  message_id: string
  versions: TelegramMessageVersion[]
}

export type TelegramMessageTombstone = {
  tombstone_id: string
  message_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string
  reason_class: TelegramTombstoneReasonClass
  actor_class: TelegramTombstoneActorClass
  observed_at: string
  source_event: string | null
  is_provider_delete: boolean
  is_local_visible: boolean
  metadata: Record<string, unknown>
  provenance: Record<string, unknown>
  created_at: string
}

export type TelegramMessageTombstoneListResponse = {
  message_id: string
  tombstones: TelegramMessageTombstone[]
}

export type TelegramReplyRef = {
  reply_ref_id: string
  source_message_id: string
  target_message_id: string
  account_id: string
  provider_chat_id: string
  source_provider_id: string
  target_provider_id: string
  reply_depth: number
  is_topic_reply: boolean
  topic_id: string | null
  source_message_summary: TelegramMessageReferenceSummary | null
  target_message_summary: TelegramMessageReferenceSummary | null
  metadata: Record<string, unknown>
  provenance: Record<string, unknown>
  created_at: string
}

export type TelegramMessageReferenceSummary = {
  message_id: string
  provider_message_id: string
  provider_chat_id: string | null
  chat_title: string
  sender: string
  sender_display_name: string | null
  text: string
  occurred_at: string | null
}

export type TelegramForwardRef = {
  forward_ref_id: string
  source_message_id: string
  account_id: string
  provider_chat_id: string
  source_provider_id: string
  forward_origin_chat_id: string | null
  forward_origin_message_id: string | null
  forward_origin_sender_id: string | null
  forward_origin_sender_name: string | null
  forward_date: string | null
  forward_depth: number
  source_message_summary: TelegramMessageReferenceSummary | null
  metadata: Record<string, unknown>
  provenance: Record<string, unknown>
  created_at: string
}

export type TelegramReplyChainResponse = {
  message_id: string
  replies: TelegramReplyRef[]
  reply_to: TelegramReplyRef[]
}

export type TelegramForwardChainResponse = {
  message_id: string
  forwards: TelegramForwardRef[]
}

export type TelegramProviderWriteCommand = {
  command_id: string
  account_id: string
  command_kind: TelegramCommandKind
  idempotency_key: string
  provider_chat_id: string
  provider_message_id: string | null
  target_ref: Record<string, unknown>
  payload: Record<string, unknown>
  capability_state: TelegramCapabilityState
  action_class: TelegramActionClass
  confirmation_decision: TelegramConfirmationDecision
  status: TelegramCommandStatus
  retry_count: number
  max_retries: number
  last_error: string | null
  result_payload: Record<string, unknown>
  audit_metadata: Record<string, unknown>
  actor_id: string
  happened_at: string
  next_attempt_at: string | null
  last_attempt_at: string | null
  locked_at: string | null
  locked_by: string | null
  provider_observed_at: string | null
  provider_state: Record<string, unknown>
  reconciliation_status: 'not_observed' | 'awaiting_provider' | 'observed' | 'mismatch' | 'not_required' | string
  reconciled_at: string | null
  dead_lettered_at: string | null
  completed_at: string | null
  created_at: string
  updated_at: string
}

export type TelegramCommandListResponse = {
  items: TelegramProviderWriteCommand[]
}

// --- Reaction types (ADR-0091) ---

export type TelegramReaction = {
  reaction_id: string
  message_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string
  sender_id: string
  sender_display_name: string | null
  reaction_emoji: string
  is_active: boolean
  observed_at: string
  source_event: string | null
  provider_actor_id: string | null
  metadata: Record<string, unknown>
  provenance: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type TelegramReactionRequest = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reaction_emoji: string
  sender_id: string
  sender_display_name?: string | null
  command_id?: string
}

export type TelegramReactionResponse = {
  reaction_id: string
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reaction_emoji: string
  is_active: boolean
  status: string
  timestamp: string
}

export type TelegramReactionGroup = {
  reaction_emoji: string
  count: number
  senders: string[]
}

export type TelegramReactionSummary = {
  message_id: string
  total_reactions: number
  active_reactions: number
  reactions: TelegramReactionGroup[]
}

export type TelegramReactionListResponse = {
  message_id: string
  reactions: TelegramReaction[]
  summary: TelegramReactionSummary
}

export type { TelegramTopic, TelegramTopicListResponse } from './telegramTopics'
export const TELEGRAM_REACTION_PALETTE = ['👍', '👎', '❤️', '🔥', '🥰', '👏', '😁', '🤔', '🤯', '😱', '🤬', '😢', '🎉', '🤩', '🤮', '💩']
