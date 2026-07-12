import type { TelegramChat } from '../../../shared/communications/types/telegram'
import type { TelegramConversationRuntimeActionRunner } from '../../../shared/communications/types/telegramRuntimeActions'

type TelegramInitialHistorySyncOptions = {
  resolveRunner: () => TelegramConversationRuntimeActionRunner | undefined
  refetchMessages: () => Promise<unknown>
  messageCount: () => number
  waitForHistoryProjection?: (initialMessageCount: number) => Promise<void>
  setError: (value: string) => void
  setSyncing: (value: boolean) => void
}

const PROJECTION_RETRY_COUNT = 10
const PROJECTION_RETRY_DELAY_MS = 250

type TelegramProjectionWaitOptions = {
  initialMessageCount: number
  messageCount: () => number
  refetchMessages: () => Promise<unknown>
  wait?: (milliseconds: number) => Promise<void>
}

/**
 * TDLib history is accepted and projected by separate durable consumers. Keep
 * the dialog in its loading state until that projection is visible locally.
 */
export async function waitForTelegramHistoryProjection({
  initialMessageCount,
  messageCount,
  refetchMessages,
  wait = delay,
}: TelegramProjectionWaitOptions): Promise<void> {
  for (let attempt = 0; attempt < PROJECTION_RETRY_COUNT; attempt += 1) {
    await refetchMessages()
    if (messageCount() > initialMessageCount) return
    if (attempt + 1 < PROJECTION_RETRY_COUNT) {
      await wait(PROJECTION_RETRY_DELAY_MS)
    }
  }
}

// TDLib history arrives through the observation pipeline. Private conversations
// are fully captured once; groups stay lazy unless their local policy opts in.
export function createTelegramInitialHistorySynchronizer(
  options: TelegramInitialHistorySyncOptions
): (chat: TelegramChat) => Promise<void> {
  const requestedChatKeys = new Set<string>()
  let isSyncing = false

  return async (chat) => {
    const runner = options.resolveRunner()
    const chatKey = `${chat.account_id}:${chat.provider_chat_id}`
    if (!runner || isSyncing || requestedChatKeys.has(chatKey)) return

    requestedChatKeys.add(chatKey)
    isSyncing = true
    options.setSyncing(true)
    options.setError('')
    const initialMessageCount = options.messageCount()
    const shouldLoadFullHistory = chat.chat_kind === 'private'
      || chat.metadata.full_history_sync_enabled === true
    try {
      await runner({
        action: shouldLoadFullHistory ? 'sync_full' : 'sync_latest',
        accountId: chat.account_id,
        providerChatId: chat.provider_chat_id,
        telegramChatId: chat.telegram_chat_id,
      })
      if (options.waitForHistoryProjection) {
        await options.waitForHistoryProjection(initialMessageCount)
      } else {
        await waitForTelegramHistoryProjection({
          initialMessageCount,
          messageCount: options.messageCount,
          refetchMessages: options.refetchMessages,
        })
      }
    } catch (error) {
      requestedChatKeys.delete(chatKey)
      options.setError(error instanceof Error ? error.message : 'Telegram history sync failed.')
    } finally {
      isSyncing = false
      options.setSyncing(false)
    }
  }
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds))
}
