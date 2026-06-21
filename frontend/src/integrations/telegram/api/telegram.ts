import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramCapabilitiesResponse,
  TelegramCallTranscriptResponse,
  TelegramCallListResponse,
  TelegramChatGroupFilterListResponse,
  TelegramChatMembersSyncResponse,
  TelegramChatActionRequest,
  TelegramChatActionResponse,
  TelegramChatLifecycleCommandResponse,
  TelegramChatFolderReassignRequest,
  TelegramChatFolderReassignResponse,
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
  TelegramSendDryRunResponse,
  TelegramMessageIngestResponse,
  TelegramMediaDownloadRequest,
  TelegramMediaDownloadResponse,
  TelegramRuntimeRestartRequest,
  TelegramRuntimeStartRequest,
  TelegramRuntimeStopRequest,
} from '../types/telegram'

// --- Capabilities ---
export async function fetchTelegramCapabilities(): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    '/api/v1/integrations/telegram/capabilities',
    'Telegram capabilities request failed'
  )
}

export async function fetchTelegramAccountCapabilities(
  accountId: string
): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}/capabilities`,
    'Telegram account capabilities request failed'
  )
}

// --- Accounts ---
export async function fetchTelegramAccounts(query?: string): Promise<TelegramAccountListResponse> {
  const qs = query?.trim() ? `?${query}` : ''
  return ApiClient.instance.get<TelegramAccountListResponse>(
    `/api/v1/integrations/telegram/accounts${qs}`,
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
    '/api/v1/integrations/telegram/accounts',
    request,
    'Telegram account setup failed'
  )
}

export async function removeTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.delete<TelegramAccountLifecycleResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}`,
    'Telegram account remove failed'
  )
}

export async function logoutTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.post<TelegramAccountLifecycleResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}/logout`,
    {},
    'Telegram account logout failed'
  )
}

export async function fetchTelegramFolders(accountId?: string): Promise<TelegramChatGroupFilterListResponse> {
  const params = new URLSearchParams()
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  const suffix = params.size ? `?${params.toString()}` : ''
  return ApiClient.instance.get<TelegramChatGroupFilterListResponse>(
    `/api/v1/integrations/telegram/conversation-folders${suffix}`,
    'Telegram folders request failed'
  )
}

export async function syncTelegramChatMembers(telegramChatId: string): Promise<TelegramChatMembersSyncResponse> {
  return ApiClient.instance.post<TelegramChatMembersSyncResponse>(
    `/api/v1/integrations/telegram/provider-sync/conversations/${encodeURIComponent(telegramChatId)}/members`,
    {},
    'Telegram chat members sync failed'
  )
}

export async function syncTelegramChats(request: TelegramChatSyncRequest): Promise<TelegramChatSyncResponse> {
  return ApiClient.instance.post<TelegramChatSyncResponse>(
    '/api/v1/integrations/telegram/provider-sync/chats',
    request,
    'Telegram chat sync failed'
  )
}

export async function pinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/pin`,
    request,
    'Telegram chat pin failed'
  )
}

export async function unpinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unpin`,
    request,
    'Telegram chat unpin failed'
  )
}

export async function archiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/archive`,
    request,
    'Telegram chat archive failed'
  )
}

export async function unarchiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unarchive`,
    request,
    'Telegram chat unarchive failed'
  )
}

export async function muteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/mute`,
    request,
    'Telegram chat mute failed'
  )
}

export async function addTelegramChatToFolder(
  telegramChatId: string,
  providerFolderId: number,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/${providerFolderId}`,
    request,
    'Telegram chat folder add failed'
  )
}

export async function removeTelegramChatFromFolder(
  telegramChatId: string,
  providerFolderId: number,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/${providerFolderId}/remove`,
    request,
    'Telegram chat folder remove failed'
  )
}

export async function reassignTelegramChatFolders(
  telegramChatId: string,
  request: TelegramChatFolderReassignRequest
): Promise<TelegramChatFolderReassignResponse> {
  return ApiClient.instance.post<TelegramChatFolderReassignResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/reassign`,
    request,
    'Telegram chat folder reassignment failed'
  )
}

export async function unmuteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unmute`,
    request,
    'Telegram chat unmute failed'
  )
}

export async function markTelegramChatRead(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/read`,
    request,
    'Telegram chat mark read failed'
  )
}

export async function markTelegramChatUnread(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unread`,
    request,
    'Telegram chat mark unread failed'
  )
}

export async function joinTelegramChat(
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    '/api/v1/integrations/telegram/provider-commands/conversations/join',
    request,
    'Telegram chat join failed'
  )
}

export async function leaveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/leave`,
    request,
    'Telegram chat leave failed'
  )
}

export async function syncTelegramHistory(request: TelegramHistorySyncRequest): Promise<TelegramHistorySyncResponse> {
  return ApiClient.instance.post<TelegramHistorySyncResponse>(
    '/api/v1/integrations/telegram/provider-sync/history',
    request,
    'Telegram history sync failed'
  )
}

// --- Runtime ---
export async function fetchTelegramRuntimeStatus(accountId: string): Promise<TelegramRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<TelegramRuntimeStatus>(
    `/api/v1/integrations/telegram/runtime/status?${params.toString()}`,
    'Telegram runtime status request failed'
  )
}

export async function startTelegramRuntime(request: TelegramRuntimeStartRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/start',
    request,
    'Telegram runtime start failed'
  )
}

export async function stopTelegramRuntime(request: TelegramRuntimeStopRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/stop',
    request,
    'Telegram runtime stop failed'
  )
}

export async function restartTelegramRuntime(request: TelegramRuntimeRestartRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/restart',
    request,
    'Telegram runtime restart failed'
  )
}

// --- Media ---
export async function downloadTelegramMedia(
  request: TelegramMediaDownloadRequest
): Promise<TelegramMediaDownloadResponse> {
  return ApiClient.instance.post<TelegramMediaDownloadResponse>(
    '/api/v1/integrations/telegram/provider-media/download',
    request,
    'Telegram media download failed'
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
    '/api/v1/integrations/telegram/fixtures/messages',
    request,
    'Telegram fixture message ingest failed'
  )
}

// --- QR Login ---
export async function startTelegramQrLogin(
  request: TelegramQrLoginStartRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    '/api/v1/integrations/telegram/login/qr/start',
    request,
    'Telegram QR login start failed'
  )
}

export async function getTelegramQrLoginStatus(setupId: string): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.get<TelegramQrLoginStatusResponse>(
    `/api/v1/integrations/telegram/login/qr/${encodeURIComponent(setupId)}`,
    'Telegram QR login status request failed'
  )
}

export async function cancelTelegramQrLogin(setupId: string): Promise<{ setup_id: string; cancelled: boolean }> {
  return ApiClient.instance.delete<{ setup_id: string; cancelled: boolean }>(
    `/api/v1/integrations/telegram/login/qr/${encodeURIComponent(setupId)}`,
    'Telegram QR login cancel failed'
  )
}

export async function submitTelegramQrPassword(
  setupId: string,
  request: TelegramQrLoginPasswordRequest
): Promise<TelegramQrLoginStatusResponse> {
  return ApiClient.instance.post<TelegramQrLoginStatusResponse>(
    `/api/v1/integrations/telegram/login/qr/${encodeURIComponent(setupId)}/password`,
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

export { fetchTelegramTopicSearch } from './telegramTopics'
export {
  addTelegramReaction,
  deleteTelegramMessage,
  editTelegramMessage,
  fetchTelegramCommands,
  fetchTelegramMessageTombstones,
  fetchTelegramMessageVersions,
  fetchTelegramReactions,
  forwardTelegramMessage,
  removeTelegramReaction,
  replyToTelegramMessage,
  restoreTelegramMessageVisibility,
} from './telegramLifecycle'
