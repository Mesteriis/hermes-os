export interface SignalHubSource {
  id: string
  code: string
  display_name: string
  category: string
  source_kind: string
  default_enabled: boolean
  supports_connections: boolean
  supports_runtime: boolean
  supports_replay: boolean
  supports_pause: boolean
  supports_mute: boolean
  capability_schema_version: number
  created_at: string
  updated_at: string
}

export interface SignalHubSourcesResponse {
  items: SignalHubSource[]
}

export interface SignalHubCapability {
  id: string
  source_code: string
  connection_id: string | null
  capability: string
  state: string
  reason: string | null
  requires_confirmation: boolean
  action_class: string
  updated_at: string
}

export interface SignalHubCapabilitiesResponse {
  items: SignalHubCapability[]
}

export interface SignalHubConnection {
  id: string
  source_code: string
  display_name: string
  status: string
  profile: string | null
  settings: Record<string, unknown>
  secret_ref: string | null
  connected_at: string | null
  last_seen_at: string | null
  last_signal_at: string | null
  last_sync_at: string | null
  created_at: string
  updated_at: string
}

export interface SignalHubConnectionsResponse {
  items: SignalHubConnection[]
}

export interface SignalHubConnectionResponse {
  item: SignalHubConnection
}

export interface SignalHubConnectionCreateRequest {
  source_code: string
  display_name: string
  status: string
  profile?: string | null
  settings?: Record<string, unknown>
  secret_ref?: string | null
}

export interface SignalHubConnectionUpdateRequest {
  display_name?: string
  status?: string
  profile?: string | null
  settings?: Record<string, unknown>
  secret_ref?: string | null
}

export interface SignalHubHealth {
  id: string
  source_code: string
  connection_id: string | null
  level: string
  summary: string
  last_ok_at: string | null
  last_failure_at: string | null
  failure_count: number
  consecutive_failure_count: number
  next_retry_at: string | null
  evidence: Record<string, unknown>
  updated_at: string
}

export interface SignalHubHealthResponse {
  items: SignalHubHealth[]
}

export interface SignalHubHealthCheckRequest {
  source_code: string
  connection_id?: string | null
}

export interface SignalHubRuntimeState {
  id: string
  source_code: string
  connection_id: string | null
  runtime_kind: string
  state: string
  last_started_at: string | null
  last_stopped_at: string | null
  last_heartbeat_at: string | null
  last_error_at: string | null
  last_error_code: string | null
  last_error_message_redacted: string | null
  metadata: Record<string, unknown>
  updated_at: string
}

export interface SignalHubRuntimeStatesResponse {
  items: SignalHubRuntimeState[]
}

export interface SignalHubReplayRequest {
  id: string
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  from_position: bigint | null
  to_position: bigint | null
  from_time: string | null
  to_time: string | null
  target_consumer: string | null
  target_projection: string | null
  status: string
  requested_by: string
  requested_at: string
  started_at: string | null
  completed_at: string | null
  last_error_redacted: string | null
  replayed_count: number
  metadata: Record<string, unknown>
}

export interface SignalHubReplayRequestsResponse {
  items: SignalHubReplayRequest[]
}

export interface SignalHubReplayRequestCreateRequest {
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  from_position?: bigint | number | string | null
  to_position?: bigint | number | string | null
  from_time?: string | null
  to_time?: string | null
  target_consumer?: string | null
  target_projection?: string | null
  metadata?: Record<string, unknown>
}

export interface SignalHubProfile {
  id: string
  code: string
  display_name: string
  description: string
  policy_count: number
  source_policies: SignalHubProfilePolicy[]
  is_system: boolean
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface SignalHubProfilesResponse {
  items: SignalHubProfile[]
}

export interface SignalHubProfilePolicy {
  scope: SignalHubPolicyScope
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  mode: SignalHubPolicyMode
  reason: string
}

export interface SignalHubProfileCreateRequest {
  code: string
  display_name: string
  description: string
  source_policies: SignalHubProfilePolicy[]
}

export interface SignalHubProfileUpdateRequest {
  display_name?: string
  description?: string
  source_policies?: SignalHubProfilePolicy[]
}

export type SignalHubPolicyScope =
  | 'global'
  | 'source'
  | 'connection'
  | 'event_pattern'
  | 'profile'

export type SignalHubPolicyMode =
  | 'enabled'
  | 'disabled'
  | 'muted'
  | 'paused'
  | 'replay_only'
  | (string & {})

export interface SignalHubPolicy {
  scope: SignalHubPolicyScope
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  mode: SignalHubPolicyMode
  reason: string
  expires_at: string | null
}

export interface SignalHubPoliciesResponse {
  items: SignalHubPolicy[]
}

export interface SignalHubPolicyRequest {
  scope: SignalHubPolicyScope
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  mode: SignalHubPolicyMode
  reason: string
  expires_at?: string | null
}

export interface SignalHubCreatePolicyResponse {
  id: string
}

export interface SignalHubControlRequest {
  scope: SignalHubPolicyScope
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  reason?: string | null
}

export interface SignalHubControlResponse {
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  policy_id: string | null
  cleared_count: number
}

export interface SignalHubRuntimeStateRequest {
  source_code: string
  runtime_kind: string
  state: string
  metadata?: Record<string, unknown>
}
