import type { TelegramChat, TelegramMessage } from '../types/telegram'
import { telegramChatLastReadInboxProviderMessageId } from './telegram'

export type TelegramThreadReadProgress = {
  lastReadProviderMessageId: string | null
  lastReadMessageId: string | null
  boundaryAfterMessageId: string | null
  hasUnreadAfterBoundary: boolean
}

export function telegramProviderMessageNumericId(
  providerMessageId: string | null | undefined
): number | null {
  const suffix = providerMessageId?.split(':').at(-1)?.trim()
  if (!suffix) return null
  const parsed = Number.parseInt(suffix, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null
}

export function telegramThreadReadProgress(
  chat: TelegramChat,
  messages: TelegramMessage[]
): TelegramThreadReadProgress {
  const lastReadProviderMessageId = telegramChatLastReadInboxProviderMessageId(chat).trim()
  const lastReadNumericId = telegramProviderMessageNumericId(lastReadProviderMessageId)
  if (!lastReadProviderMessageId || lastReadNumericId === null) {
    return {
      lastReadProviderMessageId: null,
      lastReadMessageId: null,
      boundaryAfterMessageId: null,
      hasUnreadAfterBoundary: false,
    }
  }

  let lastReadMessageId: string | null = null
  let hasUnreadAfterBoundary = false

  for (const message of messages) {
    if (message.provider_chat_id !== chat.provider_chat_id) continue
    const messageNumericId = telegramProviderMessageNumericId(message.provider_message_id)
    if (messageNumericId === null) continue
    if (messageNumericId <= lastReadNumericId) {
      lastReadMessageId = message.message_id
      continue
    }
    if (lastReadMessageId !== null) {
      hasUnreadAfterBoundary = true
      break
    }
  }

  return {
    lastReadProviderMessageId,
    lastReadMessageId,
    boundaryAfterMessageId: hasUnreadAfterBoundary ? lastReadMessageId : null,
    hasUnreadAfterBoundary,
  }
}

export function telegramLatestReadableProviderMessageId(
  chat: TelegramChat,
  messages: TelegramMessage[]
): string | null {
  let latestMessageId: string | null = null
  let latestNumericId: number | null = null

  for (const message of messages) {
    if (message.provider_chat_id !== chat.provider_chat_id) continue
    if (message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run') continue
    const numericId = telegramProviderMessageNumericId(message.provider_message_id)
    if (numericId === null) continue
    if (latestNumericId === null || numericId >= latestNumericId) {
      latestNumericId = numericId
      latestMessageId = message.provider_message_id
    }
  }

  return latestMessageId
}

export function telegramCanMarkMessageRead(
  chat: TelegramChat,
  messages: TelegramMessage[],
  message: TelegramMessage
): boolean {
  const unreadCount = typeof chat.metadata.unread_count === 'number' ? chat.metadata.unread_count : 0
  if (unreadCount <= 0) return false
  if (message.provider_chat_id !== chat.provider_chat_id) return false
  if (message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run') return false
  return telegramLatestReadableProviderMessageId(chat, messages) === message.provider_message_id
}
