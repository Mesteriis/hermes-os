// --- Provider ---
export type WhatsappWebProviderKind = 'whatsapp_web'

// --- Capabilities ---
export type WhatsappCapabilityStatus = {
  capability: string
  status: 'available' | 'blocked' | string
  closure_gate: boolean
  reason: string
}

export type WhatsappCapabilitiesResponse = {
  version: string
  runtime_mode: string
  capabilities: WhatsappCapabilityStatus[]
  unsupported_features: string[]
}

// --- Sessions ---
export type WhatsappWebSession = {
  session_id: string
  account_id: string
  device_name: string
  companion_runtime: 'fixture' | 'manual_webview' | 'blocked'
  link_state: 'fixture' | 'qr_pending' | 'linked' | 'degraded' | 'revoked' | 'blocked'
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

// --- Account setup ---
export type WhatsappWebAccountSetupRequest = {
  account_id: string
  provider_kind: WhatsappWebProviderKind
  display_name: string
  external_account_id: string
  device_name: string
  local_state_path: string
}

export type WhatsappWebAccountSetupResponse = {
  account_id: string
  provider_kind: WhatsappWebProviderKind
  runtime: string
  session: WhatsappWebSession
}

// --- Fixture ---
export type WhatsappWebFixtureMessageRequest = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
}

export type WhatsappWebMessageIngestResponse = {
  raw_record_id: string
  message_id: string
}
