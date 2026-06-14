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
export type TelegramCapabilityStatus = {
  capability: string
  status: 'available' | 'blocked' | string
  closure_gate: boolean
  reason: string
}

export type TelegramCapabilitiesResponse = {
  version: string
  runtime_mode: string
  telegram_app_credentials_configured: boolean
  tdjson_runtime_available: boolean
  qr_login_ready: boolean
  capabilities: TelegramCapabilityStatus[]
  unsupported_features: string[]
}

// --- Runtime ---
export type TelegramRuntimeStatus = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime_kind: string
  status: 'stopped' | 'running' | 'blocked' | 'degraded' | 'error' | string
  fixture_runtime: boolean
  tdjson_runtime_available: boolean
  telegram_app_credentials_configured: boolean
  live_send_available: boolean
  last_error: string | null
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
export type TelegramThreadTab = 'messages' | 'files' | 'links' | 'topics' | 'pinned' | 'timeline'
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
}

export type TelegramAttachmentHint = {
  id: string
  kind: 'document' | 'photo' | 'video' | 'audio' | 'voice' | 'file'
  fileName: string
  mimeType: string | null
  sizeBytes: number | null
  tdlibFileId: number | null
  providerAttachmentId: string
  downloadState: 'remote' | 'downloading' | 'downloaded' | 'unknown'
  localPath: string | null
  messageId: string
}

// --- Additional request/response types for API ---
export type TelegramRuntimeStartRequest = {
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
  items: { call_id: string; account_id: string; provider_chat_id: string; status: string; occurred_at: string | null }[]
}
