import type {
  TelegramCapabilitiesResponse,
  TelegramChat,
  TelegramChatSyncResponse,
  TelegramMediaDownloadRequest,
  TelegramMediaDownloadResponse,
  TelegramMessage,
  TelegramRuntimeStatus,
} from '../types/telegram'
import {
  downloadTelegramMedia,
  fetchTelegramAccounts,
  fetchTelegramCapabilities,
  fetchTelegramRuntimeStatus,
  restartTelegramRuntime,
  startTelegramRuntime,
  stopTelegramRuntime,
  syncTelegramChats,
  syncTelegramHistory,
} from './telegram'
import {
  fetchTelegramBusinessChats,
  fetchTelegramBusinessMessages,
  sendTelegramBusinessMessage,
} from '../../../shared/communications/telegramBusinessApi'

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
      fetchTelegramBusinessChats(undefined, 500),
      fetchTelegramBusinessMessages()
    ])

    const chats = chatResponse.items
    let nextChatId = selectedChatId
    if (!chats.some((chat) => chat.provider_chat_id === nextChatId)) {
      nextChatId = chats[0]?.provider_chat_id ?? ''
    }

    const messages = nextChatId
      ? (await fetchTelegramBusinessMessages(
          chats.find((chat) => chat.provider_chat_id === nextChatId)?.account_id,
          nextChatId,
          100
        )).items
      : messageResponse.items

    const accountIds = Array.from(
      new Set([
        ...accountResponse.items.map((account) => account.account_id),
        ...chats.map((chat) => chat.account_id)
      ].filter(Boolean))
    )
    const statusEntries = await Promise.all(
      accountIds.map(async (accountId) => {
        try {
          const status = await fetchTelegramRuntimeStatus(accountId)
          return [accountId, status] as const
        } catch {
          return null
        }
      })
    )
    const runtimeStatuses = Object.fromEntries(
      statusEntries.filter((entry): entry is [string, TelegramRuntimeStatus] => entry !== null)
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
    const result = await sendTelegramBusinessMessage({
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

export async function stopTelegramRuntimeFromUi(
  accountId: string
): Promise<{
  error: string
  message: string
  status: TelegramRuntimeStatus | null
}> {
  try {
    const status = await stopTelegramRuntime({ account_id: accountId })
    return {
      error: '',
      message: `Telegram runtime ${status.status}`,
      status
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram runtime stop failed',
      message: '',
      status: null
    }
  }
}

export async function restartTelegramRuntimeFromUi(
  accountId: string
): Promise<{
  error: string
  message: string
  status: TelegramRuntimeStatus | null
}> {
  try {
    const status = await restartTelegramRuntime({ account_id: accountId })
    return {
      error: '',
      message: `Telegram runtime ${status.status}`,
      status
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : 'Telegram runtime restart failed',
      message: '',
      status: null
    }
  }
}

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
