// --- Provider ---
export type WhatsappWebProviderKind = 'whatsapp_web'
export type WhatsappProviderShape = 'whatsapp_web_companion'

// --- Capabilities ---
export type WhatsappCapabilityStatus = {
  capability: string
  category: string
  status: 'available' | 'blocked' | string
  action_class: 'read' | 'local_write' | 'provider_write' | 'destructive' | 'secret_access' | string
  confirmation_required: boolean
  closure_gate: boolean
  reason: string
}

export type WhatsappProviderShapeStatus = {
  provider_shape: string
  status: 'available' | 'blocked' | 'degraded' | 'planned' | 'unsupported' | string
  reason: string
}

export type WhatsappCapabilityAccountScope = {
  account_id: string
  provider_kind: string
  provider_shape: string
  runtime_kind: string
  lifecycle_state: string
  live_runtime_available: boolean
  live_send_available: boolean
  media_download_available: boolean
  media_upload_available: boolean
}

export type WhatsappAccountSummary = {
  account_id: string
  provider_kind: WhatsappWebProviderKind
  provider_shape: WhatsappProviderShape | string | null
  display_name: string
  external_account_id: string
  runtime: string | null
  lifecycle_state: string | null
  created_at: string
  updated_at: string
}

export type WhatsappAccountListResponse = {
  items: WhatsappAccountSummary[]
}

export type WhatsappCapabilitiesResponse = {
  version: string
  runtime_mode: string
  provider_shapes: WhatsappProviderShapeStatus[]
  account_scope: WhatsappCapabilityAccountScope | null
  capabilities: WhatsappCapabilityStatus[]
  planned_features: string[]
  unsupported_features: string[]
}

export type WhatsAppRuntimeStatus = {
  account_id: string
  provider_kind: string
  provider_shape: string
  runtime_kind: string
  status: string
  live_runtime_available: boolean
  live_send_available: boolean
  media_download_available: boolean
  media_upload_available: boolean
  session_restore_available: boolean
  session_secret_ref: string | null
  runtime_blockers: string[]
  last_error: string | null
  updated_at: string
}

export type WhatsAppRuntimeHealth = {
  account_id: string
  provider_shape: string
  runtime_kind: string
  status: string
  healthy: boolean
  checks: Record<string, unknown>
  checked_at: string
}

export type WhatsAppWebCompanionBridgeRoutes = {
  authorized_session_path: string
  runtime_event_path: string
  sync_lifecycle_path: string
  message_paths: string[]
  conversation_paths: string[]
  media_paths: string[]
}

export type WhatsAppWebCompanionCommandChannel = {
  kind: string
  claim_path: string
  failure_path: string
  completion_rule: string
}

export type WhatsAppWebCompanionExtractorContract = {
  state: string
  relay_command?: string
  relay_command_policy?: Record<string, unknown>
  initialization_script: string
  script_scope: string
  origin_guard: string
  navigation_guard: string
  relay_channel: string
  runtime_bridge_dispatch: string
  allowed_observations: string[]
  forbidden_reads: string[]
  next_gate: string
}

export type WhatsAppWebCompanionSecretPolicy = {
  session_material: string
  cookies: string
  browser_profile_secrets: string
  qr_pair_code_artifacts: string
  message_bodies: string
  media_bytes: string
  postgres_storage: string
}

export type WhatsAppWebCompanionManifest = {
  account_id: string
  provider_shape: 'whatsapp_web_companion'
  runtime_kind: 'webview_companion'
  driver_id: string
  window_label: string
  target_url: string
  opened_window: boolean
  reused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  tauri_ipc_available_to_companion_window: boolean
  event_flow: string
  event_extractor: WhatsAppWebCompanionExtractorContract
  bridge_routes: WhatsAppWebCompanionBridgeRoutes
  command_channel: WhatsAppWebCompanionCommandChannel
  secret_policy: WhatsAppWebCompanionSecretPolicy
  remaining_blockers: string[]
}

export type WhatsAppWebCompanionRelayObservationRequest = {
  account_id: string
  event_family: string
  provider_event_id: string
  observed_at: string
  metadata?: Record<string, unknown>
}

export type WhatsAppWebCompanionRelayObservationReceipt = {
  account_id: string
  provider_shape: 'whatsapp_web_companion'
  runtime_kind: 'webview_companion'
  window_label: string
  event_family: string
  provider_event_id: string
  observed_at: string
  target_runtime_bridge_path: string
  typed_runtime_bridge_path: string
  relay_state: string
  relay_channel: string
  sanitized_metadata: Record<string, unknown>
  runtime_event_kind: string
  import_batch_id: string
  runtime_bridge_http_status: number
  event_flow: string
  completion_rule: string
}

export type WhatsAppRuntimeRemoveResponse = {
  account_id: string
  provider_kind: string
  removed: boolean
  unbound_secret_refs: string[]
  removed_at: string
}

export type WhatsAppProviderCommand = {
  command_id: string
  account_id: string
  command_kind: string
  idempotency_key: string
  provider_chat_id: string
  provider_message_id: string | null
  capability_state: string
  action_class: string
  confirmation_decision: string
  status: string
  retry_count: number
  max_retries: number
  last_error: string | null
  result_payload: Record<string, unknown>
  audit_metadata: Record<string, unknown>
  provider_state: Record<string, unknown>
  reconciliation_status: string
  next_attempt_at: string | null
  last_attempt_at: string | null
  provider_observed_at: string | null
  reconciled_at: string | null
  dead_lettered_at: string | null
  completed_at: string | null
  created_at: string
  updated_at: string
}

export type WhatsAppProviderCommandListResponse = {
  items: WhatsAppProviderCommand[]
}

export type WhatsAppChatSyncItem = {
  conversation_id: string
  account_id: string
  channel_kind: string
  provider_chat_id: string
  title: string
  chat_kind: string | null
  is_archived: boolean
  is_pinned: boolean
  is_muted: boolean
  is_unread: boolean
  unread_count: number | null
  participant_count: number | null
  community_parent_chat_id: string | null
  community_parent_title: string | null
  invite_link: string | null
  is_community_root: boolean
  is_broadcast: boolean
  is_newsletter: boolean
  avatar_metadata: Record<string, unknown>
  provider_labels: string[]
}

export type WhatsAppChatSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  items: WhatsAppChatSyncItem[]
}

export type WhatsAppMembersSyncItem = {
  participant_id: string
  conversation_id: string
  account_id: string
  provider_chat_id: string
  provider_member_id: string
  provider_identity_id: string | null
  sender_display_name: string | null
  role: string
  status: string | null
  identity_kind: string | null
  address: string | null
  is_admin: boolean
  is_owner: boolean
  participant_metadata: Record<string, unknown>
  identity_metadata: Record<string, unknown>
}

export type WhatsAppMembersSyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppMembersSyncItem[]
}

export type WhatsAppPresenceSyncItem = {
  identity_id: string
  account_id: string
  channel_kind: string
  provider_chat_id: string | null
  provider_identity_id: string
  identity_kind: string
  display_name: string | null
  address: string | null
  presence_state: string
  last_seen_at: string | null
  observed_at: string | null
  identity_metadata: Record<string, unknown>
}

export type WhatsAppPresenceSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppPresenceSyncItem[]
}

export type WhatsAppCallSyncItem = {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at: string | null
  ended_at: string | null
  observed_at: string | null
  metadata: Record<string, unknown>
}

export type WhatsAppCallsSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppCallSyncItem[]
}

export type WhatsAppContactSyncItem = {
  identity_id: string
  account_id: string
  channel_kind: string
  provider_identity_id: string
  identity_kind: string
  display_name: string | null
  address: string | null
  push_name: string | null
  business_profile: Record<string, unknown>
  profile_photo_ref: Record<string, unknown>
  display_name_history: string[]
  identity_metadata: Record<string, unknown>
  whatsapp_trace_metadata: Record<string, unknown>
  phone_trace_metadata: Record<string, unknown>
}

export type WhatsAppContactsSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppContactSyncItem[]
}

export type WhatsAppMediaSyncItem = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  account_id: string
  channel_kind: WhatsappWebProviderKind | string
  provider_chat_id: string | null
  provider_message_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  scan_status: string
  storage_kind: string
  storage_path: string
  message_subject: string
  sender: string
  sender_display_name: string | null
  occurred_at: string | null
  created_at: string
}

export type WhatsAppMediaSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  content_type: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppMediaSyncItem[]
}

export type WhatsAppStatusSyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsappWebMessage[]
}

export type WhatsAppLifecycleResponse = {
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

// --- Sessions ---
export type WhatsappWebSession = {
  session_id: string
  account_id: string
  device_name: string
  companion_runtime: string
  link_state: string
  local_state_path: string
  last_sync_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type WhatsappWebSessionListResponse = {
  items: WhatsappWebSession[]
}

// --- Messages ---
export type WhatsappWebMessage = {
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
  channel_kind: WhatsappWebProviderKind
  delivery_state: string
  metadata: Record<string, unknown>
}

export type WhatsappWebMessageListResponse = {
  items: WhatsappWebMessage[]
}

export type WhatsappWebMessageSearchResponse = {
  query: string
  items: WhatsappWebMessage[]
  total: number
}

export type WhatsappWebMediaItem = {
  attachment_id?: string | null
  message_id: string
  provider_message_id: string
  provider_chat_id: string
  file_name: string
  kind: string
  mime_type: string | null
  size_bytes: number | null
  occurred_at: string | null
  download_state: string
  provider_attachment_id: string | null
  local_path: string | null
}

export type WhatsappWebMediaSearchResponse = {
  query: string | null
  source: string
  provider_search_attempted: boolean
  provider_search_error: string | null
  items: WhatsappWebMediaItem[]
}

// --- Account setup ---
