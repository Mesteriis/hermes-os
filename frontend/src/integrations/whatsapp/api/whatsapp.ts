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
  WhatsAppProviderCommand,
  WhatsAppProviderCommandListResponse,
  WhatsAppRuntimeHealth,
  WhatsAppRuntimeRemoveResponse,
  WhatsAppRuntimeStatus,
  WhatsappWebSessionListResponse,
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

export function selectWhatsappSession(
  session: WhatsappWebSession,
  messageForm: Record<string, unknown>
): Record<string, unknown> {
  return {
    ...messageForm,
    account_id: session.account_id
  }
}
