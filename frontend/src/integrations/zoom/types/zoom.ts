export type ZoomAuthShape = 'oauth_user' | 'server_to_server'

export interface ZoomAccountSetupRequest {
  account_id: string
  display_name: string
  external_account_id: string
  account_email?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomLiveAccountSetupRequest extends ZoomAccountSetupRequest {
  auth_shape?: ZoomAuthShape
  client_id: string
  token_secret_ref?: string | null
  client_secret_ref?: string | null
  webhook_secret_ref?: string | null
}

export interface ZoomAccount {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  auth_shape: string
  lifecycle_state: string
  runtime_kind: string
  account_email?: string | null
  config: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ZoomAccountSetupResponse {
  account: ZoomAccount
}

export interface ZoomAccountListResponse {
  items: ZoomAccount[]
}

export interface ZoomRuntimeStatus {
  account_id: string
  provider_kind: string
  runtime_kind: string
  status: string
  healthy: boolean
  auth_shape: string
  live_runtime_available: boolean
  recording_ingest_available: boolean
  transcript_ingest_available: boolean
  runtime_blockers: string[]
  last_error?: string | null
  checked_at: string
  metadata: Record<string, unknown>
}

export interface ZoomRuntimeStartRequest {
  account_id: string
  force?: boolean
}

export interface ZoomRuntimeStopRequest {
  account_id: string
  reason?: string | null
}

export interface ZoomRuntimeRemoveRequest {
  account_id: string
  reason?: string | null
}

export interface ZoomRuntimeRemoveResponse {
  account_id: string
  provider_kind: string
  removed: boolean
  removed_at: string
}

export interface ZoomCapabilityStatus {
  capability: string
  category: string
  status: string
  action_class: string
  confirmation_required: boolean
  reason: string
}

export interface ZoomCapabilitiesResponse {
  version: string
  runtime_mode: string
  capabilities: ZoomCapabilityStatus[]
  planned_features: string[]
  unsupported_features: string[]
}

export interface ZoomOAuthStartRequest {
  account_id: string
  display_name: string
  external_account_id: string
  account_email?: string | null
  client_id: string
  client_secret?: string | null
  client_secret_ref?: string | null
  webhook_secret_ref?: string | null
  redirect_uri: string
  app_return_url?: string | null
  scopes?: string[]
  authorization_endpoint?: string | null
  token_endpoint?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomOAuthStartResponse {
  setup_id: string
  authorization_url: string
  state: string
  redirect_uri: string
}

export interface ZoomOAuthCompleteRequest {
  setup_id: string
  state: string
  authorization_code: string
  external_account_id?: string | null
}

export interface ZoomServerToServerAuthorizeRequest {
  account_id: string
  client_id: string
  client_secret?: string | null
  client_secret_ref?: string | null
  zoom_account_id?: string | null
  token_endpoint?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomAuthorizationResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  lifecycle_state: string
  runtime_kind: string
  token_secret_ref: string
  client_secret_ref?: string | null
  secret_kind: string
  store_kind: string
  authorized_at: string
}

export interface ZoomTokenRefreshRequest {
  account_id: string
  force?: boolean
  refresh_expiring_within_seconds?: number | null
}

export interface ZoomTokenRefreshResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  token_secret_ref: string
  refreshed: boolean
  refresh_strategy: string
  status: string
  expires_at: string
  checked_at: string
  secret_kind: string
  store_kind: string
}

export interface ZoomTokenMaintenanceRequest {
  account_id?: string | null
  force?: boolean
  refresh_expiring_within_seconds?: number | null
}

export interface ZoomTokenMaintenanceItem {
  account_id: string
  provider_kind: string
  auth_shape: string
  status: string
  refreshed: boolean
  expires_at?: string | null
  error?: string | null
}

export interface ZoomTokenMaintenanceResult {
  checked_count: number
  refreshed_count: number
  skipped_count: number
  failed_count: number
  refresh_expiring_within_seconds: number
  checked_at: string
  items: ZoomTokenMaintenanceItem[]
}

export interface ZoomRecordingSyncRequest {
  account_id: string
  user_id?: string | null
  from: string
  to: string
  page_size?: number | null
  max_meetings?: number | null
  api_base_url?: string | null
}

export interface ZoomRecordingSyncFailure {
  meeting_id: string
  step: string
  error: string
}

export interface ZoomRecordingSyncResult {
  account_id: string
  user_id: string
  from: string
  to: string
  meetings_seen: number
  meetings_recorded: number
  recordings_recorded: number
  media_downloads_recorded: number
  transcripts_recorded: number
  failures: ZoomRecordingSyncFailure[]
}

export interface ZoomRecordingImportAuditItem {
  attachment_id: string
  account_id: string
  meeting_id?: string | null
  meeting_uuid?: string | null
  recording_id?: string | null
  filename?: string | null
  content_type: string
  size_bytes: number
  sha256: string
  source?: string | null
  scan_status: string
  scan_summary?: string | null
  storage_kind: string
  storage_path: string
  retention_mode: string
  retention_days: number
  expires_at?: string | null
  created_at: string
}

export interface ZoomRecordingImportAuditResponse {
  account_id: string
  items: ZoomRecordingImportAuditItem[]
}

export interface ZoomRecordingImportRemoveRequest {
  reason?: string | null
}

export interface ZoomRecordingImportRemoveResponse {
  account_id: string
  attachment_id: string
  blob_id: string
  recording_id?: string | null
  removed: boolean
  blob_metadata_removed: boolean
  blob_file_removed: boolean
  removed_at: string
}

export interface ZoomRetentionCleanupRequest {
  remove_recordings?: boolean
  remove_transcripts?: boolean
  limit?: number
}

export interface ZoomRetentionCleanupItem {
  evidence_kind: string
  entity_id: string
  call_id?: string | null
  meeting_id?: string | null
  recording_id?: string | null
  transcript_id?: string | null
  expires_at?: string | null
  removed_at: string
}

export interface ZoomRetentionCleanupResponse {
  account_id: string
  checked_at: string
  recordings_removed: number
  transcripts_removed: number
  items: ZoomRetentionCleanupItem[]
}

export interface ZoomAuditEventItem {
  position: number
  event_id: string
  event_type: string
  occurred_at: string
  subject_kind?: string | null
  subject_entity_id?: string | null
  correlation_id?: string | null
  source: Record<string, unknown>
  subject: Record<string, unknown>
  payload: Record<string, unknown>
  provenance: Record<string, unknown>
}

export interface ZoomAuditEventResponse {
  account_id: string
  items: ZoomAuditEventItem[]
}

export interface ZoomWebhookSubscriptionStatusRequest {
  account_id: string
  api_base_url?: string | null
}

export interface ZoomWebhookSubscriptionReconcileRequest {
  account_id: string
  endpoint_url: string
  subscription_name?: string | null
  event_types?: string[]
  api_base_url?: string | null
}

export interface ZoomWebhookSubscriptionRemoveRequest {
  account_id: string
  subscription_id?: string | null
  api_base_url?: string | null
}

export interface ZoomWebhookSubscription {
  subscription_id: string
  subscription_name: string
  endpoint_url: string
  event_types: string[]
}

export interface ZoomWebhookSubscriptionStatusResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  checked_at: string
  managed_subscription_id?: string | null
  subscriptions: ZoomWebhookSubscription[]
}

export interface ZoomWebhookSubscriptionReconcileResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  status: string
  checked_at: string
  subscription: ZoomWebhookSubscription
}

export interface ZoomWebhookSubscriptionRemoveResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  removed: boolean
  checked_at: string
  subscription_id?: string | null
}

export interface ZoomParticipantSnapshot {
  participant_id?: string | null
  display_name?: string | null
  email?: string | null
  joined_at?: string | null
  left_at?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomRecordingRef {
  recording_id: string
  recording_type?: string | null
  download_ref?: string | null
  file_extension?: string | null
  file_size_bytes?: number | null
  recorded_at?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomMeetingObservationRequest {
  observation_id?: string | null
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  topic?: string | null
  host_email?: string | null
  join_url?: string | null
  started_at?: string | null
  ended_at?: string | null
  duration_seconds?: number | null
  participants?: ZoomParticipantSnapshot[]
  recording_refs?: ZoomRecordingRef[]
  transcript_ref?: string | null
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomMeetingIngestResult {
  call_id: string
  account_id: string
  meeting_id: string
  event_id: string
  status: string
}

export interface ZoomRecordingObservationRequest {
  observation_id?: string | null
  account_id: string
  meeting_id: string
  recording: ZoomRecordingRef
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomRecordingIngestResult {
  account_id: string
  meeting_id: string
  recording_id: string
  event_id: string
  status: string
}

export interface ZoomTranscriptObservationRequest {
  observation_id?: string | null
  transcript_id: string
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  source_recording_ref?: string | null
  language_code?: string | null
  transcript_text: string
  segments?: unknown[]
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomTranscriptIngestResult {
  transcript_id: string
  call_id: string
  account_id: string
  meeting_id: string
  event_id: string
  status: string
}

export interface ZoomTranscriptFileImportRequest {
  observation_id?: string | null
  transcript_id: string
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  source_recording_ref?: string | null
  language_code?: string | null
  file_name?: string | null
  content_type?: string | null
  file_text: string
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomTranscriptFileImportResult extends ZoomTranscriptIngestResult {
  import_format: string
  parsed_segment_count: number
}

export interface ZoomProviderCall {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at?: string | null
  ended_at?: string | null
  transcription_policy_id?: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ZoomProviderCallListResponse {
  items: ZoomProviderCall[]
}

export interface ZoomCallTranscript {
  transcript_id: string
  call_id: string
  account_id: string
  provider_chat_id: string
  transcript_status: string
  stt_provider: string
  source_audio_ref?: string | null
  language_code?: string | null
  transcript_text: string
  segments: unknown
  provenance: unknown
  created_at: string
  updated_at: string
}

export interface ZoomCallTranscriptResponse {
  transcript: ZoomCallTranscript | null
}
