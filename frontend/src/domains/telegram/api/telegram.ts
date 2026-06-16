import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramCapabilitiesResponse,
  TelegramCallTranscriptResponse,
  TelegramCallListResponse,
  TelegramChatDetailResponse,
  TelegramChatGroupFilterListResponse,
  TelegramChatMemberListResponse,
  TelegramChatListResponse,
  TelegramChatActionRequest,
  TelegramChatActionResponse,
  TelegramMessageListResponse,
  TelegramRuntimeStatus,
  TelegramAccountListResponse,
  TelegramAccountLifecycleResponse,
  TelegramChatSyncRequest,
  TelegramChatSyncResponse,
  TelegramHistorySyncRequest,
  TelegramHistorySyncResponse,
  TelegramQrLoginStatusResponse,
  TelegramQrLoginStartRequest,
  TelegramQrLoginPasswordRequest,
  TelegramAccountSetupResponse,
  TelegramManualSendResponse,
  TelegramSendDryRunResponse,
  TelegramMessageIngestResponse,
  TelegramMediaDownloadRequest,
  TelegramMediaDownloadResponse,
  TelegramRuntimeStartRequest,
  TelegramChat,
  TelegramMessage,
  TelegramTopicListResponse
} from '../types/telegram'

// --- Capabilities ---
export async function fetchTelegramCapabilities(): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    '/api/v1/telegram/capabilities',
    'Telegram capabilities request failed'
  )
}

export async function fetchTelegramAccountCapabilities(
  accountId: string
): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    `/api/v1/telegram/accounts/${encodeURIComponent(accountId)}/capabilities`,
    'Telegram account capabilities request failed'
  )
}

// --- Accounts ---
export async function fetchTelegramAccounts(query?: string): Promise<TelegramAccountListResponse> {
  const qs = query?.trim() ? `?${query}` : ''
  return ApiClient.instance.get<TelegramAccountListResponse>(
    `/api/v1/telegram/accounts${qs}`,
    'Telegram account list request failed'
  )
}

export async function setupTelegramAccount(request: {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  bot_token?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  qr_authorized?: boolean
  transcription_enabled: boolean
}): Promise<TelegramAccountSetupResponse> {
  return ApiClient.instance.post<TelegramAccountSetupResponse>(
    '/api/v1/telegram/accounts',
    request,
    'Telegram account setup failed'
  )
}

export async function removeTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.delete<TelegramAccountLifecycleResponse>(
    `/api/v1/telegram/accounts/${encodeURIComponent(accountId)}`,
    'Telegram account remove failed'
  )
}

export async function logoutTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.post<TelegramAccountLifecycleResponse>(
    `/api/v1/telegram/accounts/${encodeURIComponent(accountId)}/logout`,
    {},
    'Telegram account logout failed'
  )
}

// --- Chats ---
export async function fetchTelegramChats(accountId?: string, limit = 50): Promise<TelegramChatListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<TelegramChatListResponse>(
    `/api/v1/telegram/chats?${params.toString()}`,
    'Telegram chats request failed'
  )
}

export async function fetchTelegramChatDetail(telegramChatId: string): Promise<TelegramChatDetailResponse> {
  return ApiClient.instance.get<TelegramChatDetailResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}`,
    'Telegram chat detail request failed'
  )
}

export async function fetchTelegramFolders(accountId?: string): Promise<TelegramChatGroupFilterListResponse> {
  const params = new URLSearchParams()
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  const suffix = params.size ? `?${params.toString()}` : ''
  return ApiClient.instance.get<TelegramChatGroupFilterListResponse>(
    `/api/v1/telegram/folders${suffix}`,
    'Telegram folders request failed'
  )
}

export async function fetchTelegramChatMembers(
  telegramChatId: string,
  limit = 50
): Promise<TelegramChatMemberListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<TelegramChatMemberListResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/members?${params.toString()}`,
    'Telegram chat members request failed'
  )
}

export async function syncTelegramChats(request: TelegramChatSyncRequest): Promise<TelegramChatSyncResponse> {
  return ApiClient.instance.post<TelegramChatSyncResponse>(
    '/api/v1/telegram/sync/chats',
    request,
    'Telegram chat sync failed'
  )
}

export async function pinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/pin`,
    request,
    'Telegram chat pin failed'
  )
}

export async function unpinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/unpin`,
    request,
    'Telegram chat unpin failed'
  )
}

export async function archiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/archive`,
    request,
    'Telegram chat archive failed'
  )
}

export async function unarchiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/unarchive`,
    request,
    'Telegram chat unarchive failed'
  )
}

export async function muteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/mute`,
    request,
    'Telegram chat mute failed'
  )
}

export async function unmuteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/unmute`,
    request,
    'Telegram chat unmute failed'
  )
}

export async function markTelegramChatRead(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/read`,
    request,
    'Telegram chat mark read failed'
  )
}

export async function markTelegramChatUnread(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/unread`,
    request,
    'Telegram chat mark unread failed'
  )
}

// --- Messages ---
export async function fetchTelegramMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<TelegramMessageListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  if (providerChatId?.trim()) {
    params.set('provider_chat_id', providerChatId.trim())
  }
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/telegram/messages?${params.toString()}`,
    'Telegram messages request failed'
  )
}

export async function syncTelegramHistory(request: TelegramHistorySyncRequest): Promise<TelegramHistorySyncResponse> {
  return ApiClient.instance.post<TelegramHistorySyncResponse>(
    '/api/v1/telegram/sync/history',
    request,
    'Telegram history sync failed'
  )
}

// --- Runtime ---
export async function fetchTelegramRuntimeStatus(accountId: string): Promise<TelegramRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<TelegramRuntimeStatus>(
    `/api/v1/telegram/runtime/status?${params.toString()}`,
    'Telegram runtime status request failed'
  )
}

export async function startTelegramRuntime(request: TelegramRuntimeStartRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/telegram/runtime/start',
    request,
    'Telegram runtime start failed'
  )
}

// --- Media ---
export async function downloadTelegramMedia(
  request: TelegramMediaDownloadRequest
): Promise<TelegramMediaDownloadResponse> {
  return ApiClient.instance.post<TelegramMediaDownloadResponse>(
    '/api/v1/telegram/media/download',
    request,
    'Telegram media download failed'
  )
}

// --- Send ---
export async function sendTelegramMessage(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<TelegramManualSendResponse> {
  return ApiClient.instance.post<TelegramManualSendResponse>(
    '/api/v1/telegram/messages/send',
    request,
    'Telegram send failed'
  )
}

export async function sendTelegramDryRun(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<TelegramSendDryRunResponse> {
  return ApiClient.instance.post<TelegramSendDryRunResponse>(
    '/api/v1/policies/telegram-send/dry-run',
    request,
    'Telegram send dry-run failed'
  )
}

export async function ingestTelegramFixtureMessage(request: {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_kind: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: string
}): Promise<TelegramMessageIngestResponse> {
  return ApiClient.instance.post<TelegramMessageIngestResponse>(
    '/api/v1/telegram/messages',
    request,
    'Telegram fixture message ingest failed'
  )
}

// --- QR Login ---
export async function startTelegramQrLogin(
  request: TelegramQrLoginStartRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    '/api/v1/telegram/login/qr/start',
    request,
    'Telegram QR login start failed'
  )
}

export async function pollTelegramQrLogin(setupId: string): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.get<TelegramQrLoginStatusResponse>(
    `/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}`,
    'Telegram QR login status request failed'
  )
}

export async function cancelTelegramQrLogin(setupId: string): Promise<{ setup_id: string; cancelled: boolean }> {
  return ApiClient.instance.delete<{ setup_id: string; cancelled: boolean }>(
    `/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}`,
    'Telegram QR login cancel failed'
  )
}

export async function submitTelegramQrPassword(
  setupId: string,
  request: TelegramQrLoginPasswordRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    `/api/v1/telegram/login/qr/${encodeURIComponent(setupId)}/password`,
    request,
    'Telegram QR password submit failed'
  )
}

// --- Calls ---
export async function fetchTelegramCalls(accountId?: string, limit = 50): Promise<TelegramCallListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<TelegramCallListResponse>(
    `/api/v1/calls?${params.toString()}`,
    'Telegram call request failed'
  )
}

export async function fetchTelegramCallTranscript(callId: string): Promise<TelegramCallTranscriptResponse> {
  return ApiClient.instance.get<TelegramCallTranscriptResponse>(
    `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
    'Telegram call transcript request failed'
  )
}

// --- Service functions (ported from Svelte services/telegram/) ---

/**
 * Extract the oldest TDLib message ID from a list of messages.
 * Used to determine the `from_message_id` for paginated older-history sync.
 */
export function telegramOldestTdlibMessageId(messages: TelegramMessage[]): number | null {
  const ids: number[] = []
  for (const message of messages) {
    const suffix = message.provider_message_id.split(':').at(-1)?.trim()
    if (suffix) {
      const parsed = Number.parseInt(suffix, 10)
      if (Number.isFinite(parsed) && parsed > 0) {
        ids.push(parsed)
      }
    }
  }
  return ids.length ? Math.min(...ids) : null
}

/**
 * Load full Telegram workspace: capabilities, accounts, chats, messages and runtime statuses.
 * Selects the appropriate chat messages if a chat is selected.
 */
export async function loadTelegramWorkspace(
  selectedChatId: string,
  _selectedCallId: string
): Promise<{
  chats: TelegramChat[]
  messages: TelegramMessage[]
  capabilities: TelegramCapabilitiesResponse | null
  runtimeStatuses: Record<string, TelegramRuntimeStatus>
  selectedChatId: string
  error: string
}> {
  try {
    const [capabilityResponse, accountResponse, chatResponse, messageResponse] = await Promise.all([
      fetchTelegramCapabilities(),
      fetchTelegramAccounts(),
      fetchTelegramChats(undefined, 500),
      fetchTelegramMessages()
    ])

    const chats = chatResponse.items
    let nextChatId = selectedChatId
    if (!chats.some((chat) => chat.provider_chat_id === nextChatId)) {
      nextChatId = chats[0]?.provider_chat_id ?? ''
    }

    const messages = nextChatId
      ? (await fetchTelegramMessages(
          chats.find((c) => c.provider_chat_id === nextChatId)?.account_id,
          nextChatId,
          100
        )).items
      : messageResponse.items

    // Load runtime statuses for all referenced accounts
    const accountIds = Array.from(
      new Set([
        ...accountResponse.items.map((a) => a.account_id),
        ...chats.map((c) => c.account_id)
      ].filter(Boolean))
    )
    const statusEntries = await Promise.all(
      accountIds.map(async (aid) => {
        try {
          const status = await fetchTelegramRuntimeStatus(aid)
          return [aid, status] as const
        } catch {
          return null
        }
      })
    )
    const runtimeStatuses = Object.fromEntries(
      statusEntries.filter((e): e is [string, TelegramRuntimeStatus] => e !== null)
    )

    return {
      chats,
      messages,
      capabilities: capabilityResponse,
      runtimeStatuses,
      selectedChatId: nextChatId,
      error: ''
    }
  } catch (error) {
    return {
      chats: [],
      messages: [],
      capabilities: null,
      runtimeStatuses: {},
      selectedChatId,
      error: error instanceof Error ? error.message : 'Telegram workspace load failed'
    }
  }
}

/**
 * Sync history for the selected Telegram chat.
 * For private chats defaults to 'full' mode, otherwise 'latest'.
 */
export async function syncTelegramSelectedHistory(params: {
  account_id: string
  provider_chat_id: string
  chat_kind?: string
  mode?: 'latest' | 'older' | 'full'
  from_message_id?: number
}): Promise<{
  message: string
  error: string
  providerChatId: string
  hasMore: boolean
}> {
  try {
    const mode = params.mode ?? (params.chat_kind === 'private' ? 'full' : 'latest')
    const result = await syncTelegramHistory({
      account_id: params.account_id,
      provider_chat_id: params.provider_chat_id,
      mode,
      limit: 100,
      ...(params.from_message_id != null ? { from_message_id: params.from_message_id } : {})
    })
    return {
      message: `Telegram history synced: ${result.synced_count}`,
      error: '',
      providerChatId: result.provider_chat_id,
      hasMore: result.has_more
    }
  } catch (error) {
    return {
      message: '',
      error: error instanceof Error ? error.message : 'Telegram history sync failed',
      providerChatId: params.provider_chat_id,
      hasMore: false
    }
  }
}

/**
 * Sync older Telegram history (pagination) using a known `from_message_id`.
 */
export async function syncTelegramOlderHistory(params: {
  account_id: string
  provider_chat_id: string
  from_message_id: number
}): Promise<{
  message: string
  error: string
  hasMore: boolean
}> {
  const result = await syncTelegramSelectedHistory({
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    from_message_id: params.from_message_id,
    mode: 'older'
  })
  return {
    message: result.message,
    error: result.error,
    hasMore: result.hasMore
  }
}

/**
 * Send a manual Telegram message with error handling wrapper.
 */
export async function sendTelegramManualMessage(params: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<{
  error: string
  message: string
  providerChatId: string
  nextText: string
}> {
  try {
    const result = await sendTelegramMessage({
      account_id: params.account_id,
      provider_chat_id: params.provider_chat_id,
      text: params.text
    })
    return {
      error: '',
      message: `Telegram message ${result.status}`,
      providerChatId: result.provider_chat_id,
      nextText: ''
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram send failed',
      message: '',
      providerChatId: params.provider_chat_id,
      nextText: params.text
    }
  }
}

/**
 * Start Telegram runtime with UI-friendly error handling wrapper.
 * Returns { error, message, status } so callers can handle success/failure uniformly.
 */
export async function startTelegramRuntimeFromUi(
  accountId: string
): Promise<{
  error: string
  message: string
  status: TelegramRuntimeStatus | null
}> {
  try {
    const status = await startTelegramRuntime({ account_id: accountId })
    return {
      error: '',
      message: `Telegram runtime ${status.status}`,
      status
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram runtime start failed',
      message: '',
      status: null
    }
  }
}

/**
 * Sync Telegram chats with UI-friendly error handling wrapper.
 */
export async function syncTelegramChatsFromUi(
  accountId: string
): Promise<{
  error: string
  message: string
  result: TelegramChatSyncResponse | null
}> {
  try {
    const result = await syncTelegramChats({ account_id: accountId })
    return {
      error: '',
      message: `Telegram chats synced: ${result.synced_count}`,
      result
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram chat sync failed',
      message: '',
      result: null
    }
  }
}

/**
 * Download Telegram media with UI-friendly error handling wrapper.
 */
export async function downloadTelegramMediaFromUi(
  request: TelegramMediaDownloadRequest
): Promise<{
  error: string
  message: string
  result: TelegramMediaDownloadResponse | null
}> {
  try {
    const result = await downloadTelegramMedia(request)
    return {
      error: '',
      message: `Telegram media download started: ${result.tdlib_file_id}`,
      result
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram media download failed',
      message: '',
      result: null
    }
  }
}

// --- Forum topics ---

export async function fetchTelegramTopics(
  telegramChatId: string,
  limit = 100
): Promise<TelegramTopicListResponse> {
  return ApiClient.instance.get<TelegramTopicListResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(telegramChatId)}/topics?limit=${limit}`,
    'Telegram topics fetch failed'
  )
}

export async function fetchTelegramTopicMessages(
  topicId: string,
  limit = 50
): Promise<TelegramMessageListResponse> {
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/telegram/topics/${encodeURIComponent(topicId)}/messages?limit=${limit}`,
    'Telegram topic messages fetch failed'
  )
}

export {
  addTelegramReaction,
  deleteTelegramMessage,
  editTelegramMessage,
  fetchTelegramCommands,
  fetchTelegramMessageTombstones,
  fetchTelegramMessageVersions,
  fetchTelegramReactions,
  removeTelegramReaction,
  restoreTelegramMessageVisibility,
} from './telegramLifecycle'
