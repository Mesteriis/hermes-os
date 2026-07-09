export interface AiSettingsOverviewResponse {
  providers: AiProviderAccount[]
  models: AiModelCatalogItem[]
  routes: AiModelRoute[]
  prompts: AiPromptTemplate[]
  eval_runs: AiPromptEvalRun[]
  capability_slots: AiCapabilitySlot[]
  provider_presets: AiProviderPreset[]
}

export interface AiHubUsageStatsResponse {
  generated_at: string
  window_hours: number
  totals: AiHubUsageTotals
  providers: AiHubProviderUsageStats[]
  hourly: AiHubHourlyUsageStats[]
}

export interface AiHubUsageTotals {
  request_count: number
  completed_count: number
  failed_count: number
  estimated_tokens: number
  estimated_cost_usd?: number | null
  avg_latency_ms?: number | null
}

export interface AiHubProviderUsageStats {
  provider_id: string
  provider_kind: string
  provider_key: string
  display_name: string
  status: string
  request_count: number
  completed_count: number
  failed_count: number
  estimated_tokens: number
  estimated_cost_usd?: number | null
  avg_latency_ms?: number | null
  balance_remaining_usd?: number | null
  token_quota_remaining?: number | null
  last_request_at?: string | null
}

export interface AiHubHourlyUsageStats {
  hour: string
  provider_id: string
  request_count: number
  failed_count: number
  estimated_tokens: number
}

export interface AiCapabilitySlot {
  slot: string
  label: string
  description: string
  requires_embedding_dimension?: number | null
}

export interface AiProviderPreset {
  provider_kind: string
  provider_key: string
  display_name: string
  privacy: string
  base_url?: string | null
  command_preset?: string | null
  capabilities: string[]
}

export interface AiProviderAuthStartRequest {
  provider_kind: string
  provider_key: string
  display_name?: string | null
  callback_url: string
}

export interface AiProviderAuthStartResponse {
  setup_id: string
  provider_id: string
  provider_kind: string
  provider_key: string
  display_name: string
  callback_url: string
  login_command?: string | null
  status: string
  message: string
  expires_at: string
  provider?: AiProviderAccount | null
}

export interface AiProviderAuthStatusResponse {
  setup_id: string
  provider_id: string
  provider_kind: string
  provider_key: string
  display_name: string
  callback_url: string
  login_command?: string | null
  status: string
  message: string
  expires_at: string
  provider?: AiProviderAccount | null
}

export interface AiProviderAccount {
  provider_id: string
  provider_kind: string
  provider_key: string
  display_name: string
  status: string
  consent_state: string
  consented_at?: string | null
  config: Record<string, unknown>
  capabilities: string[]
  created_at: string
  updated_at: string
}

export interface AiProviderCreateRequest {
  provider_id?: string | null
  provider_kind: string
  provider_key: string
  display_name: string
  base_url?: string | null
  command_preset?: string | null
  config?: Record<string, unknown> | null
  capabilities?: string[] | null
  enabled?: boolean
  remote_context_consent?: boolean | null
  api_key?: string | null
}

export interface AiProviderPatchRequest {
  display_name?: string
  base_url?: string | null
  config?: Record<string, unknown> | null
  enabled?: boolean
  api_key?: string | null
}

export interface AiProviderConsentRequest {
  consented: boolean
}

export interface AiProviderCommandResponse {
  provider_id: string
  command: string
  status: string
  message: string
}

export interface AiModelCatalogItem {
  provider_id: string
  model_key: string
  display_name: string
  category: string
  privacy: string
  capabilities: string[]
  context_window?: number | null
  embedding_dimension?: number | null
  is_available: boolean
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface AiModelAvailabilityUpdateRequest {
  provider_id: string
  model_key: string
  is_available: boolean
}

export interface AiModelDownloadRequest {
  provider_id: string
  model_key: string
}

export interface AiModelRoute {
  capability_slot: string
  provider_id: string
  model_key: string
  created_at: string
  updated_at: string
}

export interface AiModelRouteUpdateRequest {
  provider_id: string
  model_key: string
}

export interface AiPromptTemplate {
  prompt_id: string
  name: string
  entity_scope: string
  capability_slot: string
  description?: string | null
  is_system: boolean
  active_version_id?: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface AiPromptEvalRun {
  eval_run_id: string
  prompt_id: string
  prompt_version_id: string
  provider_id: string
  model_key: string
  source_refs: unknown[]
  variables: Record<string, unknown>
  output_text: string
  score?: number | null
  notes?: string | null
  actor_id: string
  created_at: string
}
