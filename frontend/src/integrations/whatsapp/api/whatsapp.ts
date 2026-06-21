import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappCapabilitiesResponse,
  WhatsappWebMessageListResponse,
  WhatsappWebSessionListResponse,
  WhatsappWebAccountSetupRequest,
  WhatsappWebAccountSetupResponse,
  WhatsappWebFixtureMessageRequest,
  WhatsappWebMessageIngestResponse,
  WhatsappWebSession,
  WhatsappWebMessage
} from '../types/whatsapp'

// --- Capabilities ---
export async function fetchWhatsappCapabilities(): Promise<WhatsappCapabilitiesResponse> {
  return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
    '/api/v1/integrations/whatsapp/capabilities',
    'WhatsApp capabilities request failed'
  )
}

// --- Sessions ---
export async function fetchWhatsappWebSessions(
  accountId?: string,
  limit = 50
): Promise<WhatsappWebSessionListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<WhatsappWebSessionListResponse>(
    `/api/v1/integrations/whatsapp/sessions?${params.toString()}`,
    'WhatsApp Web sessions request failed'
  )
}

// --- Messages ---
export async function fetchWhatsappWebMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<WhatsappWebMessageListResponse> {
  void accountId
  void providerChatId
  void limit
  return Promise.reject(
    new Error(
      'WhatsApp Web messages moved to frontend/src/domains/communications/api/providerChannels; integration clients own runtime/control only'
    )
  )
}

// --- Account setup ---
export async function setupWhatsappWebFixtureAccount(
  request: WhatsappWebAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
  return ApiClient.instance.post<WhatsappWebAccountSetupResponse>(
    '/api/v1/integrations/whatsapp/fixtures/accounts',
    request,
    'WhatsApp Web account setup request failed'
  )
}

// --- Fixture message ingest ---
export async function ingestWhatsappWebFixtureMessage(
  request: WhatsappWebFixtureMessageRequest
): Promise<WhatsappWebMessageIngestResponse> {
  return ApiClient.instance.post<WhatsappWebMessageIngestResponse>(
    '/api/v1/integrations/whatsapp/fixtures/messages',
    request,
    'WhatsApp Web fixture message request failed'
  )
}

// --- Service functions ---

export function whatsappMessageTime(message: WhatsappWebMessage): string {
  const date = message.occurred_at ?? message.projected_at
  if (!date) return ''
  try {
    return new Date(date).toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return ''
  }
}

export async function loadWhatsappWebWorkspace(
  selectedSessionId: string
): Promise<{
  capabilities: WhatsappCapabilitiesResponse | null
  sessions: WhatsappWebSession[]
  messages: WhatsappWebMessage[]
  selectedSessionId: string
  error: string
}> {
  try {
    const [capabilityResponse, sessionResponse, messageResponse] = await Promise.all([
      fetchWhatsappCapabilities(),
      fetchWhatsappWebSessions(),
      fetchWhatsappWebMessages()
    ])

    const sessions = sessionResponse.items
    let nextSessionId = selectedSessionId
    if (!sessions.some((s) => s.session_id === nextSessionId)) {
      nextSessionId = sessions[0]?.session_id ?? ''
    }

    return {
      capabilities: capabilityResponse,
      sessions,
      messages: messageResponse.items,
      selectedSessionId: nextSessionId,
      error: ''
    }
  } catch (error) {
    return {
      capabilities: null,
      sessions: [],
      messages: [],
      selectedSessionId,
      error: error instanceof Error ? error.message : 'Unknown WhatsApp Web workspace error'
    }
  }
}

export async function setupWhatsappWebFixture(params: {
  account_id: string
  display_name: string
  external_account_id: string
  device_name: string
  local_state_path: string
}): Promise<{
  message: string
  error: string
  sessionId: string
  accountId: string
  providerKind: string
}> {
  try {
    const result = await setupWhatsappWebFixtureAccount({
      account_id: params.account_id,
      provider_kind: 'whatsapp_web',
      display_name: params.display_name,
      external_account_id: params.external_account_id,
      device_name: params.device_name,
      local_state_path: params.local_state_path
    })
    const providerKindLabel = result.provider_kind
      .split('_')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ')
    return {
      message: `${providerKindLabel} account ${result.account_id} saved`,
      error: '',
      sessionId: result.session.session_id,
      accountId: result.account_id,
      providerKind: result.provider_kind
    }
  } catch (error) {
    return {
      message: '',
      error: error instanceof Error ? error.message : 'WhatsApp Web fixture setup failed',
      sessionId: '',
      accountId: params.account_id,
      providerKind: 'whatsapp_web'
    }
  }
}

export async function ingestWhatsappWebMessageFixture(params: {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: string
}): Promise<{
  message: string
  error: string
  nextProviderMessageId: string
  nextOccurredAt: string
}> {
  try {
    const providerMessageId = params.provider_message_id.trim() || `wa-fixture-msg-${crypto.randomUUID()}`
    const result = await ingestWhatsappWebFixtureMessage({
      account_id: params.account_id,
      provider_chat_id: params.provider_chat_id,
      provider_message_id: providerMessageId,
      chat_title: params.chat_title,
      sender_id: params.sender_id,
      sender_display_name: params.sender_display_name,
      text: params.text,
      import_batch_id: params.import_batch_id,
      occurred_at: params.occurred_at || new Date().toISOString(),
      delivery_state: params.delivery_state as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
    })
    return {
      message: `WhatsApp Web message ${result.message_id} projected`,
      error: '',
      nextProviderMessageId: `wa-fixture-msg-${crypto.randomUUID()}`,
      nextOccurredAt: new Date().toISOString()
    }
  } catch (error) {
    return {
      message: '',
      error: error instanceof Error ? error.message : 'WhatsApp Web fixture ingest failed',
      nextProviderMessageId: params.provider_message_id,
      nextOccurredAt: params.occurred_at
    }
  }
}

export function selectWhatsappSession(
  session: WhatsappWebSession,
  messageForm: Record<string, unknown>
): Record<string, unknown> {
  return {
    ...messageForm,
    account_id: session.account_id
  }
}
