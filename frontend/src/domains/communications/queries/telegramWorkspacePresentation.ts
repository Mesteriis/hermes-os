import type { TelegramConversationRuntimeAction } from '../../../shared/communications/types/telegramRuntimeActions'
import type { TelegramChat, TelegramChatGroupFilter } from '../../../shared/communications/types/telegram'

export function telegramChatNeedsRead(chat: Pick<TelegramChat, 'metadata'>): boolean {
  const unreadCount = chat.metadata.provider_unread_count ?? chat.metadata.unread_count

  return chat.metadata.is_marked_as_unread === true
    || (typeof unreadCount === 'number' && Number.isInteger(unreadCount) && unreadCount > 0)
}

export function telegramForwardTargets(
  chats: readonly TelegramChat[],
  currentProviderChatId: string | null
): TelegramChat[] {
  return chats.filter((chat) => chat.provider_chat_id !== currentProviderChatId)
}

export function telegramProviderFolders(
  folders: readonly TelegramChatGroupFilter[]
): TelegramChatGroupFilter[] {
  return folders.filter((folder) => folder.provider_folder_id != null)
}

export function latestTelegramInboundMessageId(
  messages: readonly { delivery_state: string; provider_message_id: string }[]
): string | undefined {
  return messages.find((message) => message.delivery_state === 'received')?.provider_message_id
}

export function telegramTdlibMessageId(providerMessageId: string | null | undefined): number | undefined {
  const suffix = providerMessageId?.trim().split(':').at(-1)
  if (!suffix || !/^\d+$/.test(suffix)) return undefined
  const value = Number(suffix)
  return Number.isSafeInteger(value) && value > 0 ? value : undefined
}

export function telegramMessageSendStatus(status: string): string {
  switch (status) {
    case 'sent':
      return 'Telegram message sent.'
    case 'queued':
    case 'retrying':
      return 'Telegram message queued.'
    case 'failed':
    case 'send_failed':
    case 'send_blocked':
      return 'Telegram message failed.'
    default:
      return `Telegram message status: ${status}.`
  }
}

export function telegramRuntimeActionStatus(action: TelegramConversationRuntimeAction): string {
  const labels: Record<TelegramConversationRuntimeAction, string> = {
    archive: 'Telegram archive command queued.',
    download_media: 'Telegram media download queued.',
    folder_add: 'Telegram folder add command queued.',
    folder_remove: 'Telegram folder remove command queued.',
    folder_reassign: 'Telegram folder reassignment queued.',
    join: 'Telegram join command queued.',
    leave: 'Telegram leave command queued.',
    mark_read: 'Telegram read-state command queued.',
    mark_unread: 'Telegram unread-state command queued.',
    mute: 'Telegram mute command queued.',
    pin: 'Telegram pin command queued.',
    sync_latest: 'Telegram latest-history sync requested.',
    sync_members: 'Telegram member sync requested.',
    sync_older: 'Telegram older-history sync requested.',
    sync_full: 'Telegram full-history sync requested.',
    unarchive: 'Telegram unarchive command queued.',
    unmute: 'Telegram unmute command queued.',
    unpin: 'Telegram unpin command queued.',
    upload_media: 'Telegram media upload queued.',
  }
  return labels[action]
}
