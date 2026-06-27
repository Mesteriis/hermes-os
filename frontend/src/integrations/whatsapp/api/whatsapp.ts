import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappAccountListResponse,
  WhatsAppChatSyncResponse,
  WhatsappCapabilitiesResponse,
  WhatsAppCallsSyncResponse,
  WhatsAppContactsSyncResponse,
  WhatsAppMediaSyncResponse,
  WhatsAppMembersSyncResponse,
  WhatsAppPresenceSyncResponse,
  WhatsAppStatusSyncResponse,
  WhatsappLiveAccountSetupRequest,
  WhatsAppProviderCommand,
  WhatsAppProviderCommandListResponse,
  WhatsAppPairCodeSession,
  WhatsAppQrLinkSession,
  WhatsAppRuntimeHealth,
  WhatsAppRuntimeRemoveResponse,
  WhatsAppRuntimeStatus,
  WhatsappWebSessionListResponse,
  WhatsappWebAccountSetupRequest,
  WhatsappWebAccountSetupResponse,
  WhatsappWebFixtureMessageRequest,
  WhatsappWebMessageIngestResponse,
  WhatsappWebSession,
} from '../types/whatsapp'

// --- Capabilities ---
export async function fetchWhatsappCapabilities(): Promise<WhatsappCapabilitiesResponse> {
  return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
    '/api/v1/integrations/whatsapp/capabilities',
    'WhatsApp capabilities request failed'
  )
}

export async function fetchWhatsappAccountCapabilities(
  accountId: string
): Promise<WhatsappCapabilitiesResponse> {
  return ApiClient.instance.get<WhatsappCapabilitiesResponse>(
    `/api/v1/integrations/whatsapp/accounts/${encodeURIComponent(accountId)}/capabilities`,
    'WhatsApp account capabilities request failed'
  )
}

export async function fetchWhatsappAccounts(
  includeRemoved = false
): Promise<WhatsappAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) {
    params.set('include_removed', 'true')
  }
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<WhatsappAccountListResponse>(
    `/api/v1/integrations/whatsapp/accounts${suffix}`,
    'WhatsApp accounts request failed'
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

export async function fetchWhatsappRuntimeStatus(accountId: string): Promise<WhatsAppRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<WhatsAppRuntimeStatus>(
    `/api/v1/integrations/whatsapp/runtime/status?${params.toString()}`,
    'WhatsApp runtime status request failed'
  )
}

export async function fetchWhatsappRuntimeHealth(accountId: string): Promise<WhatsAppRuntimeHealth> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<WhatsAppRuntimeHealth>(
    `/api/v1/integrations/whatsapp/runtime/health?${params.toString()}`,
    'WhatsApp runtime health request failed'
  )
}

export async function startWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/start',
    request,
    'WhatsApp runtime start failed'
  )
}

export async function stopWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/stop',
    request,
    'WhatsApp runtime stop failed'
  )
}

export async function revokeWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/revoke',
    request,
    'WhatsApp runtime revoke failed'
  )
}

export async function relinkWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/relink',
    request,
    'WhatsApp runtime relink failed'
  )
}

export async function rotateWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeStatus> {
  return ApiClient.instance.post<WhatsAppRuntimeStatus>(
    '/api/v1/integrations/whatsapp/runtime/rotate',
    request,
    'WhatsApp runtime rotate failed'
  )
}

export async function removeWhatsappRuntime(request: {
  account_id: string
}): Promise<WhatsAppRuntimeRemoveResponse> {
  return ApiClient.instance.post<WhatsAppRuntimeRemoveResponse>(
    '/api/v1/integrations/whatsapp/runtime/remove',
    request,
    'WhatsApp runtime remove failed'
  )
}

export async function startWhatsappQrLink(request: {
  account_id: string
}): Promise<WhatsAppQrLinkSession> {
  return ApiClient.instance.post<WhatsAppQrLinkSession>(
    '/api/v1/integrations/whatsapp/login/qr/start',
    request,
    'WhatsApp QR link start failed'
  )
}

export async function startWhatsappPairCodeLink(request: {
  account_id: string
  phone_number: string
}): Promise<WhatsAppPairCodeSession> {
  return ApiClient.instance.post<WhatsAppPairCodeSession>(
    '/api/v1/integrations/whatsapp/login/pair-code/start',
    request,
    'WhatsApp pair-code link start failed'
  )
}

export async function fetchWhatsappProviderCommands(params: {
  account_id: string
  provider_chat_id?: string
  provider_message_id?: string
  command_kinds?: string[]
  limit?: number
}): Promise<WhatsAppProviderCommandListResponse> {
  const query = new URLSearchParams({ account_id: params.account_id.trim() })
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.provider_message_id?.trim()) {
    query.set('provider_message_id', params.provider_message_id.trim())
  }
  if (params.command_kinds?.length) {
    query.set('command_kinds', params.command_kinds.join(','))
  }
  if (typeof params.limit === 'number') {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsAppProviderCommandListResponse>(
    `/api/v1/integrations/whatsapp/commands?${query.toString()}`,
    'WhatsApp provider commands request failed'
  )
}

export async function fetchWhatsappSyncPresence(params: {
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsAppPresenceSyncResponse> {
  return ApiClient.instance.post<WhatsAppPresenceSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/presence',
    params,
    'WhatsApp presence sync request failed'
  )
}

export async function fetchWhatsappSyncChats(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppChatSyncResponse> {
  return ApiClient.instance.post<WhatsAppChatSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/chats',
    params,
    'WhatsApp chats sync request failed'
  )
}

export async function fetchWhatsappSyncHistory(params: {
  account_id: string
  provider_chat_id: string
  limit?: number
}): Promise<WhatsAppStatusSyncResponse> {
  return ApiClient.instance.post<WhatsAppStatusSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/history',
    params,
    'WhatsApp history sync request failed'
  )
}

export async function fetchWhatsappSyncMembers(params: {
  account_id: string
  provider_chat_id: string
  limit?: number
}): Promise<WhatsAppMembersSyncResponse> {
  return ApiClient.instance.post<WhatsAppMembersSyncResponse>(
    `/api/v1/integrations/whatsapp/provider-sync/conversations/${encodeURIComponent(params.provider_chat_id)}/members`,
    { account_id: params.account_id, limit: params.limit },
    'WhatsApp members sync request failed'
  )
}

export async function fetchWhatsappSyncCalls(params: {
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsAppCallsSyncResponse> {
  return ApiClient.instance.post<WhatsAppCallsSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/calls',
    params,
    'WhatsApp calls sync request failed'
  )
}

export async function fetchWhatsappSyncContacts(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppContactsSyncResponse> {
  return ApiClient.instance.post<WhatsAppContactsSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/contacts',
    params,
    'WhatsApp contacts sync request failed'
  )
}

export async function fetchWhatsappSyncStatuses(params: {
  account_id: string
  limit?: number
}): Promise<WhatsAppStatusSyncResponse> {
  return ApiClient.instance.post<WhatsAppStatusSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/statuses',
    params,
    'WhatsApp status sync request failed'
  )
}

export async function fetchWhatsappSyncMedia(params: {
  account_id: string
  provider_chat_id?: string
  content_type?: string
  limit?: number
}): Promise<WhatsAppMediaSyncResponse> {
  return ApiClient.instance.post<WhatsAppMediaSyncResponse>(
    '/api/v1/integrations/whatsapp/provider-sync/media',
    params,
    'WhatsApp media sync request failed'
  )
}

export async function publishWhatsappStatus(request: {
  account_id: string
  idempotency_key: string
  text: string
  command_id?: string
}): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    '/api/v1/integrations/whatsapp/provider-commands/statuses/publish',
    request,
    'WhatsApp status publish failed'
  )
}

export async function retryWhatsappProviderCommand(
  commandId: string
): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    `/api/v1/integrations/whatsapp/commands/${encodeURIComponent(commandId)}/retry`,
    {},
    'WhatsApp provider command retry failed'
  )
}

export async function deadLetterWhatsappProviderCommand(params: {
  command_id: string
  reason: string
}): Promise<WhatsAppProviderCommand> {
  return ApiClient.instance.post<WhatsAppProviderCommand>(
    `/api/v1/integrations/whatsapp/commands/${encodeURIComponent(params.command_id)}/dead-letter`,
    { reason: params.reason },
    'WhatsApp provider command dead-letter failed'
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

export async function setupWhatsappLiveAccount(
  request: WhatsappLiveAccountSetupRequest
): Promise<WhatsappWebAccountSetupResponse> {
  return ApiClient.instance.post<WhatsappWebAccountSetupResponse>(
    '/api/v1/integrations/whatsapp/accounts',
    request,
    'WhatsApp live account setup request failed'
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

export async function loadWhatsappWebWorkspace(
  selectedSessionId: string
): Promise<{
  capabilities: WhatsappCapabilitiesResponse | null
  sessions: WhatsappWebSession[]
  selectedSessionId: string
  error: string
}> {
  try {
    const [capabilityResponse, sessionResponse] = await Promise.all([
      fetchWhatsappCapabilities(),
      fetchWhatsappWebSessions(),
    ])

    const sessions = sessionResponse.items
    let nextSessionId = selectedSessionId
    if (!sessions.some((s) => s.session_id === nextSessionId)) {
      nextSessionId = sessions[0]?.session_id ?? ''
    }

    return {
      capabilities: capabilityResponse,
      sessions,
      selectedSessionId: nextSessionId,
      error: ''
    }
  } catch (error) {
    return {
      capabilities: null,
      sessions: [],
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
